use std::fmt::Debug;

use async_trait::async_trait;

use crate::pipeline::Pipeline;

#[async_trait]
pub trait InternalPipeline<Input, Output, Error>: Debug + Send + Sync {
    async fn run(&self, input: Input) -> Result<Output, Error>;
}

#[derive(Debug)]
pub struct InternalPipelineStruct<PipelineType: Pipeline> {
    pipeline: PipelineType,
}
impl<PipelineType: Pipeline> InternalPipelineStruct<PipelineType> {
    pub fn new(pipeline: PipelineType) -> Self {
        Self { pipeline }
    }
}

#[async_trait]
impl<
        Input: Send + Sync + 'static,
        Output: Send + Sync + 'static,
        Error: Send + Sync + 'static,
        PipelineType: Pipeline<Input = Input, Output = Output, Error = PipelineError> + Debug,
        PipelineError: Into<Error>,
    > InternalPipeline<Input, Output, Error> for InternalPipelineStruct<PipelineType>
{
    async fn run(&self, input: Input) -> Result<Output, Error> {
        self.pipeline.run(input).await.map_err(|e| e.into())
    }
}
