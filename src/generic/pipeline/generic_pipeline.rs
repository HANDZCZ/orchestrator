use std::{
    any::{Any, TypeId},
    fmt::Debug,
    marker::PhantomData,
};

use async_trait::async_trait;

use crate::{
    generic::node::{NextNode, Node, NodeOutput},
    pipeline::Pipeline,
};

use super::internal_node::{InternalNode, InternalNodeOutput, InternalNodeStruct};

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
#[derive(Debug)]
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
    nodes: Vec<Box<dyn InternalNode<Error>>>,
}

/// Defines which errors can occur in [`GenericPipeline`].
#[derive(Debug)]
pub enum PipelineError {
    /// Output that is supposed to be returned from pipeline has wrong type.
    WrongOutputTypeForPipeline {
        /// Name of the node which returned the wrong type.
        node_type_name: &'static str,
    },
    /// [`Node`] with specified type could not be found in pipeline.
    /// Most likely you just forgot to add it to the pipeline.
    NodeWithTypeNotFound {
        /// Name of the node which could not be found.
        node_type_name: &'static str,
    },
}

/// Specifies the type for new [`GenericPipeline`].
pub type GenericPipelineNew<Input, Output, Error> =
    GenericPipeline<Input, Output, Error, AnyNodeInput, PipelineStateNew>;

/// Specifies the type for [`GenericPipeline`] that is in the process of adding nodes.
pub type GenericPipelineAddingNodes<Input, Output, Error, NextNodeInput> =
    GenericPipeline<Input, Output, Error, NextNodeInput, PipelineStateAddingNodes>;

/// Specifies the type for built [`GenericPipeline`].
pub type GenericPipelineBuilt<Input, Output, Error, LastNodeOutput> =
    GenericPipeline<Input, Output, Error, LastNodeOutput, PipelineStateBuilt>;

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

impl<Input, Output, Error, LastNodeOutput>
    GenericPipelineBuilt<Input, Output, Error, LastNodeOutput>
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
        data: Box<dyn Any + Send + Sync>,
        node: &dyn InternalNode<Error>,
    ) -> Result<PipelineOutput<Output>, Error> {
        match data.downcast::<Output>() {
            Ok(output) => Ok(PipelineOutput::Done(*output)),
            Err(_) => Err(PipelineError::WrongOutputTypeForPipeline {
                node_type_name: node.get_node_type_name(),
            }
            .into()),
        }
    }
}

/// Defines what [`GenericPipeline`] returns.
#[derive(Debug, PartialEq)]
pub enum PipelineOutput<T> {
    /// Says that the pipeline soft failed.
    ///
    /// For more information look at [`NodeOutput::SoftFail`].
    SoftFail,
    /// Pipeline finished successfully.
    Done(T),
}

#[cfg_attr(not(docs_cfg), async_trait)]
impl<Input, Output, Error, LastNodeOutput> Pipeline
    for GenericPipelineBuilt<Input, Output, Error, LastNodeOutput>
where
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    PipelineError: Into<Error>,
    LastNodeOutput: Send + Sync + 'static + Into<Output>,
{
    type Input = Input;
    type Output = PipelineOutput<Output>;
    type Error = Error;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let mut data: Box<dyn Any + Sync + Send> = Box::new(input);
        let mut piped = false;
        let mut index = 0;
        let mut first = self.nodes[0].duplicate();
        match first.run(data, piped).await? {
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
                // In other words data should contain LastNodeOutput type.
                let last_node_output = *data
                    .downcast::<LastNodeOutput>()
                    .expect("Downcast to LastNodeOutput failed, but it shouldn't.");
                return Ok(PipelineOutput::Done(last_node_output.into()));
            }
            let node = &mut nodes[index];
            match node.run(data, piped).await? {
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
            nodes: self.nodes,
        }
    }
}

impl<Input, Output, Error, LastNodeOutput>
    GenericPipelineAddingNodes<Input, Output, Error, LastNodeOutput>
where
    LastNodeOutput: Into<Output>,
{
    /// Finalizes the pipeline so any more nodes can't be added to it.
    #[must_use]
    pub fn finish(self) -> GenericPipelineBuilt<Input, Output, Error, LastNodeOutput> {
        GenericPipelineBuilt {
            _input: PhantomData,
            _output: PhantomData,
            _next_node_input: PhantomData,
            _pipeline_state: PhantomData,
            nodes: self.nodes,
        }
    }
}
