use async_trait::async_trait;

/// Defines what methods should be implemented for an orchestrator and what types should be used in [`Pipelines`](crate::pipeline::Pipeline).
#[cfg_attr(not(docs_cfg), async_trait)]
pub trait Orchestrator
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
