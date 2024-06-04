use std::fmt::Debug;

use async_trait::async_trait;
use orchestrator::{
    node::{Node, NodeOutput, Returnable},
    orchestrator::{GeneralOrchestrator, Orchestrator, OrchestratorError},
    pipeline::{Pipeline, PipelineError, PipelineOutput},
};

#[derive(Debug, PartialEq)]
enum MyError {
    WrongPipelineOutput,
    NodeNotFound(&'static str),
    WrongNodeInput(&'static str),
    NoPipelineFinished,
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
    fn no_pipeline_finished() -> Self {
        MyError::NoPipelineFinished
    }
}

#[derive(Clone, Debug)]
struct Matcher;

#[async_trait]
impl Node for Matcher {
    type Input = String;
    type Output = String;
    type Error = MyError;

    async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
        if !input.contains("match") {
            return Self::soft_fail().into();
        }
        Self::advance(input).into()
    }
}

#[derive(Clone, Debug)]
struct Downloader;

#[async_trait]
impl Node for Downloader {
    type Input = String;
    type Output = String;
    type Error = MyError;

    async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
        Self::advance(input).into()
    }
}

#[derive(Clone, Debug)]
struct Parser {
    times: usize,
}

#[async_trait]
impl Node for Parser {
    type Input = String;
    type Output = String;
    type Error = MyError;

    async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
        if self.times == 0 {
            return Self::return_from_pipeline(input).into();
        }
        self.times -= 1;
        Self::pipe_to::<Downloader>(input).into()
    }
}

#[derive(Clone, Debug)]
struct NotDoingIt;

#[async_trait]
impl Node for NotDoingIt {
    type Input = ();
    type Output = ();
    type Error = MyError;

    async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
        Self::return_from_pipeline(input).into()
    }
}

#[tokio::test]
async fn pipeline_success() {
    let pipeline = Pipeline::<String, String, MyError>::start(Matcher)
        .add_node(Downloader)
        .finish(Parser { times: 3 });
    let res = pipeline.run("match".into()).await;
    assert_eq!(res, Ok(PipelineOutput::Done("match".to_owned())));
}

#[tokio::test]
async fn soft_fail() {
    let pipeline = Pipeline::<String, String, MyError>::start(Matcher)
        .add_node(Downloader)
        .finish(Parser { times: 3 });
    let res = pipeline.run("".into()).await;
    assert_eq!(res, Ok(PipelineOutput::SoftFail));
}

#[tokio::test]
async fn node_not_found() {
    let pipeline = Pipeline::<String, String, MyError>::start(Matcher).finish(Parser { times: 3 });
    let res = pipeline.run("match".into()).await;
    assert_eq!(res, Err(MyError::NodeNotFound("functionality::Downloader")));
}

#[tokio::test]
async fn wrong_input() {
    let pipeline = Pipeline::<String, (), MyError>::start(Matcher).finish(NotDoingIt);
    let res = pipeline.run("match".into()).await;
    assert_eq!(
        res,
        Err(MyError::WrongNodeInput("functionality::NotDoingIt"))
    )
}

#[tokio::test]
async fn wrong_output() {
    let pipeline = Pipeline::<(), String, MyError>::start(NotDoingIt).finish(Matcher);
    let res = pipeline.run(()).await;
    assert_eq!(res, Err(MyError::WrongPipelineOutput))
}

#[tokio::test]
async fn orchestrator_success() {
    let pipeline = Pipeline::<String, String, MyError>::start(Matcher)
        .add_node(Downloader)
        .finish(Parser { times: 3 });
    let mut orchestrator = GeneralOrchestrator::new();
    orchestrator.add_pipeline(pipeline);
    let res = orchestrator.run("match".into()).await;
    assert_eq!(res, Ok("match".to_owned()));
}

#[tokio::test]
async fn orchestrator_no_pipeline() {
    let pipeline = Pipeline::<String, String, MyError>::start(Matcher)
        .add_node(Downloader)
        .finish(Parser { times: 3 });
    let mut orchestrator = GeneralOrchestrator::new();
    orchestrator.add_pipeline(pipeline);
    let res = orchestrator.run("".into()).await;
    assert_eq!(res, Err(MyError::NoPipelineFinished));
}
