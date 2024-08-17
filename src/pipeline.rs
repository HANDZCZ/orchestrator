use std::fmt::Debug;

use async_trait::async_trait;

/// Defines what methods should be implemented for a pipeline.
/// It also defines what is the input, output and error should a pipeline have.
///
/// Example how can [`Pipeline`] trait be implemented.
/// In this example no nodes are used.
/// ```no_run
/// use orchestrator::{async_trait, pipeline::Pipeline, generic::pipeline::PipelineOutput};
///
/// struct MyPipeline;
///
/// #[async_trait]
/// impl Pipeline for MyPipeline {
///     type Input = String;
///     type Output = PipelineOutput<String>;
///     type Error = ();
///
///     async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
///         // some io bound operation
///         // let input = {...}.await;
///         Ok(PipelineOutput::Done(input))
///     }
/// }
/// ```
#[cfg_attr(not(docs_cfg), async_trait)]
pub trait Pipeline: Sync + Send + 'static
where
    Self::Input: Send + Sync + 'static,
    Self::Output: Send + Sync + 'static,
    Self::Error: Send + Sync + 'static,
{
    /// Pipeline input type.
    type Input;
    /// Pipeline output type.
    type Output;
    /// Pipeline error type.
    type Error;

    /// Runs the pipeline.
    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

pub(crate) trait DebuggablePipeline: Pipeline + Debug {}
impl<T> DebuggablePipeline for T where T: Pipeline + Debug {}
