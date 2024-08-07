use std::{
    any::{Any, TypeId},
    fmt::Debug,
    marker::PhantomData,
};

use async_trait::async_trait;

use crate::generic::node::{Node, NodeOutput};

use super::PipelineStorage;

#[derive(Debug)]
pub enum InternalNodeOutput {
    NodeOutput(NodeOutput<Box<dyn Any + Send + Sync>>),
    WrongInputType,
}

#[async_trait]
pub trait InternalNode<Error>: Debug + Send + Sync {
    async fn run(
        &mut self,
        input: Box<dyn Any + Send + Sync>,
        piped: bool,
        pipeline_storage: &mut PipelineStorage,
    ) -> Result<InternalNodeOutput, Error>;
    fn duplicate(&self) -> Box<dyn InternalNode<Error>>;
    fn get_node_type(&self) -> TypeId;
    #[cfg(feature = "pipeline_early_return")]
    fn get_node_type_name(&self) -> &'static str;
}

pub struct InternalNodeStruct<NodeType: Node, PreviousNodeOutputType> {
    _previous_node_output_type: PhantomData<PreviousNodeOutputType>,
    node: NodeType,
}

impl<T: Node + Debug, U> Debug for InternalNodeStruct<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.node.fmt(f)
    }
}

impl<NodeType, PreviousNodeOutputType> InternalNodeStruct<NodeType, PreviousNodeOutputType>
where
    NodeType: Node,
    PreviousNodeOutputType: Into<NodeType::Input> + 'static,
{
    pub fn new(node: NodeType) -> Self {
        Self {
            _previous_node_output_type: PhantomData,
            node,
        }
    }

    fn get_input(input: Box<dyn Any + Send + Sync>, piped: bool) -> Option<NodeType::Input> {
        let res = if piped {
            *input.downcast::<NodeType::Input>().ok()?
        } else {
            let input = input.downcast::<PreviousNodeOutputType>().ok()?;
            (*input).into()
        };
        Some(res)
    }
}

#[async_trait]
impl<NodeType, Error, PreviousNodeOutputType> InternalNode<Error>
    for InternalNodeStruct<NodeType, PreviousNodeOutputType>
where
    NodeType: Node + Debug,
    NodeType::Error: Into<Error>,
    PreviousNodeOutputType: Send + Sync + 'static + Into<NodeType::Input>,
{
    async fn run(
        &mut self,
        input: Box<dyn Any + Send + Sync>,
        piped: bool,
        pipeline_storage: &mut PipelineStorage,
    ) -> Result<InternalNodeOutput, Error> {
        let Some(input) = Self::get_input(input, piped) else {
            return Ok(InternalNodeOutput::WrongInputType);
        };
        let output = self
            .node
            .run(input, pipeline_storage)
            .await
            .map_err(Into::into)?;
        Ok(InternalNodeOutput::NodeOutput(match output {
            NodeOutput::PipeToNode(next_node) => NodeOutput::PipeToNode(next_node),
            NodeOutput::SoftFail => NodeOutput::SoftFail,
            #[cfg(feature = "pipeline_early_return")]
            NodeOutput::ReturnFromPipeline(output) => NodeOutput::ReturnFromPipeline(output),
            NodeOutput::Advance(output) => NodeOutput::Advance(Box::new(output)),
        }))
    }
    fn duplicate(&self) -> Box<dyn InternalNode<Error>> {
        Box::new(Self {
            _previous_node_output_type: PhantomData,
            node: self.node.clone(),
        })
    }
    fn get_node_type(&self) -> TypeId {
        TypeId::of::<NodeType>()
    }
    #[cfg(feature = "pipeline_early_return")]
    fn get_node_type_name(&self) -> &'static str {
        std::any::type_name::<NodeType>()
    }
}
