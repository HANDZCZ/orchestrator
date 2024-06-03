use std::{
    any::{self, Any, TypeId},
    fmt::Debug,
};

#[derive(Debug)]
pub enum NodeOutput<T> {
    SoftFail,
    ReturnFromPipeline(T),
    Advance(T),
    SuccessAndPipeOutput(NextNode),
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

pub trait Node: Debug + Send + Sync + Clone + 'static
where
    Self::Input: Send + Sync + 'static,
    Self::Output: Send + Sync + 'static,
{
    type Input;
    type Output;
    type Error;

    fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error>;
}

pub trait Returnable<NodeType: Node> {
    fn pipe_to<NodeTypeNext: Node<Input = NodeType::Output>>(
        output: NodeType::Output,
    ) -> NodeOutput<NodeType::Output> {
        NodeOutput::SuccessAndPipeOutput(NextNode {
            output: Box::new(output),
            next_node_type: TypeId::of::<NodeTypeNext>(),
            next_node_type_name: any::type_name::<NodeTypeNext>(),
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