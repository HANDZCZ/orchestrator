use async_trait::async_trait;

use crate::generic::pipeline::PipelineStorage;

use super::NodeOutput;

/// Trait that has to be implemented for adding a node to [`GenericPipeline`](crate::generic::pipeline::GenericPipeline).
///
/// Example how can [`Node`] trait be implemented.
/// ```no_run
/// use orchestrator::{async_trait, generic::{pipeline::PipelineStorage, node::{Node, NodeOutput, Returnable}}};
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
///    async fn run(&mut self, input: Self::Input, pipeline_storage: &mut PipelineStorage) -> Result<NodeOutput<Self::Output>, Self::Error> {
///        // here should be some io bound operation...
///        if input.is_empty() {
///            return Err(StringIsEmpty);
///        }
///        Self::advance(input.len()).into()
///    }
/// }
/// ```
#[cfg_attr(not(all(doc, not(doctest))), async_trait)]
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
    async fn run(
        &mut self,
        input: Self::Input,
        pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error>;
}
