use std::{
    any::{self, Any, TypeId},
    fmt::Debug,
    marker::PhantomData,
};

use async_trait::async_trait;

use crate::{
    generic::{
        node::{NextNode, Node, NodeOutput},
        AnyDebug,
    },
    pipeline::Pipeline,
};

use super::{
    internal_node::{InternalNode, InternalNodeOutput, InternalNodeStruct},
    PipelineError, PipelineOutput, PipelineStorage,
};

/// Marker type that is used to say that [`GenericPipeline`] was just created and does not have any nodes in it.
#[derive(Debug)]
pub struct PipelineStateNew;
/// Marker type that is used to say that [`GenericPipeline`] is in the process of adding nodes.
/// In this state pipeline has at least one node inside.
#[derive(Debug)]
pub struct PipelineStateAddingNodes;
/// Marker type that is used to say that [`GenericPipeline`] has been built.
/// In this state pipeline has at least one node inside.
#[derive(Debug)]
pub struct PipelineStateBuilt;

/// Marker type that is used to say that [`GenericPipeline`] can accept any type as the next [`Node`] input.
#[derive(Debug)]
pub struct AnyNodeInput;

/// Generic implementation of [`Pipeline`] trait.
/// That takes some input type and returns some output type or some error type.
///
/// Input type to this pipeline can be different from the first node input type as long as pipeline input implements `Into<NodeType::Input>`.
/// The same is true for last node output and any node error.
/// Last node output type must implement `Into<Pipeline::Output>` and any node error type must implement `Into<Pipeline::Error>`.
///
/// Input to next node can also have different type from previous node output type as long as previous node output type implements `Into<NextNode::Input>`.
///
/// Example that shows usage of [`GenericPipeline`] with input and output conversion between nodes and pipeline input and output.
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
/// // some node implementation
/// #[derive(Clone, Default, Debug)]
/// struct ForwardNode<T: Default> {
///     _type: PhantomData<T>,
/// }
/// #
/// #[async_trait]
/// impl<T: Send + Sync + Default + Clone + 'static> Node for ForwardNode<T> {
///     type Input = T;
///     type Output = T;
///     type Error = ();
///
///     async fn run(&mut self, input: Self::Input, pipeline_storage: &mut PipelineStorage) -> Result<NodeOutput<Self::Output>, Self::Error> {
///         // here you want to actually do something like some io bound operation...
///         Self::advance(input).into()
///     }
/// }
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
/// # use orchestrator::{async_trait, pipeline::Pipeline, generic::{AnyDebug, pipeline::{GenericPipeline, PipelineError, PipelineOutput, PipelineStorage}, node::{Node, Returnable, NodeOutput}}};
/// # use std::marker::PhantomData;
/// #[tokio::main]
/// async fn main() {
///     // construct generic pipeline that takes and returns a string
///     // notice the builder pattern - it is needed for type safety
///     let pipeline = GenericPipeline::<String, String, MyPipelineError>::new()
///
///         // construct and add a node that takes and returns a WrapString
///         // the pipeline input (String) will be converted into WrapString
///         // thanks to implementation of WrapString: From<String> above
///         .add_node(ForwardNode::<WrapString>::default())
///
///         // construct and add a node that takes and returns a String
///         // the node output (WrapString) wil be converted to String
///         // thanks to implementation of String: From<WrapString> above
///         .add_node(ForwardNode::<String>::default())
///
///         // construct and add a node that takes and returns a WrapString
///         //   (input to this will be converted to WrapString like the first node)
///         // the node output (WrapString) wil be converted to pipeline output (String)
///         // thanks to implementation of String: From<WrapString> above
///         .add_node(ForwardNode::<WrapString>::default())
///
///         // now just finish the pipeline
///         // here is the actual check for converting
///         // last nodes output type to pipeline output type (WrapString: Into<String>)
///         .finish();
///
///     // run the pipeline
///     let res = pipeline.run("match".into()).await;
///
///     // pipeline should run successfully
///     // and return the same thing that was at input
///     let expected: Result<_, MyPipelineError> = Ok(PipelineOutput::Done("match".to_owned()));
///     assert!(matches!(res, expected));
/// }
/// ```
pub struct GenericPipeline<
    Input,
    Output,
    Error,
    NextNodeInput = AnyNodeInput,
    State = PipelineStateNew,
> {
    _input: PhantomData<Input>,
    _output: PhantomData<Output>,
    _next_node_input: PhantomData<NextNodeInput>,
    _pipeline_state: PhantomData<State>,
    last_node_output_converter: Option<Box<dyn ConvertTo<Output>>>,
    nodes: Vec<Box<dyn InternalNode<Error>>>,
}

