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

#[derive(Debug)]
pub struct PipelineStateNew;
#[derive(Debug)]
pub struct PipelineStateAddingNodes;
#[derive(Debug)]
pub struct PipelineStateBuilt;

#[derive(Debug)]
pub struct GenericPipeline<
    Input: Debug + Send + Sync + Clone + 'static,
    Output: Debug + Send + Sync + 'static,
    Error: From<PipelineError> + Send + Sync + 'static,
    State = PipelineStateNew,
> {
    _input: PhantomData<Input>,
    _output: PhantomData<Output>,
    _pipeline_state: PhantomData<State>,
    nodes: Vec<Box<dyn InternalNode<Error>>>,
}

#[derive(Debug)]
pub enum PipelineError {
    WrongInputTypeForNode { node_type_name: &'static str },
    WrongOutputTypeForPipeline,
    NodeWithTypeNotFound { node_type_name: &'static str },
}

impl<
        Input: Debug + Send + Sync + Clone + 'static,
        Output: Debug + Send + Sync + 'static,
        Error: From<PipelineError> + Send + Sync + 'static,
    > GenericPipeline<Input, Output, Error, PipelineStateNew>
{
    pub fn start<NodeError: Into<Error>>(
        node: impl Node<Input = Input, Error = NodeError> + Debug,
    ) -> GenericPipeline<Input, Output, Error, PipelineStateAddingNodes> {
        GenericPipeline {
            _input: PhantomData,
            _output: PhantomData,
            _pipeline_state: PhantomData,
            nodes: vec![Box::new(InternalNodeStruct::new(node))],
        }
    }
}

impl<
        Input: Debug + Send + Sync + Clone + 'static,
        Output: Debug + Send + Sync + 'static,
        Error: From<PipelineError> + Send + Sync + 'static,
    > GenericPipeline<Input, Output, Error, PipelineStateBuilt>
{
    fn get_node_index(&self, ty: TypeId) -> Option<usize> {
        self.nodes
            .iter()
            .enumerate()
            .find_map(|(i, node)| (node.get_node_type() == ty).then_some(i))
    }

    fn get_pipeline_output(
        data: Box<dyn Any + Send + Sync>,
    ) -> Result<PipelineOutput<Output>, Error> {
        match data.downcast::<Output>() {
            Ok(output) => Ok(PipelineOutput::Done(*output)),
            Err(_) => Err(Error::from(PipelineError::WrongOutputTypeForPipeline)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PipelineOutput<T> {
    SoftFail,
    Done(T),
}

#[async_trait]
impl<
        Input: Debug + Send + Sync + Clone + 'static,
        Output: Debug + Send + Sync + 'static,
        Error: From<PipelineError> + Send + Sync + 'static,
    > Pipeline for GenericPipeline<Input, Output, Error, PipelineStateBuilt>
{
    type Input = Input;
    type Output = PipelineOutput<Output>;
    type Error = Error;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let mut data: Box<dyn Any + Sync + Send> = Box::new(input);
        let mut index = 0;
        let mut first = self.nodes[0].duplicate();
        match first.run(data).await? {
            InternalNodeOutput::NodeOutput(NodeOutput::SoftFail) => {
                return Ok(PipelineOutput::SoftFail)
            }
            InternalNodeOutput::NodeOutput(NodeOutput::ReturnFromPipeline(data)) => {
                return Self::get_pipeline_output(data);
            }
            InternalNodeOutput::NodeOutput(NodeOutput::PipeToNode(NextNode {
                output,
                next_node_type,
                next_node_type_name,
            })) => {
                data = output;
                index = self.get_node_index(next_node_type).ok_or(Error::from(
                    PipelineError::NodeWithTypeNotFound {
                        node_type_name: next_node_type_name,
                    },
                ))?;
            }
            InternalNodeOutput::NodeOutput(NodeOutput::Advance(output)) => {
                data = output;
                index += 1;
            }
            InternalNodeOutput::WrongInputType => {
                return Err(Error::from(PipelineError::WrongInputTypeForNode {
                    node_type_name: first.get_node_type_name(),
                }));
            }
        };

        let mut nodes: Vec<Box<dyn InternalNode<Error>>> = Vec::with_capacity(self.nodes.len());
        nodes.push(first);
        nodes.extend(self.nodes.iter().skip(1).map(|node| node.duplicate()));

        loop {
            if index >= nodes.len() {
                return Self::get_pipeline_output(data);
            }
            let node = &mut nodes[index];
            match node.run(data).await? {
                InternalNodeOutput::NodeOutput(NodeOutput::SoftFail) => {
                    return Ok(PipelineOutput::SoftFail)
                }
                InternalNodeOutput::NodeOutput(NodeOutput::ReturnFromPipeline(data)) => {
                    return Self::get_pipeline_output(data);
                }
                InternalNodeOutput::NodeOutput(NodeOutput::PipeToNode(NextNode {
                    output,
                    next_node_type,
                    next_node_type_name,
                })) => {
                    data = output;
                    index = self.get_node_index(next_node_type).ok_or(Error::from(
                        PipelineError::NodeWithTypeNotFound {
                            node_type_name: next_node_type_name,
                        },
                    ))?;
                }
                InternalNodeOutput::NodeOutput(NodeOutput::Advance(output)) => {
                    data = output;
                    index += 1;
                }
                InternalNodeOutput::WrongInputType => {
                    return Err(Error::from(PipelineError::WrongInputTypeForNode {
                        node_type_name: node.get_node_type_name(),
                    }));
                }
            }
        }
    }
}

impl<
        Input: Debug + Send + Sync + Clone + 'static,
        Output: Debug + Send + Sync + 'static,
        Error: From<PipelineError> + Send + Sync + 'static,
    > GenericPipeline<Input, Output, Error, PipelineStateAddingNodes>
{
    pub fn add_node<NodeError: Into<Error>>(
        mut self,
        node: impl Node<Error = NodeError> + Debug,
    ) -> Self {
        self.nodes.push(Box::new(InternalNodeStruct::new(node)));
        self
    }

    pub fn finish<NodeError: Into<Error>>(
        self,
        node: impl Node<Output = Output, Error = NodeError> + Debug,
    ) -> GenericPipeline<Input, Output, Error, PipelineStateBuilt> {
        GenericPipeline {
            _input: PhantomData,
            _output: PhantomData,
            _pipeline_state: PhantomData,
            nodes: self.add_node(node).nodes,
        }
    }
}
