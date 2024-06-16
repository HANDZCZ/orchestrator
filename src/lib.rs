#![warn(missing_docs)]
#![warn(unused_doc_comments)]
#![warn(clippy::empty_docs)]
#![warn(clippy::tabs_in_doc_comments)]
#![warn(clippy::suspicious_doc_comments)]
#![warn(clippy::test_attr_in_doctest)]
#![warn(rustdoc::private_intra_doc_links)]
#![warn(clippy::empty_line_after_doc_comments)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![forbid(rustdoc::broken_intra_doc_links)]
#![forbid(missing_debug_implementations)]
#![forbid(invalid_doc_attributes)]
#![cfg_attr(docs_cfg, feature(async_fn_in_trait))]

//! A crate that provides pipelines and an orchestrator.
//!
//! It provides traits [`Pipeline`](crate::pipeline::Pipeline) and [`Orchestrator`](crate::orchestrator::Orchestrator).
//! It also provide generic implementations ([`GenericPipeline`](crate::generic::pipeline::GenericPipeline),
//! [`GenericOrchestrator`](crate::generic::orchestrator::GenericOrchestrator))
//! of these traits.
//! To use the generic types you need to implement the [`Node`](crate::generic::node::Node) trait and mostly follow the compilers nagging.
//! You can also use [`FnPipeline`](crate::generic::pipeline::FnPipeline) and [`FnNode`](crate::generic::node::fn_node::FnNode) that implement traits
//! [`Pipeline`](crate::pipeline::Pipeline) or [`Node`](crate::generic::node::Node) and wrap around async functions.
//!
//! # Example
//!
//! Example usage of [`GenericPipeline`](crate::generic::pipeline::GenericPipeline),
//! [`GenericOrchestrator`](crate::generic::orchestrator::GenericOrchestrator) and
//! [`Node`](crate::generic::node::Node).
//! It also contains an implementation of the [`Pipeline`](crate::pipeline::Pipeline) trait.
//!
//! In this example you can also see input/output conversion between not only
//! orchestrator and pipeline (in this case pipeline error is also converted to orchestrator error),
//! but also between nodes in pipeline.
//! ```
//! // type to convert from and into
//! #[derive(Debug, Default, Clone, PartialEq)]
//! struct WrapString(String);
//! #
//! impl From<String> for WrapString //...
//! # {
//! #     fn from(value: String) -> Self {
//! #         WrapString(value)
//! #     }
//! # }
//! #
//! impl From<WrapString> for String //...
//! # {
//! #     fn from(value: WrapString) -> Self {
//! #         value.0
//! #     }
//! # }
//!
//! // some node implementation
//! #[derive(Clone, Default, Debug)]
//! struct ForwardNode<T: Default> {
//!     _type: PhantomData<T>,
//! }
//! #
//! #[async_trait]
//! impl<T: Send + Sync + Default + Clone + 'static> Node for ForwardNode<T> {
//!     type Input = T;
//!     type Output = T;
//!     type Error = ();
//!
//!     async fn run(&mut self, input: Self::Input, pipeline_storage: &mut PipelineStorage) -> Result<NodeOutput<Self::Output>, Self::Error> {
//!         // here you want to actually do something like some io bound operation...
//!         Self::advance(input).into()
//!     }
//! }
//!
//! // pipeline implementation that soft fails
//! #[derive(Default, Debug)]
//! struct SoftFailPipeline<T: Default> {
//!     _type: PhantomData<T>,
//! }
//! #
//! #[async_trait]
//! impl<T: Send + Sync + 'static + Default> Pipeline for SoftFailPipeline<T> {
//!     type Input = T;
//!     type Output = PipelineOutput<T>;
//!     type Error = MyPipelineError;
//!
//!     async fn run(&self, _input: Self::Input) -> Result<Self::Output, Self::Error> {
//!         Ok(PipelineOutput::SoftFail)
//!     }
//! }
//!
//! #[derive(Debug)]
//! enum MyPipelineError {
//!     WrongPipelineOutput {
//!         node_type_name: &'static str,
//!         data: Box<dyn AnyDebug>,
//!         expected_type_name: &'static str,
//!         got_type_name: &'static str,
//!     },
//!     NodeNotFound(&'static str),
//!     SomeNodeError,
//! }
//! #
//! // conversion from generic pipeline error into ours
//! impl From<PipelineError> for MyPipelineError //...
//! # {
//! #     fn from(value: PipelineError) -> Self {
//! #         match value {
//! #             PipelineError::WrongOutputTypeForPipeline {
//! #                 data,
//! #                 node_type_name,
//! #                 expected_type_name,
//! #                 got_type_name,
//! #             } => Self::WrongPipelineOutput {
//! #                 data,
//! #                 expected_type_name,
//! #                 got_type_name,
//! #                 node_type_name,
//! #             },
//! #             PipelineError::NodeWithTypeNotFound { node_type_name } => {
//! #                 Self::NodeNotFound(node_type_name)
//! #             }
//! #         }
//! #     }
//! # }
//! #
//! // conversion from Node errors into ours
//! impl From<()> for MyPipelineError //...
//! # {
//! #     fn from(_value: ()) -> Self {
//! #         Self::SomeNodeError
//! #     }
//! # }
//!
//! #[derive(Debug, PartialEq)]
//! enum MyOrchestratorError<T> {
//!     AllPipelinesSoftFailed,
//!     PipelineError(T),
//! }
//! #
//! // conversion from generic orchestrator error into ours
//! impl<T> From<OrchestratorError> for MyOrchestratorError<T> //...
//! # {
//! #     fn from(_value: OrchestratorError) -> Self {
//! #         Self::AllPipelinesSoftFailed
//! #     }
//! # }
//! #
//! impl From<MyPipelineError> for MyOrchestratorError<MyPipelineError> //...
//! # {
//! #     fn from(value: MyPipelineError) -> Self {
//! #         Self::PipelineError(value)
//! #     }
//! # }
//!
//! # use orchestrator::{async_trait, pipeline::Pipeline, orchestrator::Orchestrator, generic::{AnyDebug, orchestrator::{GenericOrchestrator, OrchestratorError}, pipeline::{GenericPipeline, PipelineError, PipelineOutput, PipelineStorage}, node::{Node, Returnable, NodeOutput}}};
//! # use std::marker::PhantomData;
//! #[tokio::main]
//! async fn main() {
//!     // construct generic pipeline that takes and returns a string
//!     // notice the builder pattern - it is needed for type safety
//!     let pipeline = GenericPipeline::<String, String, MyPipelineError>::new()
//!
//!         // construct and add a node that takes and returns a WrapString
//!         // the pipeline input (String) will be converted into WrapString
//!         // thanks to implementation of WrapString: From<String> above
//!         .add_node(ForwardNode::<WrapString>::default())
//!
//!         // construct and add a node that takes and returns a String
//!         // the node output (WrapString) wil be converted to String
//!         // thanks to implementation of String: From<WrapString> above
//!         .add_node(ForwardNode::<String>::default())
//!
//!         // construct and add a node that takes and returns a WrapString
//!         //   (input to this will be converted to WrapString like the first node)
//!         // the node output (WrapString) wil be converted to pipeline output (String)
//!         // thanks to implementation of String: From<WrapString> above
//!         .add_node(ForwardNode::<WrapString>::default())
//!
//!         // now just finish the pipeline
//!         // here is the actual check for converting
//!         // last nodes output type to pipeline output type (WrapString: Into<String>)
//!         .finish();
//!
//!     // construct generic orchestrator that takes and returns a WrapString
//!     let mut orchestrator = GenericOrchestrator::<WrapString, WrapString, MyOrchestratorError<_>>::new();
//!
//!     // add a pipeline that can soft fail
//!     orchestrator.add_pipeline(SoftFailPipeline::<WrapString>::default());
//!
//!     // construct and add a pipeline that takes and returns a String
//!     // the orchestrator input (WrapString) will be converted into String
//!     // thanks to implementation of WrapString: From<String> above
//!     // and the pipeline output (String) wil be converted to WrapString
//!     // thanks to implementation of String: From<WrapString> above
//!     orchestrator.add_pipeline(pipeline);
//!
//!     let input = WrapString("Nice".into());
//!     // run the orchestrator
//!     // soft fail pipeline will run first and then our generic pipeline
//!     // if the soft fail pipeline would have failed instead of soft failing
//!     // only the soft fail pipeline would have ran and error from that pipeline
//!     // would have been returned from orchestrator
//!     let output = orchestrator.run(input.clone()).await;
//!
//!     // output should be the same as input
//!     let expected: Result<_, MyOrchestratorError<MyPipelineError>> = Ok(input);
//!     assert!(matches!(output, expected));
//! }
//! ```

/// Module that houses stuff related to generics ([`Node`](crate::generic::node::Node),
/// [`GenericPipeline`](crate::generic::pipeline::GenericPipeline),
/// [`GenericOrchestrator`](crate::generic::orchestrator::GenericOrchestrator)
/// [`FnPipeline`](crate::generic::pipeline::FnPipeline),
/// [`FnNode`](crate::generic::node::fn_node::FnNode)).
pub mod generic;
/// Module that houses stuff related to [`Orchestrator`](crate::orchestrator::Orchestrator) trait.
pub mod orchestrator;
/// Module that houses stuff related to [`Pipeline`](crate::pipeline::Pipeline) trait.
pub mod pipeline;
pub use async_trait::async_trait;
