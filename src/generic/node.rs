use std::any::{self, Any, TypeId};

use async_trait::async_trait;

/// Trait that has to be implemented for adding a node to [`GenericPipeline`](crate::generic::pipeline::GenericPipeline).
///
/// Example how can [`Node`] trait be implemented.
/// ```no_run
/// use orchestrator::{async_trait, generic::node::{Node, NodeOutput, Returnable}};
///
/// #[derive(Clone)]
/// struct StringLen;
/// struct StringIsEmpty;
///
/// #[async_trait]
/// impl Node for StringLen {
///    type Input = String;
///    type Output = usize;
///    type Error = StringIsEmpty;
///
///    async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
///        // here should be some io bound operation...
///        if input.is_empty() {
///            return Err(StringIsEmpty);
///        }
///        Self::advance(input.len()).into()
///    }
/// }
/// ```
#[cfg_attr(not(docs_cfg), async_trait)]
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

/// Part of an output that object implementing [`Node`] trait must return.
///
/// This type is supposed to be constructed through [`Returnable`] trait.
#[derive(Debug)]
#[must_use]
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

/// Designates the type for [`NodeOutput::PipeToNode`].
#[derive(Debug)]
pub struct NextNode {
    pub(crate) output: Box<dyn Any + Send + Sync>,
    pub(crate) next_node_type: TypeId,
    pub(crate) next_node_type_name: &'static str,
}

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
    fn return_from_pipeline(output: Self::Output) -> NodeOutput<Self::Output> {
        NodeOutput::ReturnFromPipeline(output)
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
