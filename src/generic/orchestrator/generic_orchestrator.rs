use std::fmt::Debug;

use async_trait::async_trait;

use crate::{orchestrator::Orchestrator, pipeline::Pipeline};

use super::internal_pipeline::{InternalPipeline, InternalPipelineOutput, InternalPipelineStruct};

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
pub struct GenericOrchestrator<Input, Output, Error> {
    pipelines: Vec<Box<dyn InternalPipeline<Input, InternalPipelineOutput<Output>, Error>>>,
}

#[cfg_attr(not(docs_cfg), async_trait)]
impl<Input, Output, Error> Orchestrator for GenericOrchestrator<Input, Output, Error>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    OrchestratorError: Into<Error>,
{
    type Input = Input;
    type Output = Output;
    type Error = Error;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        for pipeline in &self.pipelines {
            match pipeline.run(input.clone()).await? {
                InternalPipelineOutput::SoftFail => continue,
                InternalPipelineOutput::Done(output) => return Ok(output),
            }
        }
        Err(OrchestratorError::AllPipelinesSoftFailed.into())
    }
}

impl<Input, Output, Error> GenericOrchestrator<Input, Output, Error>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    OrchestratorError: Into<Error>,
{
    /// Creates a new instance of [`GenericOrchestrator`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
        }
    }

    /// Adds a pipeline to the [`GenericOrchestrator`] which needs to have same the input and output as the [`GenericOrchestrator`].
    /// It also needs to have an error type that implements `Into<OrchestratorErrorType>`.
    pub fn add_pipeline<PipelineType>(&mut self, pipeline: PipelineType)
    where
        PipelineType: Pipeline + Debug,
        Input: Into<PipelineType::Input>,
        PipelineType::Output: Into<InternalPipelineOutput<Output>>,
        PipelineType::Error: Into<Error>,
    {
        self.pipelines
            .push(Box::new(InternalPipelineStruct::new(pipeline)));
    }
}

impl<Input, Output, Error> Default for GenericOrchestrator<Input, Output, Error>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    OrchestratorError: Into<Error>,
{
    fn default() -> Self {
        Self::new()
    }
}
