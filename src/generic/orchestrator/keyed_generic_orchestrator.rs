use std::{collections::HashMap, fmt::Debug, hash::Hash};

use async_trait::async_trait;

use crate::{
    orchestrator::{ErrorInner, Orchestrator, OrchestratorRunInner},
    pipeline::Pipeline,
};

use super::{
    internal_pipeline::{InternalPipelineOutput, InternalPipelineStruct},
    InternalPipelineType, OrchestratorError,
};

/// Generic implementation of [`Orchestrator`] trait.
/// That takes some input type and returns some output type or some error type.
///
/// This type should be used when multiple pipelines working on the same type of data are used.
/// Because [`KeyedGenericOrchestrator`] puts pipelines into buckets that are separated by a key.
/// This key can be anything as long as it implements [`Hash`] and [`Eq`].
///
/// If [`GenericOrchestrator`](super::GenericOrchestrator) is used instead of [`KeyedGenericOrchestrator`],
/// then pipelines are ran until all of them soft fail or until pipeline, that is able to process this data successfully is found.
///
/// Input type to this orchestrator can be different from the pipeline input type as long as orchestrator input implements `Into<Pipeline::Input>`.
/// The same is true for orchestrator output and error. Pipeline output/error type must implement `Into<Orchestrator::Output>`/`Into<Orchestrator::Error>`.
///
/// Example that shows usage of [`KeyedGenericOrchestrator`].
/// ```
/// // type to convert from and into
/// #[derive(Debug, Default, Hash, Eq, PartialEq, Clone)]
/// enum EventType {
///     Event1,
///     Event2,
///     #[default]
///     Unknown,
/// }
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
/// #[derive(Debug, Default, Clone, PartialEq)]
/// struct Event {
///     event_type: EventType,
///     some_data: String,
/// }
///
/// # use orchestrator::{async_trait, pipeline::Pipeline, orchestrator::Orchestrator, generic::{pipeline::PipelineOutput, orchestrator::{KeyedGenericOrchestrator, OrchestratorError}}};
/// # use std::marker::PhantomData;
/// # use std::sync::{RwLock, Arc};
/// #[tokio::main]
/// async fn main() {
///     let known_input = Event {
///         event_type: EventType::Event2,
///         some_data: "Hi".into(),
///     };
///     let unknown_input = Event {
///         event_type: EventType::Unknown,
///         some_data: "huh".into(),
///     };
///
///     // construct generic orchestrator that takes and returns an event and it's key is EventType
///     let mut orchestrator = KeyedGenericOrchestrator::<Event, Event, MyOrchestratorError<()>, EventType>::new(|e| e.event_type.clone());
///
///     // construct and add a pipeline that works with key EventType::Event1
///     let event1_pipeline = ForwardPipeline::<Event>::default();
///     orchestrator.add_pipeline(EventType::Event1, event1_pipeline.clone());
///
///     // construct and add a pipeline that works with key EventType::Event2
///     let event2_pipeline = ForwardPipeline::<Event>::default();
///     orchestrator.add_pipeline(EventType::Event2, event2_pipeline.clone());
///
///     // run the orchestrator
///     let known_output = orchestrator.run(known_input.clone()).await;
///     let unknown_output = orchestrator.run(unknown_input).await;
///
///     // known output should be the same as input
///     assert_eq!(known_output, Ok(known_input));
///     // unknown output should be an error
///     assert_eq!(unknown_output, Err(MyOrchestratorError::<()>::AllPipelinesSoftFailed));
///
///     // only the event2_pipeline should have ran since it's the only one keyed for EventType::Event2
///     let ran = event1_pipeline.ran.read().unwrap();
///     assert_eq!(*ran, 0);
///     let ran = event2_pipeline.ran.read().unwrap();
///     assert_eq!(*ran, 1);
/// }
/// ```
pub struct KeyedGenericOrchestrator<Input, Output, Error, Key: Debug> {
    pipeline_buckets: HashMap<Key, Vec<InternalPipelineType<Input, Output, Error>>>,
    key_extractor: Box<dyn Fn(&Input) -> Key + Send + Sync>,
}

impl<Input, Output, Error, Key: Debug> Debug
    for KeyedGenericOrchestrator<Input, Output, Error, Key>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyedGenericOrchestrator")
            .field("pipeline_buckets", &self.pipeline_buckets)
            .finish_non_exhaustive()
    }
}

#[cfg_attr(not(docs_cfg), async_trait)]
impl<Input, Output, Error, Key> Orchestrator for KeyedGenericOrchestrator<Input, Output, Error, Key>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    Key: Send + Sync + 'static + Eq + Hash + Debug,
    OrchestratorError: Into<Error>,
{
    type Input = Input;
    type Output = Output;
    type Error = Error;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        self.run_inner(input).await.map_err(|e| match e {
            ErrorInner::AllPipelinesSoftFailed => OrchestratorError::AllPipelinesSoftFailed.into(),
            ErrorInner::Other(e) => e,
        })
    }
}

#[cfg_attr(not(docs_cfg), async_trait)]
impl<Input, Output, Error, Key> OrchestratorRunInner
    for KeyedGenericOrchestrator<Input, Output, Error, Key>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    Key: Send + Sync + 'static + Hash + Eq + Debug,
    OrchestratorError: Into<Error>,
{
    async fn run_inner(&self, input: Self::Input) -> Result<Self::Output, ErrorInner<Self::Error>> {
        let key = (self.key_extractor)(&input);
        let Some(pipelines) = self.pipeline_buckets.get(&key) else {
            return Err(ErrorInner::AllPipelinesSoftFailed);
        };
        for pipeline in pipelines {
            match pipeline
                .run(input.clone())
                .await
                .map_err(|e| ErrorInner::Other(e))?
            {
                InternalPipelineOutput::SoftFail => continue,
                InternalPipelineOutput::Done(output) => return Ok(output),
            }
        }
        Err(ErrorInner::AllPipelinesSoftFailed)
    }
}

impl<Input, Output, Error, Key> KeyedGenericOrchestrator<Input, Output, Error, Key>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    Key: Send + Sync + 'static + Eq + Hash + Debug,
    OrchestratorError: Into<Error>,
{
    /// Creates a new instance of [`KeyedGenericOrchestrator`].
    #[must_use]
    pub fn new(key_extractor: impl Fn(&Input) -> Key + Send + Sync + 'static) -> Self {
        Self {
            pipeline_buckets: HashMap::new(),
            key_extractor: Box::new(key_extractor),
        }
    }

    /// Adds a pipeline to the [`KeyedGenericOrchestrator`] with specified key.
    ///
    /// Pipeline needs to have an input type that can be converted from orchestrator input type.
    /// It also needs to have an output/error type that can be converted to orchestrator output/error type.
    pub fn add_pipeline<PipelineType>(&mut self, key: Key, pipeline: PipelineType)
    where
        PipelineType: Pipeline + Debug,
        Input: Into<PipelineType::Input>,
        PipelineType::Output: Into<InternalPipelineOutput<Output>>,
        PipelineType::Error: Into<Error>,
    {
        let bucket = self.pipeline_buckets.entry(key).or_default();
        bucket.push(Box::new(InternalPipelineStruct::new(pipeline)));
    }
}
