use std::any::{self, Any, TypeId};

use async_trait::async_trait;

/// Trait that has to be implemented for adding a node to a [`GenericPipeline`](crate::generic::pipeline::GenericPipeline).
#[async_trait]
pub trait Node: Send + Sync + Clone + 'static
where
    Self::Input: Send + Sync + 'static,
    Self::Output: Send + Sync + 'static,
{
    /// Node input type.
    type Input;
    /// Node output type.
    type Output;
    /// Node error type.
    type Error;

    /// Defines what the node is supposed to be doing.
    async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error>;
}

/// Part of an output that object implementing [`Node`] trait must return
#[derive(Debug)]
pub enum NodeOutput<T> {
    /// Fails the pipeline without returning error.
    ///
    /// For example it can be used when we have multiple pipelines with the same input and output, but every pipeline has different implementation.
    /// As a concrete example image you have a pipeline consisting of Matcher, Downloader and Parser.
    /// - Matcher - checks the input using some rules and either decides if this input can be processed by this pipeline or not
    /// - Downloader - downloads the resource
    /// - Parser - parses the resource
    ///
    /// In this example Matcher can soft fail.
    SoftFail,
    /// Returns from [`Pipeline`](crate::pipeline::Pipeline) early.
    ReturnFromPipeline(T),
    /// Advances [`Pipeline`](crate::pipeline::Pipeline) to the next [`Node`].
    Advance(T),
    /// Pipes the output from the current [`Node`] to a [`Node`] with defined type.
    ///
    /// For more information look at [`Returnable::pipe_to`].
    PipeToNode(NextNode),
}

impl<T, E> From<NodeOutput<T>> for Result<NodeOutput<T>, E> {
    fn from(value: NodeOutput<T>) -> Self {
        Ok(value)
    }
}

/// Designates the type for [`NodeOutput::PipeToNode`]
#[derive(Debug)]
pub struct NextNode {
    pub(crate) output: Box<dyn Any + Send + Sync>,
    pub(crate) next_node_type: TypeId,
    pub(crate) next_node_type_name: &'static str,
}

/// Defines bunch of helper functions for [`Node`] that return [`NodeOutput`].
pub trait Returnable<NodeType: Node> {
    /// Creates [`NodeOutput`] that pipes the output to [`Node`] with type `NextNodeType`.
    ///
    /// Can be used for:
    /// - saying which [`Node`] should be run next
    /// - jumping to a [`Node`] in a [`Pipeline`](crate::pipeline::Pipeline)
    fn pipe_to<NextNodeType: Node<Input = NodeType::Output>>(
        output: NodeType::Output,
    ) -> NodeOutput<NodeType::Output> {
        NodeOutput::PipeToNode(NextNode {
            output: Box::new(output),
            next_node_type: TypeId::of::<NextNodeType>(),
            next_node_type_name: any::type_name::<NextNodeType>(),
        })
    }

    /// Creates [`NodeOutput`] that returns from a [`Pipeline`](crate::pipeline::Pipeline) early.
    fn return_from_pipeline(output: NodeType::Output) -> NodeOutput<NodeType::Output> {
        NodeOutput::ReturnFromPipeline(output)
    }

    /// Creates [`NodeOutput`] that advances a [`Pipeline`](crate::pipeline::Pipeline).
    fn advance(output: NodeType::Output) -> NodeOutput<NodeType::Output> {
        NodeOutput::Advance(output)
    }

    /// Creates [`NodeOutput`] that soft fails a [`Pipeline`](crate::pipeline::Pipeline).
    ///
    /// For more information look at [`NodeOutput::SoftFail`].
    fn soft_fail() -> NodeOutput<NodeType::Output> {
        NodeOutput::SoftFail
    }
}

impl<NodeType: Node> Returnable<NodeType> for NodeType {}
