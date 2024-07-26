use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

/// Part of an output that object implementing [`Node`](crate::generic::node::Node) trait must return.
///
/// This type is supposed to be constructed through [`Returnable`](crate::generic::node::Returnable) trait.
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
    #[cfg(feature = "pipeline_early_return")]
    ReturnFromPipeline(ReturnFromPipelineOutput),
    /// Advances [`Pipeline`](crate::pipeline::Pipeline) to the next [`Node`](crate::generic::node::Node).
    Advance(T),
    /// Pipes the output from the current [`Node`](crate::generic::node::Node) to a [`Node`](crate::generic::node::Node) with defined type.
    ///
    /// For more information look at [`Returnable::pipe_to`](crate::generic::node::Returnable::pipe_to).
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

/// Designates the output for [`NodeOutput::ReturnFromPipeline`].
#[derive(Debug)]
#[cfg(feature = "pipeline_early_return")]
pub struct ReturnFromPipelineOutput(pub(crate) Box<dyn crate::generic::SuperAnyDebug>);
