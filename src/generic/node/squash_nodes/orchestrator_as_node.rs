use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;

use crate::{
    generic::{
        node::{Node, NodeOutput, Returnable},
        pipeline::PipelineStorage,
    },
    orchestrator::Orchestrator,
};

trait DebuggableOrchestrator: Orchestrator + Debug {}
impl<T> DebuggableOrchestrator for T where T: Orchestrator + Debug {}

#[derive(Debug)]
pub struct OrchestratorAsNode<Input, Output, Error> {
    #[cfg(docs_cfg)]
    _types: std::marker::PhantomData<(Input, Output, Error)>,
    #[cfg(not(docs_cfg))]
    orchestrator:
        Arc<Box<dyn DebuggableOrchestrator<Input = Input, Output = Output, Error = Error>>>,
}
impl<Input, Output, Error> Clone for OrchestratorAsNode<Input, Output, Error> {
    fn clone(&self) -> Self {
        Self {
            orchestrator: self.orchestrator.clone(),
        }
    }
}
impl<Input, Output, Error> OrchestratorAsNode<Input, Output, Error> {
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
impl<Input, Output, Error> Node for OrchestratorAsNode<Input, Output, Error>
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
        Self::advance(self.orchestrator.run(input).await?).into()
    }
}

pub trait OrchestratorAsNodeExt: Orchestrator + Debug {
    fn into_node(self) -> OrchestratorAsNode<Self::Input, Self::Output, Self::Error>
    where
        Self: Sized,
    {
        OrchestratorAsNode::new(self)
    }
}
impl<T> OrchestratorAsNodeExt for T where T: Orchestrator + Debug {}
