use std::fmt::Debug;

use async_trait::async_trait;

use crate::{generic::pipeline::PipelineOutput, orchestrator::Orchestrator, pipeline::Pipeline};

use super::internal_pipeline::{InternalPipeline, InternalPipelineStruct};

/// Defines which errors can occur in [`GenericOrchestrator`].
#[derive(Debug)]
pub enum OrchestratorError {
    /// All the pipelines added to orchestrator soft failed.
    ///
    /// For more information about soft fail look at [`NodeOutput::SoftFail`](crate::generic::node::NodeOutput::SoftFail).
    AllPipelinesSoftFailed,
}

/// Generic implementation of [`Orchestrator`] trait.
/// That takes some input type and returns some output type or some error type.
#[derive(Debug)]
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
    /// Creates a new instance of [`GenericOrchestrator`].
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
        }
    }

    /// Adds a pipeline to the [`GenericOrchestrator`] which needs to have same the input and output as the [`GenericOrchestrator`].
    /// It also needs to have an error type that implements `Into<OrchestratorErrorType>`.
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
