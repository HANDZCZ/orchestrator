use std::fmt::Debug;

use async_trait::async_trait;

use crate::{generic::pipeline::PipelineOutput, orchestrator::Orchestrator, pipeline::Pipeline};

use super::internal_pipeline::{InternalPipeline, InternalPipelineStruct};

#[derive(Debug)]
pub enum OrchestratorError {
    AllPipelinesSoftFailed,
}

pub struct GenericOrchestrator<
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: From<OrchestratorError> + Send + Sync + 'static,
> {
    pipelines: Vec<Box<dyn InternalPipeline<Input, PipelineOutput<Output>, Error>>>,
}

#[async_trait]
impl<
        Input: Debug + Send + Sync + Clone + 'static,
        Output: Debug + Send + Sync + 'static,
        Error: From<OrchestratorError> + Send + Sync + 'static,
    > Orchestrator for GenericOrchestrator<Input, Output, Error>
{
    type Input = Input;
    type Output = Output;
    type Error = Error;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        for pipeline in &self.pipelines {
            match pipeline.run(input.clone()).await? {
                PipelineOutput::SoftFail => continue,
                PipelineOutput::Done(output) => return Ok(output),
            }
        }
        Err(Error::from(OrchestratorError::AllPipelinesSoftFailed))
    }
}

impl<
        Input: Debug + Send + Sync + Clone + 'static,
        Output: Debug + Send + Sync + 'static,
        Error: From<OrchestratorError> + Send + Sync + 'static,
    > GenericOrchestrator<Input, Output, Error>
{
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
        }
    }
    pub fn add_pipeline<PipelineError: Into<Error>>(
        &mut self,
        pipeline: impl Pipeline<Input = Input, Output = PipelineOutput<Output>, Error = PipelineError>
            + Debug,
    ) {
        self.pipelines
            .push(Box::new(InternalPipelineStruct::new(pipeline)));
    }
}

impl<
        Input: Debug + Send + Sync + Clone + 'static,
        Output: Debug + Send + Sync + 'static,
        Error: From<OrchestratorError> + Send + Sync + 'static,
    > Default for GenericOrchestrator<Input, Output, Error>
{
    fn default() -> Self {
        Self::new()
    }
}
