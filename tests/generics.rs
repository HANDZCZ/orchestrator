use std::fmt::Debug;

use async_trait::async_trait;
use orchestrator::{
    generic::{
        node::{Node, NodeOutput, Returnable},
        orchestrator::{GenericOrchestrator, OrchestratorError},
        pipeline::{GenericPipeline, PipelineError, PipelineOutput},
    },
    orchestrator::Orchestrator,
    pipeline::Pipeline,
};

#[derive(Debug, PartialEq)]
enum MyOrchestratorError {
    PipelineError(MyPipelineError),
    AllPipelinesSoftFailed,
}

impl From<OrchestratorError> for MyOrchestratorError {
    fn from(value: OrchestratorError) -> Self {
        match value {
            OrchestratorError::AllPipelinesSoftFailed => Self::AllPipelinesSoftFailed,
        }
    }
}

impl From<MyPipelineError> for MyOrchestratorError {
    fn from(value: MyPipelineError) -> Self {
        Self::PipelineError(value)
    }
}

#[derive(Debug, PartialEq)]
enum MyPipelineError {
    WrongPipelineOutput,
    NodeNotFound(&'static str),
    WrongNodeInput(&'static str),
    SomeNodeError,
}

impl From<PipelineError> for MyPipelineError {
    fn from(value: PipelineError) -> Self {
        match value {
            PipelineError::WrongInputTypeForNode { node_type_name } => {
                Self::WrongNodeInput(node_type_name)
            }
            PipelineError::WrongOutputTypeForPipeline => Self::WrongPipelineOutput,
            PipelineError::NodeWithTypeNotFound { node_type_name } => {
                Self::NodeNotFound(node_type_name)
            }
        }
    }
}

impl From<MatcherError> for MyPipelineError {
    fn from(_value: MatcherError) -> Self {
        Self::SomeNodeError
    }
}

impl From<DownloaderError> for MyPipelineError {
    fn from(_value: DownloaderError) -> Self {
        Self::SomeNodeError
    }
}

impl From<ParserError> for MyPipelineError {
    fn from(_value: ParserError) -> Self {
        Self::SomeNodeError
    }
}

impl From<NotDoingItError> for MyPipelineError {
    fn from(_value: NotDoingItError) -> Self {
        Self::SomeNodeError
    }
}

#[derive(Clone, Debug)]
struct Matcher;
#[derive(Debug)]
struct MatcherError;

#[async_trait]
impl Node for Matcher {
    type Input = String;
    type Output = String;
    type Error = MatcherError;

    async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
        if !input.contains("match") {
            return Self::soft_fail().into();
        }
        Self::advance(input).into()
    }
}

#[derive(Clone, Debug)]
struct Downloader;
#[derive(Debug)]
struct DownloaderError;

#[async_trait]
impl Node for Downloader {
    type Input = String;
    type Output = String;
    type Error = DownloaderError;

    async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
        Self::advance(input).into()
    }
}

#[derive(Clone, Debug)]
struct Parser {
    times: usize,
}
#[derive(Debug)]
struct ParserError;

#[async_trait]
impl Node for Parser {
    type Input = String;
    type Output = String;
    type Error = ParserError;

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
#[derive(Debug)]
struct NotDoingItError;

#[async_trait]
impl Node for NotDoingIt {
    type Input = ();
    type Output = ();
    type Error = NotDoingItError;

    async fn run(&mut self, input: Self::Input) -> Result<NodeOutput<Self::Output>, Self::Error> {
        Self::return_from_pipeline(input).into()
    }
}

#[tokio::test]
async fn pipeline_success() {
    let pipeline = GenericPipeline::<String, String, MyPipelineError>::start(Matcher)
        .add_node(Downloader)
        .finish(Parser { times: 3 });
    let res = pipeline.run("match".into()).await;
    assert_eq!(res, Ok(PipelineOutput::Done("match".to_owned())));
}

#[tokio::test]
async fn soft_fail() {
    let pipeline = GenericPipeline::<String, String, MyPipelineError>::start(Matcher)
        .add_node(Downloader)
        .finish(Parser { times: 3 });
    let res = pipeline.run("".into()).await;
    assert_eq!(res, Ok(PipelineOutput::SoftFail));
}

#[tokio::test]
async fn node_not_found() {
    let pipeline = GenericPipeline::<String, String, MyPipelineError>::start(Matcher)
        .finish(Parser { times: 3 });
    let res = pipeline.run("match".into()).await;
    assert_eq!(
        res,
        Err(MyPipelineError::NodeNotFound("generics::Downloader"))
    );
}

#[tokio::test]
async fn wrong_input() {
    let pipeline =
        GenericPipeline::<String, (), MyPipelineError>::start(Matcher).finish(NotDoingIt);
    let res = pipeline.run("match".into()).await;
    assert_eq!(
        res,
        Err(MyPipelineError::WrongNodeInput("generics::NotDoingIt"))
    )
}

#[tokio::test]
async fn wrong_output() {
    let pipeline =
        GenericPipeline::<(), String, MyPipelineError>::start(NotDoingIt).finish(Matcher);
    let res = pipeline.run(()).await;
    assert_eq!(res, Err(MyPipelineError::WrongPipelineOutput))
}

#[tokio::test]
async fn orchestrator_success() {
    let pipeline = GenericPipeline::<String, String, MyPipelineError>::start(Matcher)
        .add_node(Downloader)
        .finish(Parser { times: 3 });
    let mut orchestrator: GenericOrchestrator<String, String, MyOrchestratorError> =
        GenericOrchestrator::new();
    orchestrator.add_pipeline(pipeline);
    let res = orchestrator.run("match".into()).await;
    assert_eq!(res, Ok("match".to_owned()));
}

#[tokio::test]
async fn orchestrator_no_pipeline() {
    let pipeline = GenericPipeline::<String, String, MyPipelineError>::start(Matcher)
        .add_node(Downloader)
        .finish(Parser { times: 3 });
    let mut orchestrator: GenericOrchestrator<String, String, MyOrchestratorError> =
        GenericOrchestrator::new();
    orchestrator.add_pipeline(pipeline);
    let res = orchestrator.run("".into()).await;
    assert_eq!(res, Err(MyOrchestratorError::AllPipelinesSoftFailed));
}