use std::{fmt::Debug, hash::Hash, sync::Arc};

use async_trait::async_trait;

use crate::{
    generic::{
        node::{Node, NodeOutput, Returnable},
        pipeline::PipelineStorage,
    },
    orchestrator::{
        DebuggableRunInnerOrchestrator, ErrorInner, Orchestrator, OrchestratorRunInner,
    },
};

use super::{GenericOrchestrator, KeyedGenericOrchestrator, OrchestratorError};

/// Type that wraps around some generic orchestrator type to crate a [`Node`] from it.
///
/// This type correctly converts [`AllPipelinesSoftFailed`](crate::generic::orchestrator::OrchestratorError::AllPipelinesSoftFailed) error into [`NodeOutput::SoftFail`].
///
/// Example that shows usage of [`GenericOrchestratorAsNode`].
/// ```
/// #[derive(Debug, PartialEq)]
/// enum MyOrchestratorError<T> {
///     AllPipelinesSoftFailed,
///     PipelineError(T),
/// }
/// #
/// // conversion from generic orchestrator error into ours
/// impl<T> From<OrchestratorError> for MyOrchestratorError<T> //...
/// # {
/// #     fn from(_value: OrchestratorError) -> Self {
/// #         Self::AllPipelinesSoftFailed
/// #     }
/// # }
/// #
/// // conversion from ForwardPipeline error into ours
/// impl From<()> for MyOrchestratorError<()> //...
/// # {
/// #     fn from(_value: ()) -> Self {
/// #         Self::PipelineError(())
/// #     }
/// # }
/// #
/// // conversion from generic pipeline error into ours
/// impl From<PipelineError> for MyOrchestratorError<()> //...
/// # {
/// #     fn from(_value: PipelineError) -> Self {
/// #         Self::PipelineError(())
/// #     }
/// # }
/// #
///
/// // some pipeline implementation
/// #[derive(Default, Debug, Clone)]
/// struct ForwardPipeline<T: Default> {
///     _type: PhantomData<T>,
///     // tells us how many times the pipeline ran
///     ran: Arc<RwLock<usize>>,
/// }
/// #
/// #[async_trait]
/// impl<T: Send + Sync + 'static + Default> Pipeline for ForwardPipeline<T> {
///     type Input = T;
///     type Output = PipelineOutput<T>;
///     type Error = ();
///
///     async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
///         let mut ran = self.ran.write().unwrap();
///         *ran += 1;
///         // here you want to actually do something like running some nodes
///         // but we will just forward input to output
///         Ok(PipelineOutput::Done(input))
///     }
/// }
///
/// # use orchestrator::{async_trait, pipeline::Pipeline, orchestrator::Orchestrator, generic::{pipeline::{GenericPipeline, PipelineOutput, PipelineError}, orchestrator::{GenericOrchestrator, OrchestratorError}}};
/// # use std::marker::PhantomData;
/// # use std::sync::{RwLock, Arc};
/// #[tokio::main]
/// async fn main() {
///     let input = "Nice".to_owned();
///
///     // construct generic orchestrator that takes and returns a string
///     let mut orchestrator = GenericOrchestrator::<String, String, MyOrchestratorError<()>>::new();
///     // construct and add a pipeline that takes and returns a String
///     let string_pipeline = ForwardPipeline::<String>::default();
///     orchestrator.add_pipeline(string_pipeline.clone());
///     orchestrator.add_pipeline(string_pipeline.clone());
///     orchestrator.add_pipeline(string_pipeline.clone());
///
///     // convert orchestrator into node
///     let orchestrator_as_node = orchestrator.into_node();
///
///     // construct generic orchestrator that takes and returns a string
///     let mut orchestrator = GenericOrchestrator::<String, String, MyOrchestratorError<()>>::new();
///     // construct and add an orchestrator as a node that takes and returns a String
///     // this orchestrator doesn't have any pipelines so it returns an error AllPipelinesSoftFailed
///     let empty_orchestrator = GenericOrchestrator::<String, String, MyOrchestratorError<()>>::new();
///     orchestrator.add_pipeline(
///         GenericPipeline::<String, String, MyOrchestratorError<()>>::builder()
///             .add_node(empty_orchestrator.into_node())
///             .finish()
///     );
///     // add orchestrator as node
///     orchestrator.add_pipeline(
///         GenericPipeline::<String, String, MyOrchestratorError<()>>::builder()
///             .add_node(orchestrator_as_node)
///             .finish()
///     );
///
///     // run the orchestrator
///     let output = orchestrator.run(input.clone()).await;
///
///     // output should be the same as input
///     // since the error from the empty orchestrator is AllPipelinesSoftFailed
///     // this error is simply converted into NodeOutput::SoftFail that is then converted to PipelineOutput::SoftFail
///     // that means that the second pipeline is ran and that one returns successfully
///     assert_eq!(output, Ok(input));
///
///     // pipeline should have ran only once
///     let ran = string_pipeline.ran.read().unwrap();
///     assert_eq!(*ran, 1);
/// }
/// ```
#[derive(Debug)]
pub struct GenericOrchestratorAsNode<Input, Output, Error> {
    #[cfg(docs_cfg)]
    _types: std::marker::PhantomData<(Input, Output, Error)>,
    #[cfg(not(docs_cfg))]
    orchestrator:
        Arc<dyn DebuggableRunInnerOrchestrator<Input = Input, Output = Output, Error = Error>>,
}