impl<Input, Output, Error, NextNodeInput, State> Debug
    for GenericPipeline<Input, Output, Error, NextNodeInput, State>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericPipeline")
            .field("nodes", &self.nodes)
            .finish_non_exhaustive()
    }
}

/// Specifies the type for new [`GenericPipeline`].
pub type GenericPipelineNew<Input, Output, Error> =
    GenericPipeline<Input, Output, Error, AnyNodeInput, PipelineStateNew>;

/// Specifies the type for [`GenericPipeline`] that is in the process of adding nodes.
pub type GenericPipelineAddingNodes<Input, Output, Error, NextNodeInput> =
    GenericPipeline<Input, Output, Error, NextNodeInput, PipelineStateAddingNodes>;

/// Specifies the type for built [`GenericPipeline`].
pub type GenericPipelineBuilt<Input, Output, Error> =
    GenericPipeline<Input, Output, Error, (), PipelineStateBuilt>;

impl<Input, Output, Error> GenericPipelineNew<Input, Output, Error>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    PipelineError: Into<Error>,
{
    /// Creates new instance of [`GenericPipeline`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            _input: PhantomData,
            _output: PhantomData,
            _next_node_input: PhantomData,
            _pipeline_state: PhantomData,
            last_node_output_converter: None,
            nodes: Vec::new(),
        }
    }
}

impl<Input, Output, Error> Default for GenericPipelineNew<Input, Output, Error>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    PipelineError: Into<Error>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Input, Output, Error> GenericPipelineBuilt<Input, Output, Error>
where
    Output: 'static,
    PipelineError: Into<Error>,
{
    fn get_node_index(&self, ty: TypeId) -> Option<usize> {
        self.nodes
            .iter()
            .enumerate()
            .find_map(|(i, node)| (node.get_node_type() == ty).then_some(i))
    }

    fn get_pipeline_output(
        data: Box<dyn AnyDebug>,
        node: &dyn InternalNode<Error>,
    ) -> Result<PipelineOutput<Output>, Error> {
        if (*data).type_id() == TypeId::of::<Output>() {
            Ok(PipelineOutput::Done(
                *data.into_box_any().downcast::<Output>().unwrap(),
            ))
        } else {
            let (type_name, data) = data.with_type_name();
            Err(PipelineError::WrongOutputTypeForPipeline {
                node_type_name: node.get_node_type_name(),
                got_type_name: type_name,
                data,
                expected_type_name: any::type_name::<Output>(),
            }
            .into())
        }
    }
}

#[cfg_attr(not(docs_cfg), async_trait)]
impl<Input, Output, Error> Pipeline for GenericPipelineBuilt<Input, Output, Error>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    PipelineError: Into<Error>,
{
    type Input = Input;
    type Output = PipelineOutput<Output>;
    type Error = Error;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let mut data: Box<dyn Any + Sync + Send> = Box::new(input);
        let mut pipeline_storage = PipelineStorage::new();
        let mut piped = false;
        let mut index = 0;
        let mut first = self.nodes[0].duplicate();
        match first.run(data, piped, &mut pipeline_storage).await? {
            InternalNodeOutput::NodeOutput(NodeOutput::SoftFail) => {
                return Ok(PipelineOutput::SoftFail)
            }
            InternalNodeOutput::NodeOutput(NodeOutput::ReturnFromPipeline(data)) => {
                return Self::get_pipeline_output(data, &*first);
            }
            InternalNodeOutput::NodeOutput(NodeOutput::PipeToNode(NextNode {
                output,
                next_node_type,
                next_node_type_name,
            })) => {
                data = output;
                piped = true;
                index = self.get_node_index(next_node_type).ok_or(
                    PipelineError::NodeWithTypeNotFound {
                        node_type_name: next_node_type_name,
                    }
                    .into(),
                )?;
            }
            InternalNodeOutput::NodeOutput(NodeOutput::Advance(output)) => {
                data = output;
                piped = false;
                index += 1;
            }
            InternalNodeOutput::WrongInputType => {
                unreachable!(
                    "Type safety for the win!\n\tIf you reach this something went seriously wrong."
                );
            }
        };

        let mut nodes: Vec<Box<dyn InternalNode<Error>>> = Vec::with_capacity(self.nodes.len());
        nodes.push(first);
        nodes.extend(self.nodes.iter().skip(1).map(|node| node.duplicate()));

