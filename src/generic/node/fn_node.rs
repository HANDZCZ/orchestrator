use std::{fmt::Debug, future::Future, marker::PhantomData};

use async_trait::async_trait;

use crate::generic::pipeline::PipelineStorage;

use super::{Node, NodeOutput};

/// Implementation of [`Node`] trait.
/// That takes some async function and wraps around it to crate a node.
/// This function takes some input type and returns a future with output type `Result<NodeOutput<some output type>, some error type>`.
///
/// Example that shows usage of [`FnNode`].
/// ```
/// use orchestrator::{
///     generic::{
///         node::{FnNode, AnyNode, NodeOutput, Returnable},
///         pipeline::{GenericPipeline, PipelineError, PipelineOutput},
///         AnyDebug,
///     },
///     pipeline::Pipeline,
/// };
///
/// #[derive(Debug)]
/// enum MyPipelineError {
///     WrongPipelineOutput {
///         node_type_name: &'static str,
///         data: Box<dyn AnyDebug>,
///         expected_type_name: &'static str,
///         got_type_name: &'static str,
///     },
///     NodeNotFound(&'static str),
///     SomeNodeError,
/// }
/// #
/// // conversion from generic pipeline error into ours
/// impl From<PipelineError> for MyPipelineError //...
/// # {
/// #     fn from(value: PipelineError) -> Self {
/// #         match value {
/// #             PipelineError::WrongOutputTypeForPipeline {
/// #                 data,
/// #                 node_type_name,
/// #                 expected_type_name,
/// #                 got_type_name,
/// #             } => Self::WrongPipelineOutput {
/// #                 data,
/// #                 expected_type_name,
/// #                 got_type_name,
/// #                 node_type_name,
/// #             },
/// #             PipelineError::NodeWithTypeNotFound { node_type_name } => {
/// #                 Self::NodeNotFound(node_type_name)
/// #             }
/// #         }
/// #     }
/// # }
/// #
/// // conversion from Node errors into ours
/// impl From<()> for MyPipelineError {
///     fn from(_value: ()) -> Self {
///         Self::SomeNodeError
///     }
/// }
///
/// async fn normal_async_fn(input: String) -> Result<NodeOutput<String>, ()> {
///     // use AnyNode to construct NodeOutput
///     AnyNode::advance(input).into()
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let closure_with_async = |input: String| async {
///         // use AnyNode to construct NodeOutput
///         AnyNode::advance(input).into()
///     };
///
///     // construct generic pipeline that takes and returns a string
///     let pipeline = GenericPipeline::<String, String, MyPipelineError>::new()
///
///         // create and add node from function
///         .add_node(FnNode::new(normal_async_fn))
///
///         // create and add node from closure
///         // <_, _, (), _> is needed to tell the compiler what type Error has
///         .add_node(FnNode::<_, _, (), _>::new(closure_with_async))
///
///         // now just finish the pipeline
///         .finish();
///
///     let input = "Ok".to_string();
///     let res = pipeline.run(input.clone()).await;
///     let expected: Result<_, MyPipelineError> = Ok(PipelineOutput::Done(input));
///     assert!(matches!(res, expected));
/// }
/// ```
pub struct FnNode<Input, Output, Error, FnType> {
    _input_type: PhantomData<Input>,
    _output_type: PhantomData<Output>,
    _error_type: PhantomData<Error>,
    f: FnType,
}

impl<Input, Output, Error, FnType: Clone> Clone for FnNode<Input, Output, Error, FnType> {
    fn clone(&self) -> Self {
        Self {
            _input_type: PhantomData,
            _output_type: PhantomData,
            _error_type: PhantomData,
            f: self.f.clone(),
        }
    }
}

impl<Input, Output, Error, FnType> Debug for FnNode<Input, Output, Error, FnType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnNode")
            .field("_input_type", &self._input_type)
            .field("_output_type", &self._output_type)
            .field("_error_type", &self._error_type)
            .finish_non_exhaustive()
    }
}

impl<Input, Output, Error, FnType, FutureOutput> FnNode<Input, Output, Error, FnType>
where
    FnType: Fn(Input, &mut PipelineStorage) -> FutureOutput + Clone + Send + Sync + 'static,
    FutureOutput: Future<Output = Result<NodeOutput<Output>, Error>> + Send + Sync,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    /// Creates new instance of [`FnNode`] from async function.
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

#[cfg_attr(not(docs_cfg), async_trait)]
impl<Input, Output, Error, FnType, FutureOutput> Node for FnNode<Input, Output, Error, FnType>
where
    FnType: Fn(Input, &mut PipelineStorage) -> FutureOutput + Clone + Send + Sync + 'static,
    FutureOutput: Future<Output = Result<NodeOutput<Output>, Error>> + Send + Sync,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    type Input = Input;
    type Output = Output;
    type Error = Error;

    async fn run(
        &mut self,
        input: Self::Input,
        pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error> {
        let fut = (self.f)(input, pipeline_storage);
        fut.await
    }
}

impl<Input, Output, Error, FnType, FutureOutput> From<FnType>
    for FnNode<Input, Output, Error, FnType>
where
    FnType: Fn(Input, &mut PipelineStorage) -> FutureOutput + Clone + Send + Sync + 'static,
    FutureOutput: Future<Output = Result<NodeOutput<Output>, Error>> + Send + Sync,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    fn from(value: FnType) -> Self {
        Self::new(value)
    }
}
