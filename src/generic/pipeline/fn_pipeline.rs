use std::{fmt::Debug, future::Future, marker::PhantomData};

use async_trait::async_trait;

use crate::pipeline::Pipeline;

use super::PipelineOutput;

/// Implementation of [`Pipeline`] trait.
/// That takes some async function and wraps around it to crate a pipeline.
///
/// This function takes some input type and returns a future with output type `Result<PipelineOutput<some output type>, some error type>`.
///
/// Example that shows usage of [`FnPipeline`].
/// ```
/// use orchestrator::{
///     generic::pipeline::{FnPipeline, PipelineOutput},
///     pipeline::Pipeline,
/// };
///
/// async fn normal_async_fn(input: String) -> Result<PipelineOutput<String>, ()> {
///     Ok(PipelineOutput::Done(input))
/// }
///
/// #[tokio::main]
/// async fn main() {
///     // create pipeline from function
///     let pipeline = FnPipeline::new(normal_async_fn);
///     let input = "Ok".to_string();
///     let res = pipeline.run(input.clone()).await;
///     assert_eq!(res, Ok(PipelineOutput::Done(input)));
///
///
///     let closure_with_async = |input: String| async {
///         Ok(PipelineOutput::Done(input))
///     };
///
///     // create pipeline from closure
///     // <_, _, (), _> is needed to tell the compiler what type Error has
///     let pipeline = FnPipeline::<_, _, (), _>::new(closure_with_async);
///     let input = "Ok".to_string();
///     let res = pipeline.run(input.clone()).await;
///     assert_eq!(res, Ok(PipelineOutput::Done(input)));
/// }
/// ```
#[derive(Clone)]
pub struct FnPipeline<Input, Output, Error, FnType> {
    _input_type: PhantomData<Input>,
    _output_type: PhantomData<Output>,
    _error_type: PhantomData<Error>,
    f: FnType,
}

impl<Input, Output, Error, FnType> Debug for FnPipeline<Input, Output, Error, FnType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnPipeline").finish_non_exhaustive()
    }
}

impl<Input, Output, Error, FnType, FutureOutput> FnPipeline<Input, Output, Error, FnType>
where
    FnType: Fn(Input) -> FutureOutput + Send + Sync + 'static,
    FutureOutput: Future<Output = Result<PipelineOutput<Output>, Error>> + Send + Sync,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    /// Creates new instance of [`FnPipeline`] from async function.
    #[must_use]
    pub fn new(f: FnType) -> Self {
        Self {
            _input_type: PhantomData,
            _output_type: PhantomData,
            _error_type: PhantomData,
            f,
        }
    }
}

#[cfg_attr(not(all(doc, not(doctest))), async_trait)]
impl<Input, Output, Error, FnType, FutureOutput> Pipeline
    for FnPipeline<Input, Output, Error, FnType>
where
    FnType: Fn(Input) -> FutureOutput + Send + Sync + 'static,
    FutureOutput: Future<Output = Result<PipelineOutput<Output>, Error>> + Send + Sync,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    type Input = Input;
    type Output = PipelineOutput<Output>;
    type Error = Error;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let fut = (self.f)(input);
        fut.await
    }
}

impl<Input, Output, Error, FnType, FutureOutput> From<FnType>
    for FnPipeline<Input, Output, Error, FnType>
where
    FnType: Fn(Input) -> FutureOutput + Send + Sync + 'static,
    FutureOutput: Future<Output = Result<PipelineOutput<Output>, Error>> + Send + Sync,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    fn from(value: FnType) -> Self {
        Self::new(value)
    }
}
