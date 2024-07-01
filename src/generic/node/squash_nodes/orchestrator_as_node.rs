use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;

use crate::{
    generic::{
        node::{Node, NodeOutput, Returnable},
        pipeline::PipelineStorage,
    },
    orchestrator::Orchestrator,
};

trait DebuggableOrchestrator: Orchestrator + Debug {}
impl<T> DebuggableOrchestrator for T where T: Orchestrator + Debug {}

/// Type that wraps around some [`Orchestrator`] to crate a [`Node`] from it.
///
/// Example that shows usage of [`OrchestratorAsNode`].
/// ```no_run
/// use orchestrator::{
///     async_trait,
///     pipeline::Pipeline,
///     generic::{
///         node::squash_nodes::OrchestratorAsNodeExt,
///         pipeline::PipelineOutput
///     },
///     orchestrator::Orchestrator
/// };
/// use std::fmt::Debug;
///
/// enum MyOrchestratorError {
///     AllPipelinesSoftFailed,
///     PipelineError,
/// }
///
/// struct MyOrchestrator {
///     pipelines: Vec<Box<dyn Pipeline<Input = String, Output = String, Error = ()>>>,
/// }
/// impl Debug for MyOrchestrator //...
/// # {
/// #    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// #        f.debug_struct("MyOrchestrator").finish_non_exhaustive()
/// #    }
/// # }
///
/// #[async_trait]
/// impl Orchestrator for MyOrchestrator {
///     type Input = String;
///     type Output = String;
///     type Error = MyOrchestratorError;
///
///     async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
///         let mut res = String::new();
///         for pipeline in &self.pipelines {
///             res += &pipeline.run(input.clone())
///                 .await
///                 .map_err(|e| MyOrchestratorError::PipelineError)?;
///         }
///         Ok(res)
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     // construct orchestrator with some pipelines
///     let orchestrator = MyOrchestrator {
///         // ...
/// #         pipelines: Vec::new()
///     };
///     // convert it to node
///     let orchestrator_as_node = orchestrator.into_node();
///     // do something with the node
///     // ...
/// }
/// ```
#[derive(Debug)]
pub struct OrchestratorAsNode<Input, Output, Error> {
    #[cfg(docs_cfg)]
    _types: std::marker::PhantomData<(Input, Output, Error)>,
    #[cfg(not(docs_cfg))]
    orchestrator:
        Arc<Box<dyn DebuggableOrchestrator<Input = Input, Output = Output, Error = Error>>>,
}
impl<Input, Output, Error> Clone for OrchestratorAsNode<Input, Output, Error> {
    fn clone(&self) -> Self {
        Self {
            orchestrator: self.orchestrator.clone(),
        }
    }
}
impl<Input, Output, Error> OrchestratorAsNode<Input, Output, Error> {
    /// Creates new instance of [`OrchestratorAsNode`] from type that implements [`Orchestrator`] trait.
    pub fn new<OrchestratorType>(orchestrator: OrchestratorType) -> Self
    where
        OrchestratorType:
            Orchestrator<Input = Input, Output = Output, Error = Error> + Debug + 'static,
    {
        Self {
            orchestrator: Arc::new(Box::new(orchestrator)),
        }
    }
}

#[cfg_attr(not(docs_cfg), async_trait)]
impl<Input, Output, Error> Node for OrchestratorAsNode<Input, Output, Error>
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
        Self::advance(self.orchestrator.run(input).await?).into()
    }
}

/// Extension for [`Orchestrator`] trait that converts orchestrator into [`Node`].
pub trait OrchestratorAsNodeExt: Orchestrator + Debug {
    /// Converts [`Orchestrator`] into [`Node`].
    fn into_node(self) -> OrchestratorAsNode<Self::Input, Self::Output, Self::Error>
    where
        Self: Sized,
    {
        OrchestratorAsNode::new(self)
    }
}
impl<T> OrchestratorAsNodeExt for T where T: Orchestrator + Debug {}
