use async_trait::async_trait;

#[async_trait]
pub trait Orchestrator
where
    Self::Input: Send + Sync + Clone + 'static,
    Self::Output: Send + Sync + 'static,
    Self::Error: Send + Sync + 'static,
{
    type Input;
    type Output;
    type Error;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
