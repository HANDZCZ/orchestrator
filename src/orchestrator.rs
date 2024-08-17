use async_trait::async_trait;

/// Defines what methods should be implemented for an orchestrator and what types should be used in [`Pipelines`](crate::pipeline::Pipeline).
///
/// Example how can [`Orchestrator`] trait be implemented.
/// ```no_run
/// use orchestrator::{async_trait, pipeline::Pipeline, generic::pipeline::PipelineOutput, orchestrator::Orchestrator};
///
/// enum MyOrchestratorError {
///     AllPipelinesSoftFailed,
///     PipelineError,
/// }
///
/// struct MyOrchestrator {
///     pipelines: Vec<Box<dyn Pipeline<Input = String, Output = PipelineOutput<String>, Error = ()>>>,
/// }
///
/// #[async_trait]
/// impl Orchestrator for MyOrchestrator {
///     type Input = String;
///     type Output = String;
///     type Error = MyOrchestratorError;
///
///     async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
///         for pipeline in &self.pipelines {
///             match pipeline.run(input.clone()).await {
///                 Ok(PipelineOutput::SoftFail) => continue,
///                 Ok(PipelineOutput::Done(res)) => return Ok(res),
///                 Err(_) => return Err(MyOrchestratorError::PipelineError),
///             }
///         }
///         Err(MyOrchestratorError::AllPipelinesSoftFailed)
///     }
/// }
/// ```
#[cfg_attr(not(docs_cfg), async_trait)]
pub trait Orchestrator: Send + Sync + 'static
where
    Self::Input: Send + Sync + Clone + 'static,
    Self::Output: Send + Sync + 'static,
    Self::Error: Send + Sync + 'static,
{
    /// Orchestrator input type.
    type Input;
    /// Orchestrator output type.
    type Output;
    /// Orchestrator error type.
    type Error;

    /// Runs the [`Pipelines`](crate::pipeline::Pipeline) and returns the result when some [`Pipeline`](crate::pipeline::Pipeline) finishes successfully.
    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

pub(crate) trait DebuggableOrchestrator: Orchestrator + Debug {}
impl<T> DebuggableOrchestrator for T where T: Orchestrator + Debug {}

pub(crate) enum ErrorInner<E> {
    AllPipelinesSoftFailed,
    Other(E),
}
