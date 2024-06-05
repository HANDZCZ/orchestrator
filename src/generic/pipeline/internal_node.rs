use std::{
    any::{self, Any, TypeId},
    fmt::Debug,
};

use async_trait::async_trait;

use crate::generic::node::{Node, NodeOutput};

#[derive(Debug)]
pub enum InternalNodeOutput {
    NodeOutput(NodeOutput<Box<dyn Any + Send + Sync>>),
    WrongInputType,
}

#[async_trait]
pub trait InternalNode<Error>: Debug + Send + Sync {
    async fn run(&mut self, input: Box<dyn Any + Send + Sync>)
        -> Result<InternalNodeOutput, Error>;
    fn duplicate(&self) -> Box<dyn InternalNode<Error>>;
    fn get_node_type(&self) -> TypeId;
    fn get_node_type_name(&self) -> &'static str;
}

#[derive(Debug)]
pub struct InternalNodeStruct<NodeType: Node> {
    node: NodeType,
}

impl<NodeType: Node> InternalNodeStruct<NodeType> {
    pub fn new(node: NodeType) -> Self {
        Self { node }
    }
}

#[async_trait]
impl<NodeType: Node<Error = NodeError> + Debug, Error, NodeError: Into<Error>> InternalNode<Error>
    for InternalNodeStruct<NodeType>
{
    async fn run(
        &mut self,
        input: Box<dyn Any + Send + Sync>,
    ) -> Result<InternalNodeOutput, Error> {
        let Ok(input) = input.downcast::<NodeType::Input>() else {
            return Ok(InternalNodeOutput::WrongInputType);
        };
        let output = self.node.run(*input).await.map_err(|e| e.into())?;
        Ok(InternalNodeOutput::NodeOutput(match output {
            NodeOutput::PipeToNode(next_node) => NodeOutput::PipeToNode(next_node),
            NodeOutput::SoftFail => NodeOutput::SoftFail,
            NodeOutput::ReturnFromPipeline(output) => {
                NodeOutput::ReturnFromPipeline(Box::new(output))
            }
            NodeOutput::Advance(output) => NodeOutput::Advance(Box::new(output)),
        }))
    }
    fn duplicate(&self) -> Box<dyn InternalNode<Error>> {
        Box::new(Self {
            node: self.node.clone(),
        })
    }
    fn get_node_type(&self) -> TypeId {
        TypeId::of::<NodeType>()
    }
    fn get_node_type_name(&self) -> &'static str {
        any::type_name::<NodeType>()
    }
}
