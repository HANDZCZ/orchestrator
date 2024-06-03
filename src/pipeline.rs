use std::{
    any::{Any, TypeId},
    fmt::Debug,
    marker::PhantomData,
};

use crate::{
    internal::{InternalNode, InternalNodeOutput, InternalNodeStruct},
    node::{NextNode, Node, NodeOutput},
};

#[derive(Debug)]
pub struct PipelineStateNew;
#[derive(Debug)]
pub struct PipelineStateAddingNodes;
#[derive(Debug)]
pub struct PipelineStateBuilt;

#[derive(Debug)]
pub struct Pipeline<
    Input: Send + Sync + Clone + 'static,
    Output: Send + Sync + 'static,
    Error: PipelineError,
    State = PipelineStateNew,
> {
    _input: PhantomData<Input>,
    _output: PhantomData<Output>,
    _pipeline_state: PhantomData<State>,
    nodes: Vec<Box<dyn InternalNode<Error>>>,
}

#[derive(Debug, PartialEq)]
pub enum PipelineOutput<T> {
    SoftFail,
    Done(T),
}

pub trait PipelineError {
    fn wrong_input_type_for_node(node_type_name: &'static str) -> Self;
    fn wrong_output_type_for_pipeline() -> Self;
    fn node_with_type_not_found(node_type_name: &'static str) -> Self;
}

impl<Input: Send + Sync + Clone + 'static, Output: Send + Sync + 'static, Error: PipelineError>
    Pipeline<Input, Output, Error, PipelineStateNew>
{
    pub fn start(
        node: impl Node<Input = Input, Error = Error>,
    ) -> Pipeline<Input, Output, Error, PipelineStateAddingNodes> {
        Pipeline {
            _input: PhantomData,
            _output: PhantomData,
            _pipeline_state: PhantomData,
            nodes: vec![Box::new(InternalNodeStruct::new(node))],
        }
    }
}

impl<Input: Send + Sync + Clone + 'static, Output: Send + Sync + 'static, Error: PipelineError>
    Pipeline<Input, Output, Error, PipelineStateBuilt>
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
            Err(_) => Err(Error::wrong_output_type_for_pipeline()),
        }
    }

    pub fn run(&self, input: Input) -> Result<PipelineOutput<Output>, Error> {
        let mut data: Box<dyn Any + Sync + Send> = Box::new(input);
        let mut index = 0;
        let mut first = self.nodes[0].duplicate();
        match first.run(data)? {
            InternalNodeOutput::NodeOutput(NodeOutput::SoftFail) => {
                return Ok(PipelineOutput::SoftFail)
            }
            InternalNodeOutput::NodeOutput(NodeOutput::ReturnFromPipeline(data)) => {
                return Self::get_pipeline_output(data);
            }
            InternalNodeOutput::NodeOutput(NodeOutput::SuccessAndPipeOutput(NextNode {
                output,
                next_node_type,
                next_node_type_name,
            })) => {
                data = output;
                index = self
                    .get_node_index(next_node_type)
                    .ok_or(Error::node_with_type_not_found(next_node_type_name))?;
            }
            InternalNodeOutput::NodeOutput(NodeOutput::Advance(output)) => {
                data = output;
                index += 1;
            }
            InternalNodeOutput::WrongInputType => {
                return Err(Error::wrong_input_type_for_node(first.get_node_type_name()));
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
            match node.run(data)? {
                InternalNodeOutput::NodeOutput(NodeOutput::SoftFail) => {
                    return Ok(PipelineOutput::SoftFail)
                }
                InternalNodeOutput::NodeOutput(NodeOutput::ReturnFromPipeline(data)) => {
                    return Self::get_pipeline_output(data);
                }
                InternalNodeOutput::NodeOutput(NodeOutput::SuccessAndPipeOutput(NextNode {
                    output,
                    next_node_type,
                    next_node_type_name,
                })) => {
                    data = output;
                    index = self
                        .get_node_index(next_node_type)
                        .ok_or(Error::node_with_type_not_found(next_node_type_name))?;
                }
                InternalNodeOutput::NodeOutput(NodeOutput::Advance(output)) => {
                    data = output;
                    index += 1;
                }
                InternalNodeOutput::WrongInputType => {
                    return Err(Error::wrong_input_type_for_node(node.get_node_type_name()));
                }
            }
        }
    }
}

impl<Input: Send + Sync + Clone + 'static, Output: Send + Sync + 'static, Error: PipelineError>
    Pipeline<Input, Output, Error, PipelineStateAddingNodes>
{
    pub fn add_node(mut self, node: impl Node<Error = Error>) -> Self {
        self.nodes.push(Box::new(InternalNodeStruct::new(node)));
        self
    }

    pub fn finish(
        self,
        node: impl Node<Output = Output, Error = Error>,
    ) -> Pipeline<Input, Output, Error, PipelineStateBuilt> {
        Pipeline {
            _input: PhantomData,
            _output: PhantomData,
            _pipeline_state: PhantomData,
            nodes: self.add_node(node).nodes,
        }
    }
}
