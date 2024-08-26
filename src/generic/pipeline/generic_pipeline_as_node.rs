use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    generic::{
        node::{Node, NodeOutput},
        pipeline::{GenericPipeline, PipelineError, PipelineOutput, PipelineStorage},
    },
    pipeline::Pipeline,
};

/// Type that wraps around built [`GenericPipeline`](crate::generic::pipeline::GenericPipeline) to crate a [`Node`] from it.
///
/// This type correctly converts [`PipelineOutput<T>`](PipelineOutput) output type into [`NodeOutput<T>`](NodeOutput).
///
/// Example that shows usage of [`GenericPipelineAsNode`].
/// ```
/// // some node implementation
/// #[derive(Clone, Default, Debug)]
/// struct ForwardNode<T: Default> {
///     _type: PhantomData<T>,
///     // tells us how many times the node ran
///     ran: Arc<RwLock<usize>>,
/// }
/// #
/// #[async_trait]
/// impl<T: Send + Sync + Default + Clone + 'static> Node for ForwardNode<T> {
///     type Input = T;
///     type Output = T;
///     type Error = ();
///
///     async fn run(&mut self, input: Self::Input, pipeline_storage: &mut PipelineStorage) -> Result<NodeOutput<Self::Output>, Self::Error> {
///         let mut ran = self.ran.write().unwrap();
///         *ran += 1;
///         // here you want to actually do something like some io bound operation...
///         Self::advance(input).into()
///     }
/// }
///
/// #[derive(Debug)]
/// enum MyPipelineError {
///     WrongPipelineOutput {
///         node_type_name: &'static str,
///         data: Box<dyn AnyDebug>,
///         expected_type_name: &'static str,
///         got_type_name: &'static str,
///     },
///     NodeNotFound(&'static str),
///     SomeNodeError,
/// }
/// #
/// // conversion from generic pipeline error into ours
/// impl From<PipelineError> for MyPipelineError //...
/// # {
/// #     fn from(value: PipelineError) -> Self {
/// #         match value {
/// #             PipelineError::WrongOutputTypeForPipeline {
/// #                 data,
/// #                 node_type_name,
/// #                 expected_type_name,
/// #                 got_type_name,
/// #             } => Self::WrongPipelineOutput {
/// #                 data,
/// #                 expected_type_name,
/// #                 got_type_name,
/// #                 node_type_name,
/// #             },
/// #             PipelineError::NodeWithTypeNotFound { node_type_name } => {
/// #                 Self::NodeNotFound(node_type_name)
/// #             }
/// #         }
/// #     }
/// # }
/// #
/// // conversion from Node errors into ours
/// impl From<()> for MyPipelineError {
///     fn from(_value: ()) -> Self {
///         Self::SomeNodeError
///     }
/// }
///
/// # use orchestrator::{async_trait, pipeline::Pipeline, generic::{AnyDebug, pipeline::{GenericPipeline, PipelineError, PipelineOutput, PipelineStorage}, node::{Node, Returnable, NodeOutput}}};
/// # use std::marker::PhantomData;
/// # use std::sync::{RwLock, Arc};
/// #[tokio::main]
/// async fn main() {
///     let fwd_node = ForwardNode::<String>::default();
///
///     // construct generic pipeline that takes and returns a String
///     let pipeline = GenericPipeline::<String, String, MyPipelineError>::builder()
///         // construct and add a nodes that take and return a String
///         .add_node(fwd_node.clone())
///         .add_node(ForwardNode::<String>::default())
///         .add_node(ForwardNode::<String>::default())
///         // now just finish the pipeline
///         .finish();
///
///     // convert pipeline into node
///     let pipeline_as_node = pipeline.into_node();
///
///     // construct generic pipeline that takes and returns a string
///     let pipeline = GenericPipeline::<String, String, MyPipelineError>::builder()
///         // add a node that was created from pipeline
///         .add_node(pipeline_as_node.clone())
///         .add_node(pipeline_as_node.clone())
///         .add_node(pipeline_as_node)
///         // now just finish the pipeline
///         .finish();
///
///     // run the pipeline
///     let res = pipeline.run("match".into()).await;
///
///     // pipeline should run successfully
///     // and return the same thing that was at input
///     let expected: Result<_, MyPipelineError> = Ok(PipelineOutput::Done("match".to_owned()));
///     assert!(matches!(res, expected));
///
///     // node should have ran 3 times
///     let ran = fwd_node.ran.read().unwrap();
///     assert_eq!(*ran, 3);
/// }
/// ```
#[derive(Debug)]
pub struct GenericPipelineAsNode<Input, Output, Error> {
    pipeline: Arc<GenericPipeline<Input, Output, Error>>,
    share_pipeline_storage: bool,
}

impl<Input, Output, Error> GenericPipelineAsNode<Input, Output, Error> {
    /// Creates new instance of [`GenericPipelineAsNode`] from [`GenericPipeline`].
    #[must_use]
    pub fn new(pipeline: GenericPipeline<Input, Output, Error>) -> Self {
        Self {
            pipeline: Arc::new(pipeline),
            share_pipeline_storage: false,
        }
    }

    /// Enables sharing of [`PipelineStorage`].
    ///
    /// When used [`PipelineStorage`] that is passed into this [`Node`]
    /// is then passed to nodes that are contained in [`GenericPipeline`](crate::generic::pipeline::GenericPipeline)
    /// from which this node was built.
    #[must_use]
    pub fn share_pipeline_storage(mut self) -> Self {
        self.share_pipeline_storage = true;
        self
    }
}
impl<Input, Output, Error> From<GenericPipeline<Input, Output, Error>>
    for GenericPipelineAsNode<Input, Output, Error>
{
    fn from(value: GenericPipeline<Input, Output, Error>) -> Self {
        Self::new(value)
    }
}

impl<Input, Output, Error> GenericPipeline<Input, Output, Error> {
    /// Converts [`GenericPipeline`] into [`Node`].
    #[must_use]
    pub fn into_node(self) -> GenericPipelineAsNode<Input, Output, Error> {
        GenericPipelineAsNode::new(self)
    }
}
impl<Input, Output, Error> Clone for GenericPipelineAsNode<Input, Output, Error> {
    fn clone(&self) -> Self {
        Self {
            pipeline: self.pipeline.clone(),
            share_pipeline_storage: self.share_pipeline_storage,
        }
    }
}

#[cfg_attr(not(all(doc, not(doctest))), async_trait)]
impl<Input, Output, Error> Node for GenericPipelineAsNode<Input, Output, Error>
where
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    PipelineError: Into<Error>,
{
    type Input = Input;
    type Output = Output;
    type Error = Error;

    async fn run(
        &mut self,
        input: Self::Input,
        pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error> {
        if self.share_pipeline_storage {
            return Ok(self
                .pipeline
                .run_with_pipeline_storage(input, pipeline_storage)
                .await?
                .into());
        }
        Ok(self.pipeline.run(input).await?.into())
    }
}

impl<T> From<PipelineOutput<T>> for NodeOutput<T> {
    fn from(value: PipelineOutput<T>) -> Self {
        match value {
            PipelineOutput::SoftFail => NodeOutput::SoftFail,
            PipelineOutput::Done(data) => NodeOutput::Advance(data),
        }
    }
}
