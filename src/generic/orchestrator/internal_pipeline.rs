use std::fmt::Debug;

use async_trait::async_trait;

use crate::{generic::pipeline::PipelineOutput, pipeline::Pipeline};

#[async_trait]
pub trait InternalPipeline<Input, Output, Error>: Debug + Send + Sync {
    async fn run(&self, input: Input) -> Result<Output, Error>;
}

pub struct InternalPipelineStruct<PipelineType: Pipeline> {
    pipeline: PipelineType,
}

impl<T: Pipeline + Debug> Debug for InternalPipelineStruct<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.pipeline.fmt(f)
    }
}

impl<PipelineType: Pipeline> InternalPipelineStruct<PipelineType> {
    pub fn new(pipeline: PipelineType) -> Self {
        Self { pipeline }
    }
}

#[async_trait]
impl<Input, Output, Error, PipelineType> InternalPipeline<Input, Output, Error>
    for InternalPipelineStruct<PipelineType>
where
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    PipelineType: Pipeline + Debug,
    Input: Into<PipelineType::Input>,
    PipelineType::Output: Into<Output>,
    PipelineType::Error: Into<Error>,
{
    async fn run(&self, input: Input) -> Result<Output, Error> {
        self.pipeline
            .run(input.into())
            .await
            .map(Into::into)
            .map_err(Into::into)
    }
}

// Needed because specialization is not stable
//
// The following code gives conflicting implementations for impl<T> From<T> for T { ... } error
// impl<T, U> From<PipelineOutput<U>> for PipelineOutput<T>
// where
//     U: Into<T>,
// { ... }
#[derive(Debug)]
pub enum InternalPipelineOutput<T> {
    SoftFail,
    Done(T),
}

impl<T, U> From<PipelineOutput<U>> for InternalPipelineOutput<T>
where
    U: Into<T>,
{
    fn from(value: PipelineOutput<U>) -> Self {
        match value {
            PipelineOutput::SoftFail => InternalPipelineOutput::SoftFail,
            PipelineOutput::Done(val) => InternalPipelineOutput::Done(val.into()),
        }
    }
}

impl<T, U> From<U> for InternalPipelineOutput<T>
where
    U: Into<PipelineOutput<U>>,
    U: Into<T>,
{
    fn from(value: U) -> Self {
        // ¯\_(ツ)_/¯
        // U -> PipelineOutput<U> -> InternalPipelineOutput<T>
        let pipeline_output: PipelineOutput<U> = value.into();
        pipeline_output.into()
    }
}
