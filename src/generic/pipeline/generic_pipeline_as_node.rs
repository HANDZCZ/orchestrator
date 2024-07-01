use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    generic::{
        node::{Node, NodeOutput},
        pipeline::{GenericPipelineBuilt, PipelineError, PipelineOutput, PipelineStorage},
    },
    pipeline::Pipeline,
};

#[derive(Debug)]
pub struct GenericPipelineAsNode<Input, Output, Error> {
    pipeline: Arc<GenericPipelineBuilt<Input, Output, Error>>,
    share_pipeline_storage: bool,
}

impl<Input, Output, Error> GenericPipelineAsNode<Input, Output, Error> {
    #[must_use]
    pub fn new(pipeline: GenericPipelineBuilt<Input, Output, Error>) -> Self {
        Self {
            pipeline: Arc::new(pipeline),
            share_pipeline_storage: false,
        }
    }

    #[must_use]
    pub fn share_pipeline_storage(mut self) -> Self {
        self.share_pipeline_storage = true;
        self
    }
}
impl<Input, Output, Error> From<GenericPipelineBuilt<Input, Output, Error>>
    for GenericPipelineAsNode<Input, Output, Error>
{
    fn from(value: GenericPipelineBuilt<Input, Output, Error>) -> Self {
        Self::new(value)
    }
}

impl<Input, Output, Error> GenericPipelineBuilt<Input, Output, Error> {
    #[must_use]
    pub fn into_node(self) -> GenericPipelineAsNode<Input, Output, Error> {
        GenericPipelineAsNode::new(self)
    }
}
impl<Input, Output, Error> Clone for GenericPipelineAsNode<Input, Output, Error> {
    fn clone(&self) -> Self {
        Self {
            pipeline: self.pipeline.clone(),
            share_pipeline_storage: self.share_pipeline_storage,
        }
    }
}

#[cfg_attr(not(docs_cfg), async_trait)]
impl<Input, Output, Error> Node for GenericPipelineAsNode<Input, Output, Error>
where
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    PipelineError: Into<Error>,
{
    type Input = Input;
    type Output = Output;
    type Error = Error;

    async fn run(
        &mut self,
        input: Self::Input,
        pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error> {
        if self.share_pipeline_storage {
            return Ok(self
                .pipeline
                .run_with_pipeline_storage(input, pipeline_storage)
                .await?
                .into());
        }
        Ok(self.pipeline.run(input).await?.into())
    }
}

impl<T> From<PipelineOutput<T>> for NodeOutput<T> {
    fn from(value: PipelineOutput<T>) -> Self {
        match value {
            PipelineOutput::SoftFail => NodeOutput::SoftFail,
            PipelineOutput::Done(data) => NodeOutput::Advance(data),
        }
    }
}
