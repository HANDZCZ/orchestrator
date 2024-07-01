use std::sync::Arc;

use async_trait::async_trait;

use crate::{generic::pipeline::PipelineOutput, pipeline::Pipeline};

use super::{ErrorInner, GenericOrchestrator};

#[derive(Debug)]
pub struct GenericOrchestratorAsPipeline<Input, Output, Error> {
    orchestrator: Arc<GenericOrchestrator<Input, Output, Error>>,
}

impl<Input, Output, Error> GenericOrchestratorAsPipeline<Input, Output, Error> {
    #[must_use]
    pub fn new(orchestrator: GenericOrchestrator<Input, Output, Error>) -> Self {
        Self {
            orchestrator: Arc::new(orchestrator),
        }
    }
}
impl<Input, Output, Error> From<GenericOrchestrator<Input, Output, Error>>
    for GenericOrchestratorAsPipeline<Input, Output, Error>
{
    fn from(value: GenericOrchestrator<Input, Output, Error>) -> Self {
        Self::new(value)
    }
}

impl<Input, Output, Error> GenericOrchestrator<Input, Output, Error> {
    #[must_use]
    pub fn into_pipeline(self) -> GenericOrchestratorAsPipeline<Input, Output, Error> {
        GenericOrchestratorAsPipeline::new(self)
    }
}
impl<Input, Output, Error> Clone for GenericOrchestratorAsPipeline<Input, Output, Error> {
    fn clone(&self) -> Self {
        Self {
            orchestrator: self.orchestrator.clone(),
        }
    }
}

#[cfg_attr(not(docs_cfg), async_trait)]
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
