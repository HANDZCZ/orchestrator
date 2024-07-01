use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;

use crate::{orchestrator::Orchestrator, pipeline::Pipeline};

trait DebuggableOrchestrator: Orchestrator + Debug {}
impl<T> DebuggableOrchestrator for T where T: Orchestrator + Debug {}

#[derive(Debug)]
pub struct OrchestratorAsPipeline<Input, Output, Error> {
    #[cfg(docs_cfg)]
    _types: std::marker::PhantomData<(Input, Output, Error)>,
    #[cfg(not(docs_cfg))]
    orchestrator:
        Arc<Box<dyn DebuggableOrchestrator<Input = Input, Output = Output, Error = Error>>>,
}
impl<Input, Output, Error> Clone for OrchestratorAsPipeline<Input, Output, Error> {
    fn clone(&self) -> Self {
        Self {
            orchestrator: self.orchestrator.clone(),
        }
    }
}
impl<Input, Output, Error> OrchestratorAsPipeline<Input, Output, Error> {
    pub fn new<OrchestratorType>(orchestrator: OrchestratorType) -> Self
    where
        OrchestratorType:
            Orchestrator<Input = Input, Output = Output, Error = Error> + Debug + 'static,
    {
        Self {
            orchestrator: Arc::new(Box::new(orchestrator)),
        }
    }
}

#[cfg_attr(not(docs_cfg), async_trait)]
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

pub trait OrchestratorAsPipelineExt: Orchestrator + Debug {
    fn into_pipeline(self) -> OrchestratorAsPipeline<Self::Input, Self::Output, Self::Error>
    where
        Self: Sized,
    {
        OrchestratorAsPipeline::new(self)
    }
}
impl<T> OrchestratorAsPipelineExt for T where T: Orchestrator + Debug {}
