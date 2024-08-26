use std::{fmt::Debug, hash::Hash, sync::Arc};

use async_trait::async_trait;

use crate::{
    generic::pipeline::PipelineOutput,
    orchestrator::{
        DebuggableRunInnerOrchestrator, ErrorInner, Orchestrator, OrchestratorRunInner,
    },
    pipeline::Pipeline,
};

use super::{GenericOrchestrator, KeyedGenericOrchestrator, OrchestratorError};

/// Type that wraps around some generic orchestrator type to crate a [`Pipeline`] from it.
///
/// This type correctly converts [`AllPipelinesSoftFailed`](crate::generic::orchestrator::OrchestratorError::AllPipelinesSoftFailed) error into [`PipelineOutput::SoftFail`].
///
/// Example that shows usage of [`GenericOrchestratorAsPipeline`].
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
/// #     fn from(_value: orchestrator::generic::orchestrator::OrchestratorError) -> Self {
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
/// # use orchestrator::{async_trait, pipeline::Pipeline, orchestrator::Orchestrator, generic::{pipeline::PipelineOutput, orchestrator::{GenericOrchestrator, OrchestratorError}}};
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
///     // convert orchestrator into pipeline
///     let orchestrator_as_pipeline = orchestrator.into_pipeline();
///
///     // construct generic orchestrator that takes and returns a string
///     let mut orchestrator = GenericOrchestrator::<String, String, MyOrchestratorError<()>>::new();
///     // construct and add an orchestrator as a pipeline that takes and returns a String
///     // this orchestrator doesn't have any pipelines so it returns an error AllPipelinesSoftFailed
///     let empty_orchestrator = GenericOrchestrator::<String, String, MyOrchestratorError<()>>::new();
///     orchestrator.add_pipeline(empty_orchestrator.into_pipeline());
///     // add a pipeline that was created from orchestrator
///     orchestrator.add_pipeline(orchestrator_as_pipeline);
///
///     // run the orchestrator
///     let output = orchestrator.run(input.clone()).await;
///
///     // output should be the same as input
///     // since the error from the empty orchestrator is AllPipelinesSoftFailed
///     // this error is simply converted into PipelineOutput::SoftFail
///     // that means that the second pipeline is ran and that one returns successfully
///     assert_eq!(output, Ok(input));
///
///     // pipeline should have ran only once
///     let ran = string_pipeline.ran.read().unwrap();
///     assert_eq!(*ran, 1);
/// }
/// ```
#[derive(Debug)]
pub struct GenericOrchestratorAsPipeline<Input, Output, Error> {
    #[cfg(all(doc, not(doctest)))]
    _types: std::marker::PhantomData<(Input, Output, Error)>,
    #[cfg(not(all(doc, not(doctest))))]
    orchestrator:
        Arc<dyn DebuggableRunInnerOrchestrator<Input = Input, Output = Output, Error = Error>>,
}

impl<Input, Output, Error> GenericOrchestratorAsPipeline<Input, Output, Error> {
    /// Creates new instance of [`GenericOrchestratorAsPipeline`] from some generic orchestrator.
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

#[cfg_attr(not(all(doc, not(doctest))), async_trait)]
impl<Input, Output, Error> Pipeline for GenericOrchestratorAsPipeline<Input, Output, Error>
where
    Input: Clone + Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    type Input = Input;
    type Output = PipelineOutput<Output>;
    type Error = Error;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        match self.orchestrator.run_inner(input).await {
            Ok(output) => Ok(PipelineOutput::Done(output)),
            Err(ErrorInner::AllPipelinesSoftFailed) => Ok(PipelineOutput::SoftFail),
            Err(ErrorInner::Other(e)) => Err(e),
        }
    }
}

impl<Input, Output, Error> Clone for GenericOrchestratorAsPipeline<Input, Output, Error> {
    fn clone(&self) -> Self {
        Self {
            orchestrator: self.orchestrator.clone(),
        }
    }
}

impl<Input, Output, Error, T> From<T> for GenericOrchestratorAsPipeline<Input, Output, Error>
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

macro_rules! impl_into_pipeline {
    ($name:ident) => {
        impl_into_pipeline!($name,);
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
            /// Correctly converts generic orchestrator into [`Pipeline`].
            ///
            /// For more info look at [`GenericOrchestratorAsPipeline`].
            #[must_use]
            pub fn into_pipeline(self) -> GenericOrchestratorAsPipeline<Input, Output, Error> {
                GenericOrchestratorAsPipeline::new(self)
            }
        }
    };
}

impl_into_pipeline!(GenericOrchestrator);
impl_into_pipeline!(KeyedGenericOrchestrator, Key: Debug + Send + Sync + Eq + Hash + 'static);
