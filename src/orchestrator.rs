use crate::pipeline::{Pipeline, PipelineError, PipelineOutput, PipelineStateBuilt};

pub trait OrchestratorError {
    fn no_matching_pipeline() -> Self;
}

type OrchestratorPipeline<Input, Output, Error> =
    Pipeline<Input, Output, Error, PipelineStateBuilt>;

pub trait Orchestrator
where
    Self::Input: Send + Sync + Clone + 'static,
    Self::Output: Send + Sync + 'static,
    Self::Error: OrchestratorError + PipelineError,
{
    type Input;
    type Output;
    type Error;

    fn get_pipelines(&self) -> &[OrchestratorPipeline<Self::Input, Self::Output, Self::Error>];
    fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        for pipeline in self.get_pipelines() {
            match pipeline.run(input.clone())? {
                PipelineOutput::SoftFail => continue,
                PipelineOutput::Done(output) => return Ok(output),
            }
        }
        Err(Self::Error::no_matching_pipeline())
    }
}

#[derive(Debug)]
pub struct GeneralOrchestrator<
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: OrchestratorError + PipelineError,
> {
    pipelines: Vec<Pipeline<Input, Output, Error, PipelineStateBuilt>>,
}

impl<
        Input: Send + Sync + Clone + 'static,
        Output: Send + Sync + 'static,
        Error: OrchestratorError + PipelineError,
    > Orchestrator for GeneralOrchestrator<Input, Output, Error>
{
    type Input = Input;
    type Output = Output;
    type Error = Error;

    fn get_pipelines(
        &self,
    ) -> &[Pipeline<Self::Input, Self::Output, Self::Error, PipelineStateBuilt>] {
        &self.pipelines
    }
}

impl<
        Input: Send + Sync + Clone + 'static,
        Output: Send + Sync + 'static,
        Error: OrchestratorError + PipelineError,
    > GeneralOrchestrator<Input, Output, Error>
{
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
        }
    }
    pub fn add_pipeline(&mut self, pipeline: Pipeline<Input, Output, Error, PipelineStateBuilt>) {
        self.pipelines.push(pipeline);
    }
}

impl<
        Input: Send + Sync + Clone + 'static,
        Output: Send + Sync + 'static,
        Error: OrchestratorError + PipelineError,
    > Default for GeneralOrchestrator<Input, Output, Error>
{
    fn default() -> Self {
        Self::new()
    }
}
