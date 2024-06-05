use std::any::{self, Any, TypeId};

use async_trait::async_trait;

#[async_trait]
pub trait Node: Send + Sync + Clone + 'static
where
    Self::Input: Send + Sync + 'static,
    Self::Output: Send + Sync + 'static,
{
    type Input;
    type Output;
    type Error;

    async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error>;
}

#[derive(Debug)]
pub enum NodeOutput<T> {
    SoftFail,
    ReturnFromPipeline(T),
    Advance(T),
    PipeToNode(NextNode),
}

impl<T, E> From<NodeOutput<T>> for Result<NodeOutput<T>, E> {
    fn from(value: NodeOutput<T>) -> Self {
        Ok(value)
    }
}

#[derive(Debug)]
pub struct NextNode {
    pub(crate) output: Box<dyn Any + Send + Sync>,
    pub(crate) next_node_type: TypeId,
    pub(crate) next_node_type_name: &'static str,
}

pub trait Returnable<NodeType: Node> {
    fn pipe_to<NextNodeType: Node<Input = NodeType::Output>>(
        output: NodeType::Output,
    ) -> NodeOutput<NodeType::Output> {
        NodeOutput::PipeToNode(NextNode {
            output: Box::new(output),
            next_node_type: TypeId::of::<NextNodeType>(),
            next_node_type_name: any::type_name::<NextNodeType>(),
        })
    }

    fn return_from_pipeline(output: NodeType::Output) -> NodeOutput<NodeType::Output> {
        NodeOutput::ReturnFromPipeline(output)
    }

    fn advance(output: NodeType::Output) -> NodeOutput<NodeType::Output> {
        NodeOutput::Advance(output)
    }

    fn soft_fail() -> NodeOutput<NodeType::Output> {
        NodeOutput::SoftFail
    }
}

impl<NodeType: Node> Returnable<NodeType> for NodeType {}