        loop {
            if index >= nodes.len() {
                // When index is larger than nodes.len() last node in nodes should have been the last node that have ran.
                // In other words data should contain last node output type.
                let output = self
                    .last_node_output_converter
                    .as_ref()
                    .expect("Last node output converted should always exist in built pipeline")
                    .convert(data)
                    .expect("Converting data to pipeline output failed");
                return Ok(PipelineOutput::Done(output));
            }
            let node = &mut nodes[index];
            match node.run(data, piped, &mut pipeline_storage).await? {
                InternalNodeOutput::NodeOutput(NodeOutput::SoftFail) => {
                    return Ok(PipelineOutput::SoftFail)
                }
                InternalNodeOutput::NodeOutput(NodeOutput::ReturnFromPipeline(data)) => {
                    return Self::get_pipeline_output(data, &**node);
                }
                InternalNodeOutput::NodeOutput(NodeOutput::PipeToNode(NextNode {
                    output,
                    next_node_type,
                    next_node_type_name,
                })) => {
                    data = output;
                    piped = true;
                    index = self.get_node_index(next_node_type).ok_or(
                        PipelineError::NodeWithTypeNotFound {
                            node_type_name: next_node_type_name,
                        }
                        .into(),
                    )?;
                }
                InternalNodeOutput::NodeOutput(NodeOutput::Advance(output)) => {
                    data = output;
                    piped = false;
                    index += 1;
                }
                InternalNodeOutput::WrongInputType => {
                    unreachable!("Type safety for the win!\n\tIf you reach this something went seriously wrong.");
                }
            }
        }
    }
}

impl<Input, Output, Error> GenericPipelineNew<Input, Output, Error>
where
    Input: Debug + Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    PipelineError: Into<Error>,
{
    /// Adds node to the [`GenericPipeline`].
    pub fn add_node<NodeType>(
        mut self,
        node: NodeType,
    ) -> GenericPipelineAddingNodes<Input, Output, Error, NodeType::Output>
    where
        NodeType: Node + Debug,
        Input: Into<NodeType::Input>,
        NodeType::Error: Into<Error>,
    {
        self.nodes
            .push(Box::new(InternalNodeStruct::<NodeType, Input>::new(node)));
        GenericPipelineAddingNodes {
            _input: PhantomData,
            _output: PhantomData,
            _next_node_input: PhantomData,
            _pipeline_state: PhantomData,
            last_node_output_converter: None,
            nodes: self.nodes,
        }
    }
}

impl<Input, Output, Error, NodeInput> GenericPipelineAddingNodes<Input, Output, Error, NodeInput>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    PipelineError: Into<Error>,
    NodeInput: Debug + Send + Sync + 'static,
{
    /// Adds node to the [`GenericPipeline`].
    pub fn add_node<NodeType>(
        mut self,
        node: NodeType,
    ) -> GenericPipelineAddingNodes<Input, Output, Error, NodeType::Output>
    where
        NodeType: Node + Debug,
        NodeType::Error: Into<Error>,
        NodeInput: Into<NodeType::Input>,
    {
        self.nodes
            .push(Box::new(InternalNodeStruct::<NodeType, NodeInput>::new(
                node,
            )));
        GenericPipelineAddingNodes {
            _input: PhantomData,
            _output: PhantomData,
            _next_node_input: PhantomData,
            _pipeline_state: PhantomData,
            last_node_output_converter: None,
            nodes: self.nodes,
        }
    }
}

impl<Input, Output, Error, LastNodeOutput>
    GenericPipelineAddingNodes<Input, Output, Error, LastNodeOutput>
where
    Output: Send + Sync + 'static,
    LastNodeOutput: Into<Output> + Send + Sync + 'static,
{
    /// Finalizes the pipeline so any more nodes can't be added to it.
    #[must_use]
    pub fn finish(self) -> GenericPipelineBuilt<Input, Output, Error> {
        GenericPipelineBuilt {
            _input: PhantomData,
            _output: PhantomData,
            _next_node_input: PhantomData,
            _pipeline_state: PhantomData,
            last_node_output_converter: Some(Box::new(
                DowncastConverter::<LastNodeOutput, Output>::new(),
            )),
            nodes: self.nodes,
        }
    }
}

trait ConvertTo<T>: Send + Sync {
    fn convert(&self, data: Box<dyn Any + Send + Sync>) -> Option<T>;
}

struct DowncastConverter<Input, Output>
where
    Input: Into<Output>,
{
    _node_output_type: PhantomData<Input>,
    _pipeline_output_type: PhantomData<Output>,
}

impl<Input, Output> DowncastConverter<Input, Output>
where
    Input: Into<Output>,
{
    fn new() -> Self {
        Self {
            _node_output_type: PhantomData,
            _pipeline_output_type: PhantomData,
        }
    }
}

impl<FromType, IntoType> ConvertTo<IntoType> for DowncastConverter<FromType, IntoType>
where
    FromType: Into<IntoType> + 'static + Send + Sync,
    IntoType: Send + Sync,
{
    fn convert(&self, data: Box<dyn Any + Send + Sync>) -> Option<IntoType> {
        let box_from = data.downcast::<FromType>().ok()?;
        let from = *box_from;
        let into = from.into();
        Some(into)
    }
}
