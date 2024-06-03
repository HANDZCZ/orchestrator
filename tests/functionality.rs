use std::fmt::Debug;

use orchestrator::{
    node::{Node, NodeOutput, Returnable}, orchestrator::{GeneralOrchestrator, Orchestrator, OrchestratorError}, pipeline::{Pipeline, PipelineError, PipelineOutput}
};

#[derive(Debug, PartialEq)]
enum MyError {
    WrongPipelineOutput,
    NodeNotFound(&'static str),
    WrongNodeInput(&'static str),
    NoPipeline
}

impl PipelineError for MyError {
    fn wrong_output_type_for_pipeline() -> Self {
        MyError::WrongPipelineOutput
    }
    fn node_with_type_not_found(node_type_name: &'static str) -> Self {
        MyError::NodeNotFound(node_type_name)
    }

    fn wrong_input_type_for_node(node_type_name: &'static str) -> Self {
        MyError::WrongNodeInput(node_type_name)
    }
}

impl OrchestratorError for MyError {
    fn no_matching_pipeline() -> Self {
        MyError::NoPipeline
    }
}

#[derive(Clone, Debug)]
struct Matcher;

impl Node for Matcher {
    type Input = String;
    type Output = String;
    type Error = MyError;

    fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
        if !input.contains("match") {
            return Self::soft_fail().into();
        }
        Self::advance(input).into()
    }
}

#[derive(Clone, Debug)]
struct Downloader;

impl Node for Downloader {
    type Input = String;
    type Output = String;
    type Error = MyError;

    fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
        Self::advance(input).into()
    }
}

#[derive(Clone, Debug)]
struct Parser {
    times: usize,
}

impl Node for Parser {
    type Input = String;
    type Output = String;
    type Error = MyError;

    fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
        if self.times == 0 {
            return Self::return_from_pipeline(input).into();
        }
        self.times -= 1;
        Self::pipe_to::<Downloader>(input).into()
    }
}

#[derive(Clone, Debug)]
struct NotDoingIt;

impl Node for NotDoingIt {
    type Input = ();
    type Output = ();
    type Error = MyError;

    fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
        Self::return_from_pipeline(input).into()
    }
}

#[test]
fn pipeline_success() {
    let pipeline = Pipeline::<String, String, MyError>::start(Matcher)
        .add_node(Downloader)
        .finish(Parser { times: 3 });
    let res = pipeline.run("match".into());
    assert_eq!(res, Ok(PipelineOutput::Done("match".to_owned())));
}

#[test]
fn soft_fail() {
    let pipeline = Pipeline::<String, String, MyError>::start(Matcher)
        .add_node(Downloader)
        .finish(Parser { times: 3 });
    let res = pipeline.run("".into());
    assert_eq!(res, Ok(PipelineOutput::SoftFail));
}

#[test]
fn node_not_found() {
    let pipeline = Pipeline::<String, String, MyError>::start(Matcher).finish(Parser { times: 3 });
    let res = pipeline.run("match".into());
    assert_eq!(res, Err(MyError::NodeNotFound("functionality::Downloader")));
}

#[test]
fn wrong_input() {
    let pipeline = Pipeline::<String, (), MyError>::start(Matcher).finish(NotDoingIt);
    let res = pipeline.run("match".into());
    assert_eq!(res, Err(MyError::WrongNodeInput("functionality::NotDoingIt")))
}

#[test]
fn wrong_output() {
    let pipeline = Pipeline::<(), String, MyError>::start(NotDoingIt).finish(Matcher);
    let res = pipeline.run(());
    assert_eq!(res, Err(MyError::WrongPipelineOutput))
}


#[test]
fn orchestrator_success() {
    let pipeline = Pipeline::<String, String, MyError>::start(Matcher)
        .add_node(Downloader)
        .finish(Parser { times: 3 });
    let mut orchestrator = GeneralOrchestrator::new();
    orchestrator.add_pipeline(pipeline);
    let res = orchestrator.run("match".into());
    assert_eq!(res, Ok("match".to_owned()));
}

#[test]
fn orchestrator_no_pipeline() {
    let pipeline = Pipeline::<String, String, MyError>::start(Matcher)
        .add_node(Downloader)
        .finish(Parser { times: 3 });
    let mut orchestrator = GeneralOrchestrator::new();
    orchestrator.add_pipeline(pipeline);
    let res = orchestrator.run("".into());
    assert_eq!(res, Err(MyError::NoPipeline));
}