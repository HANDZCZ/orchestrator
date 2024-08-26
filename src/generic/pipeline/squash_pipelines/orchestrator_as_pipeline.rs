use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;

use crate::{
    orchestrator::{DebuggableOrchestrator, Orchestrator},
    pipeline::Pipeline,
};

/// Type that wraps around [`Orchestrator`] to crate a [`Pipeline`] from it.
///
/// Example that shows usage of [`OrchestratorAsPipeline`].
/// ```no_run
/// use orchestrator::{
///     async_trait,
///     pipeline::Pipeline,
///     generic::pipeline::{
///         squash_pipelines::OrchestratorAsPipelineExt,
///         PipelineOutput
///     },
///     orchestrator::Orchestrator
/// };
/// use std::fmt::Debug;
///
/// enum MyOrchestratorError {
///     AllPipelinesSoftFailed,
///     PipelineError,
/// }
///
/// struct MyOrchestrator {
///     pipelines: Vec<Box<dyn Pipeline<Input = String, Output = String, Error = ()>>>,
/// }
/// impl Debug for MyOrchestrator //...
/// # {
/// #    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// #        f.debug_struct("MyOrchestrator").finish_non_exhaustive()
/// #    }
/// # }
///
/// #[async_trait]
/// impl Orchestrator for MyOrchestrator {
///     type Input = String;
///     type Output = String;
///     type Error = MyOrchestratorError;
///
///     async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
///         let mut res = String::new();
///         for pipeline in &self.pipelines {
///             res += &pipeline.run(input.clone())
///                 .await
///                 .map_err(|e| MyOrchestratorError::PipelineError)?;
///         }
///         Ok(res)
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     // construct orchestrator with some pipelines
///     let orchestrator = MyOrchestrator {
///         // ...
/// #         pipelines: Vec::new()
///     };
///     // convert it to pipeline
///     let orchestrator_as_node = orchestrator.into_pipeline();
///     // do something with the pipeline
///     // ...
/// }
/// ```
#[derive(Debug)]
pub struct OrchestratorAsPipeline<Input, Output, Error> {
    #[cfg(all(doc, not(doctest)))]
    _types: std::marker::PhantomData<(Input, Output, Error)>,
    #[cfg(not(all(doc, not(doctest))))]
    orchestrator: Arc<dyn DebuggableOrchestrator<Input = Input, Output = Output, Error = Error>>,
}
impl<Input, Output, Error> Clone for OrchestratorAsPipeline<Input, Output, Error> {
    fn clone(&self) -> Self {
        Self {
            orchestrator: self.orchestrator.clone(),
        }
    }
}
impl<Input, Output, Error> OrchestratorAsPipeline<Input, Output, Error> {
    /// Creates new instance of [`OrchestratorAsPipeline`] from type that implements [`Orchestrator`] trait.
    pub fn new<OrchestratorType>(orchestrator: OrchestratorType) -> Self
    where
        OrchestratorType:
            Orchestrator<Input = Input, Output = Output, Error = Error> + Debug + 'static,
    {
        Self {
            orchestrator: Arc::new(orchestrator),
        }
    }
}

#[cfg_attr(not(all(doc, not(doctest))), async_trait)]
impl<Input, Output, Error> Pipeline for OrchestratorAsPipeline<Input, Output, Error>
where
    Input: Clone + Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    type Input = Input;
    type Output = Output;
    type Error = Error;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(self.orchestrator.run(input).await?)
    }
}

/// Extension for [`Orchestrator`] trait that converts orchestrator into [`Pipeline`].
pub trait OrchestratorAsPipelineExt: Orchestrator + Debug {
    /// Converts [`Orchestrator`] into [`Pipeline`].
    fn into_pipeline(self) -> OrchestratorAsPipeline<Self::Input, Self::Output, Self::Error>
    where
        Self: Sized,
    {
        OrchestratorAsPipeline::new(self)
    }
}
impl<T> OrchestratorAsPipelineExt for T where T: Orchestrator + Debug {}
