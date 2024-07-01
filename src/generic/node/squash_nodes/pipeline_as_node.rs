use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;

use crate::{
    generic::{
        node::{Node, NodeOutput, Returnable},
        pipeline::PipelineStorage,
    },
    pipeline::Pipeline,
};

trait DebuggablePipeline: Pipeline + Debug {}
impl<T> DebuggablePipeline for T where T: Pipeline + Debug {}

#[derive(Debug)]
pub struct PipelineAsNode<Input, Output, Error> {
    #[cfg(docs_cfg)]
    _types: std::marker::PhantomData<(Input, Output, Error)>,
    #[cfg(not(docs_cfg))]
    pipeline: Arc<Box<dyn DebuggablePipeline<Input = Input, Output = Output, Error = Error>>>,
}
impl<Input, Output, Error> Clone for PipelineAsNode<Input, Output, Error> {
    fn clone(&self) -> Self {
        Self {
            pipeline: self.pipeline.clone(),
        }
    }
}
impl<Input, Output, Error> PipelineAsNode<Input, Output, Error> {
    pub fn new<PipelineType>(pipeline: PipelineType) -> Self
    where
        PipelineType: Pipeline<Input = Input, Output = Output, Error = Error> + Debug + 'static,
    {
        Self {
            pipeline: Arc::new(Box::new(pipeline)),
        }
    }
}

#[cfg_attr(not(docs_cfg), async_trait)]
impl<Input, Output, Error> Node for PipelineAsNode<Input, Output, Error>
where
    Input: Clone + Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    type Input = Input;
    type Output = Output;
    type Error = Error;

    async fn run(
        &mut self,
        input: Self::Input,
        _pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error> {
        Self::advance(self.pipeline.run(input).await?).into()
    }
}

pub trait PipelineAsNodeExt: Pipeline + Debug {
    fn into_node(self) -> PipelineAsNode<Self::Input, Self::Output, Self::Error>
    where
        Self: Sized,
    {
        PipelineAsNode::new(self)
    }
}
impl<T> PipelineAsNodeExt for T where T: Pipeline + Debug {}
