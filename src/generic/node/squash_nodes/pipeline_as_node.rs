use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;

use crate::{
    generic::{
        node::{Node, NodeOutput, Returnable},
        pipeline::PipelineStorage,
    },
    pipeline::{DebuggablePipeline, Pipeline},
};

/// Type that wraps around some [`Pipeline`] to crate a [`Node`] from it.
///
/// Example that shows usage of [`PipelineAsNode`].
/// ```no_run
/// use orchestrator::{
///     async_trait,
///     pipeline::Pipeline,
///     generic::{
///         node::squash_nodes::PipelineAsNodeExt,
///         pipeline::PipelineOutput
///     }
/// };
///
/// #[derive(Debug)]
/// struct MyPipeline;
///
/// #[async_trait]
/// impl Pipeline for MyPipeline {
///     type Input = String;
///     type Output = String;
///     type Error = ();
///
///     async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
///         // some io bound operation
///         // let input = {...}.await;
///         Ok(input)
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     // construct pipeline
///     let pipeline = MyPipeline;
///     // convert it to node
///     let pipeline_as_node = pipeline.into_node();
///     // do something with the node
///     // ...
/// }
/// ```
#[derive(Debug)]
pub struct PipelineAsNode<Input, Output, Error> {
    #[cfg(docs_cfg)]
    _types: std::marker::PhantomData<(Input, Output, Error)>,
    #[cfg(not(docs_cfg))]
    pipeline: Arc<dyn DebuggablePipeline<Input = Input, Output = Output, Error = Error>>,
}
impl<Input, Output, Error> Clone for PipelineAsNode<Input, Output, Error> {
    fn clone(&self) -> Self {
        Self {
            pipeline: self.pipeline.clone(),
        }
    }
}
impl<Input, Output, Error> PipelineAsNode<Input, Output, Error> {
    /// Creates new instance of [`PipelineAsNode`] from type that implements [`Pipeline`] trait.
    pub fn new<PipelineType>(pipeline: PipelineType) -> Self
    where
        PipelineType: Pipeline<Input = Input, Output = Output, Error = Error> + Debug + 'static,
    {
        Self {
            pipeline: Arc::new(pipeline),
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

/// Extension for [`Pipeline`] trait that converts pipeline into [`Node`].
pub trait PipelineAsNodeExt: Pipeline + Debug {
    /// Converts [`Pipeline`] into [`Node`].
    fn into_node(self) -> PipelineAsNode<Self::Input, Self::Output, Self::Error>
    where
        Self: Sized,
    {
        PipelineAsNode::new(self)
    }
}
impl<T> PipelineAsNodeExt for T where T: Pipeline + Debug {}
