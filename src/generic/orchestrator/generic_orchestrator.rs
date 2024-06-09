use std::fmt::Debug;

use async_trait::async_trait;

use crate::{orchestrator::Orchestrator, pipeline::Pipeline};

use super::internal_pipeline::{InternalPipeline, InternalPipelineOutput, InternalPipelineStruct};

/// Defines which errors can occur in [`GenericOrchestrator`].
#[derive(Debug)]
pub enum OrchestratorError {
    /// All the pipelines added to orchestrator soft failed.
    ///
    /// For more information about soft fail look at [`NodeOutput::SoftFail`](crate::generic::node::NodeOutput::SoftFail).
    AllPipelinesSoftFailed,
}

/// Generic implementation of [`Orchestrator`] trait.
/// That takes some input type and returns some output type or some error type.
///
/// Input type to this orchestrator can be different from the pipeline input type as long as orchestrator input implements `Into<Pipeline::Input>`.
/// The same is true for orchestrator output and error. Pipeline output/error type must implement `Into<Orchestrator::Output>`/`Into<Orchestrator::Error>`.
///
/// Example that shows usage of [`GenericOrchestrator`] with input and output conversion.
/// ```
/// // type to convert from and into
/// #[derive(Debug, Default, Clone, PartialEq)]
/// struct WrapString(String);
/// #
/// impl From<String> for WrapString //...
/// # {
/// #     fn from(value: String) -> Self {
/// #         WrapString(value)
/// #     }
/// # }
/// #
/// impl From<WrapString> for String //...
/// # {
/// #     fn from(value: WrapString) -> Self {
/// #         value.0
/// #     }
/// # }
///
/// #[derive(Debug, PartialEq)]
/// enum MyOrchestratorError<T> {
///     AllPipelinesSoftFailed,
///     PipelineError(T),
/// }
/// #
/// // conversion from generic orchestrator error into ours
/// impl<T> From<OrchestratorError> for MyOrchestratorError<T> //...
/// # {
/// #     fn from(_value: orchestrator::generic::orchestrator::OrchestratorError) -> Self {
/// #         Self::AllPipelinesSoftFailed
/// #     }
/// # }
/// #
/// // conversion from ForwardPipeline error into ours
/// impl From<()> for MyOrchestratorError<()> //...
/// # {
/// #     fn from(_value: ()) -> Self {
/// #         Self::PipelineError(())
/// #     }
/// # }
///
/// // some pipeline implementation
/// #[derive(Default, Debug, Clone)]
/// struct ForwardPipeline<T: Default> {
///     _type: PhantomData<T>,
///     // tells us how many times the pipeline ran
///     ran: Arc<RwLock<usize>>,
/// }
/// #
/// #[async_trait]
/// impl<T: Send + Sync + 'static + Default> Pipeline for ForwardPipeline<T> {
///     type Input = T;
///     type Output = PipelineOutput<T>;
///     type Error = ();
///
///     async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
///         let mut ran = self.ran.write().unwrap();
///         *ran += 1;
///         // here you want to actually do something like running some nodes
///         // but we will just forward input to output
///         Ok(PipelineOutput::Done(input))
///     }
/// }
///
/// # use orchestrator::{async_trait, pipeline::Pipeline, orchestrator::Orchestrator, generic::{pipeline::PipelineOutput, orchestrator::{GenericOrchestrator, OrchestratorError}}};
/// # use std::marker::PhantomData;
/// # use std::sync::{RwLock, Arc};
/// #[tokio::main]
/// async fn main() {
///     let input = "Nice".to_owned();
///
///     // construct generic orchestrator that takes and returns a string
///     let mut orchestrator = GenericOrchestrator::<String, String, MyOrchestratorError<()>>::new();
///
///     // construct and add a pipeline that takes and returns a WrapString
///     // the orchestrator input (String) will be converted into WrapString
///     // thanks to implementation of WrapString: From<String> above
///     // and the pipeline output (WrapString) wil be converted to String
///     // thanks to implementation of String: From<WrapString> above
///     let wrap_string_pipeline = ForwardPipeline::<WrapString>::default();
///     orchestrator.add_pipeline(wrap_string_pipeline.clone());
///
///     // construct and add a pipeline that takes and returns a String
///     let string_pipeline = ForwardPipeline::<String>::default();
///     orchestrator.add_pipeline(string_pipeline.clone());
///
///     // run the orchestrator
///     let output = orchestrator.run(input.clone()).await;
///
///     // output should be the same as input
///     assert_eq!(output, Ok(input));
///
///     // only the first pipeline should have ran since it didn't soft failed
///     let ran = wrap_string_pipeline.ran.read().unwrap();
///     assert_eq!(*ran, 1);
///     let ran = string_pipeline.ran.read().unwrap();
///     assert_eq!(*ran, 0);
/// }
/// ```
#[derive(Debug)]
pub struct GenericOrchestrator<Input, Output, Error> {
    pipelines: Vec<Box<dyn InternalPipeline<Input, InternalPipelineOutput<Output>, Error>>>,
}

#[cfg_attr(not(docs_cfg), async_trait)]
impl<Input, Output, Error> Orchestrator for GenericOrchestrator<Input, Output, Error>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    OrchestratorError: Into<Error>,
{
    type Input = Input;
    type Output = Output;
    type Error = Error;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        for pipeline in &self.pipelines {
            match pipeline.run(input.clone()).await? {
                InternalPipelineOutput::SoftFail => continue,
                InternalPipelineOutput::Done(output) => return Ok(output),
            }
        }
        Err(OrchestratorError::AllPipelinesSoftFailed.into())
    }
}

impl<Input, Output, Error> GenericOrchestrator<Input, Output, Error>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    OrchestratorError: Into<Error>,
{
    /// Creates a new instance of [`GenericOrchestrator`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
        }
    }

    /// Adds a pipeline to the [`GenericOrchestrator`] which needs to have same the input and output as the [`GenericOrchestrator`].
    /// It also needs to have an error type that implements `Into<OrchestratorErrorType>`.
    pub fn add_pipeline<PipelineType>(&mut self, pipeline: PipelineType)
    where
        PipelineType: Pipeline + Debug,
        Input: Into<PipelineType::Input>,
        PipelineType::Output: Into<InternalPipelineOutput<Output>>,
        PipelineType::Error: Into<Error>,
    {
        self.pipelines
            .push(Box::new(InternalPipelineStruct::new(pipeline)));
    }
}

impl<Input, Output, Error> Default for GenericOrchestrator<Input, Output, Error>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    OrchestratorError: Into<Error>,
{
    fn default() -> Self {
        Self::new()
    }
}