impl<Input, Output, Error> GenericOrchestratorAsNode<Input, Output, Error> {
    /// Creates new instance of [`GenericOrchestratorAsNode`] from some generic orchestrator.
    #[must_use]
    fn new<OrchestratorType>(orchestrator: OrchestratorType) -> Self
    where
        OrchestratorType: Orchestrator<Input = Input, Output = Output, Error = Error>
            + OrchestratorRunInner
            + Debug
            + 'static,
    {
        Self {
            orchestrator: Arc::new(orchestrator),
        }
    }
}

#[cfg_attr(not(docs_cfg), async_trait)]
impl<Input, Output, Error> Node for GenericOrchestratorAsNode<Input, Output, Error>
where
    Input: Clone + Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    type Input = Input;
    type Output = Output;
    type Error = Error;

    async fn run(
        &mut self,
        input: Self::Input,
        _pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error> {
        match self.orchestrator.run_inner(input).await {
            Ok(output) => Self::advance(output).into(),
            Err(ErrorInner::AllPipelinesSoftFailed) => Self::soft_fail().into(),
            Err(ErrorInner::Other(e)) => Err(e),
        }
    }
}

impl<Input, Output, Error> Clone for GenericOrchestratorAsNode<Input, Output, Error> {
    fn clone(&self) -> Self {
        Self {
            orchestrator: self.orchestrator.clone(),
        }
    }
}

impl<Input, Output, Error, T> From<T> for GenericOrchestratorAsNode<Input, Output, Error>
where
    T: Orchestrator<Input = Input, Output = Output, Error = Error> + Debug + OrchestratorRunInner,
    Input: Send + Sync + Clone + Debug + 'static,
    Output: Send + Sync + Debug + 'static,
    Error: Send + Sync + Debug + 'static,
    OrchestratorError: Into<Error>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

macro_rules! impl_into_node {
    ($name:ident) => {
        impl_into_node!($name,);
    };
    ($name:ident, $($arg:ident $(: $first_bound:tt $(+ $other_bounds:tt)*)?),*) => {
        impl<Input, Output, Error, $($arg),*> $name<Input, Output, Error, $($arg),*>
        where
            Input: Send + Sync + Clone + Debug + 'static,
            Output: Send + Sync + Debug + 'static,
            Error: Send + Sync + Debug + 'static,
            OrchestratorError: Into<Error>,
            $($($arg: $first_bound $(+ $other_bounds)*)?),*
        {
            /// Correctly converts generic orchestrator into [`Node`].
            ///
            /// For more info look at [`GenericOrchestratorAsNode`].
            #[must_use]
            pub fn into_node(self) -> GenericOrchestratorAsNode<Input, Output, Error> {
                GenericOrchestratorAsNode::new(self)
            }
        }
    };
}

impl_into_node!(GenericOrchestrator);
impl_into_node!(KeyedGenericOrchestrator, Key: Debug + Send + Sync + Eq + Hash + 'static);
