use async_trait::async_trait;

/// Defines what methods should be implemented for a pipeline.
/// It also defines what is the input, output and error should a pipeline have.
#[cfg_attr(not(docs_cfg), async_trait)]
pub trait Pipeline: Sync + Send + 'static
where
    Self::Input: Send + Sync + 'static,
    Self::Output: Send + Sync + 'static,
    Self::Error: Send + Sync + 'static,
{
    /// Pipeline input type.
    type Input;
    /// Pipeline output type.
    type Output;
    /// Pipeline error type.
    type Error;

    /// Runs the pipeline.
    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
