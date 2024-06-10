use std::{
    any::{self, Any, TypeId},
    fmt::Debug,
};

use super::{NextNode, Node, NodeOutput};

/// Defines bunch of helper functions for [`Node`] that return [`NodeOutput`].
pub trait Returnable: Node {
    /// Creates [`NodeOutput`] that pipes data to [`Node`] with the `NextNodeType` type.
    ///
    /// Can be used for:
    /// - saying which [`Node`] should be run next
    /// - jumping to a [`Node`] in [`GenericPipeline`](crate::generic::pipeline::GenericPipeline)
    ///
    /// ```no_run
    /// # use orchestrator::{async_trait, generic::node::{Node, NodeOutput, Returnable}};
    /// #
    /// # #[derive(Clone)]
    /// # struct NodeToPipeInto;
    /// #
    /// # #[async_trait]
    /// # impl Node for NodeToPipeInto {
    /// #    type Input = &'static str;
    /// #    type Output = ();
    /// #    type Error = ();
    /// #
    /// #    async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
    /// #        unimplemented!()
    /// #    }
    /// # }
    /// #
    /// # #[derive(Clone)]
    /// # struct SomeNode;
    /// #
    /// # #[async_trait]
    /// # impl Node for SomeNode {
    /// #    type Input = String;
    /// #    type Output = usize;
    /// #    type Error = ();
    /// #
    /// // run method in Node trait
    /// // Self is in this example implementing Node trait
    /// async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
    ///     // pipes data from node where it's called to node with concrete type NodeToPipeInto
    ///     Self::pipe_to::<NodeToPipeInto>("some data").into()
    /// }
    /// # }
    /// ```
    fn pipe_to<NextNodeType: Node>(data: NextNodeType::Input) -> NodeOutput<Self::Output> {
        NodeOutput::PipeToNode(NextNode {
            output: Box::new(data),
            next_node_type: TypeId::of::<NextNodeType>(),
            next_node_type_name: any::type_name::<NextNodeType>(),
        })
    }

    /// Creates [`NodeOutput`] that returns from a [`Pipeline`](crate::pipeline::Pipeline) early.
    ///
    /// ```no_run
    /// # use orchestrator::{async_trait, generic::node::{Node, NodeOutput, Returnable}};
    /// #
    /// # #[derive(Clone)]
    /// # struct SomeNode;
    /// #
    /// # #[async_trait]
    /// # impl Node for SomeNode {
    /// #    type Input = String;
    /// #    type Output = &'static str;
    /// #    type Error = ();
    /// #
    /// // run method in Node trait
    /// // Self is in this example implementing Node trait
    /// async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
    ///     // returns data from pipeline early
    ///     Self::return_from_pipeline("some data").into()
    /// }
    /// # }
    /// ```
    fn return_from_pipeline<Output>(output: Output) -> NodeOutput<Self::Output>
    where
        Output: Any + Sync + Send + Debug,
    {
        NodeOutput::ReturnFromPipeline(Box::new(output))
    }

    /// Creates [`NodeOutput`] that advances a [`Pipeline`](crate::pipeline::Pipeline).
    ///
    /// ```no_run
    /// # use orchestrator::{async_trait, generic::node::{Node, NodeOutput, Returnable}};
    /// #
    /// # #[derive(Clone)]
    /// # struct SomeNode;
    /// #
    /// # #[async_trait]
    /// # impl Node for SomeNode {
    /// #    type Input = String;
    /// #    type Output = &'static str;
    /// #    type Error = ();
    /// #
    /// // run method in Node trait
    /// // Self is in this example implementing Node trait
    /// async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
    ///     // advances pipeline to the next node
    ///     Self::advance("some data").into()
    /// }
    /// # }
    /// ```
    fn advance(output: Self::Output) -> NodeOutput<Self::Output> {
        NodeOutput::Advance(output)
    }

    /// Creates [`NodeOutput`] that soft fails a [`Pipeline`](crate::pipeline::Pipeline).
    ///
    /// ```no_run
    /// # use orchestrator::{async_trait, generic::node::{Node, NodeOutput, Returnable}};
    /// #
    /// # #[derive(Clone)]
    /// # struct SomeNode;
    /// #
    /// # #[async_trait]
    /// # impl Node for SomeNode {
    /// #    type Input = String;
    /// #    type Output = &'static str;
    /// #    type Error = ();
    /// #
    /// // run method in Node trait
    /// // Self is in this example implementing Node trait
    /// async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
    ///     // causes the pipeline to soft fail
    ///     Self::soft_fail().into()
    /// }
    /// # }
    /// ```
    ///
    /// For more information look at [`NodeOutput::SoftFail`].
    fn soft_fail() -> NodeOutput<Self::Output> {
        NodeOutput::SoftFail
    }
}

impl<NodeType: Node> Returnable for NodeType {}