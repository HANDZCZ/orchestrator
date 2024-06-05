use std::{marker::PhantomData, sync::RwLock};

use async_trait::async_trait;
use orchestrator::{
    generic::pipeline::PipelineOutput, orchestrator::Orchestrator, pipeline::Pipeline,
};

#[derive(Default)]
struct ForwardPipeline<T: Default> {
    _type: PhantomData<T>,
}

#[async_trait]
impl<T: Send + Sync + 'static + Default> Pipeline for ForwardPipeline<T> {
    type Input = T;
    type Output = PipelineOutput<T>;
    type Error = ();

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(PipelineOutput::Done(input))
    }
}

#[tokio::test]
async fn pipeline_success() {
    let input = "Nice";
    let pipeline = ForwardPipeline::default();
    let output = pipeline.run(input).await;
    assert_eq!(output, Ok(PipelineOutput::Done(input)));
}

#[derive(Default)]
struct FailPipeline<T: Default> {
    _type: PhantomData<T>,
}

#[async_trait]
impl<T: Send + Sync + 'static + Default> Pipeline for FailPipeline<T> {
    type Input = T;
    type Output = PipelineOutput<T>;
    type Error = ();

    async fn run(&self, _input: Self::Input) -> Result<Self::Output, Self::Error> {
        Err(())
    }
}

#[tokio::test]
async fn pipeline_fail() {
    let input = "Nice";
    let pipeline = FailPipeline::default();
    let output = pipeline.run(input).await;
    assert_eq!(output, Err(()));
}

#[derive(Default)]
struct SoftFailPipeline<T: Default> {
    _type: PhantomData<T>,
}

#[async_trait]
impl<T: Send + Sync + 'static + Default> Pipeline for SoftFailPipeline<T> {
    type Input = T;
    type Output = PipelineOutput<T>;
    type Error = ();

    async fn run(&self, _input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(PipelineOutput::SoftFail)
    }
}

#[derive(Debug, PartialEq)]
enum MyOrchestratorError<T> {
    AllPipelinesSoftFailed,
    PipelineError(T),
}

#[derive(Default)]
struct MyOrchestrator<T: Default> {
    _type: PhantomData<T>,
    pipelines: Vec<Box<dyn Pipeline<Input = T, Output = PipelineOutput<T>, Error = ()>>>,
    ran: RwLock<usize>,
}

#[async_trait]
impl<T: Send + Sync + 'static + Default + Clone> Orchestrator for MyOrchestrator<T> {
    type Input = T;
    type Output = T;
    type Error = MyOrchestratorError<()>;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        for pipeline in &self.pipelines {
            {
                let mut ran = self.ran.write().unwrap();
                *ran += 1;
            }
            match pipeline.run(input.clone()).await {
                Ok(PipelineOutput::SoftFail) => continue,
                Ok(PipelineOutput::Done(res)) => return Ok(res),
                Err(e) => return Err(MyOrchestratorError::PipelineError(e)),
            }
        }
        Err(MyOrchestratorError::AllPipelinesSoftFailed)
    }
}

#[tokio::test]
async fn orchestrator_success() {
    let input = "Nice";
    let mut orchestrator = MyOrchestrator::default();
    orchestrator
        .pipelines
        .push(Box::new(SoftFailPipeline::default()));
    orchestrator
        .pipelines
        .push(Box::new(SoftFailPipeline::default()));
    orchestrator
        .pipelines
        .push(Box::new(SoftFailPipeline::default()));
    orchestrator
        .pipelines
        .push(Box::new(SoftFailPipeline::default()));
    orchestrator
        .pipelines
        .push(Box::new(SoftFailPipeline::default()));
    orchestrator
        .pipelines
        .push(Box::new(ForwardPipeline::default()));
    let output = orchestrator.run(input).await;
    assert_eq!(output, Ok(input));
    let ran = orchestrator.ran.read().unwrap();
    assert_eq!(*ran, 6);
}

#[tokio::test]
async fn orchestrator_fail() {
    let input = "Nice";
    let mut orchestrator = MyOrchestrator::default();
    orchestrator
        .pipelines
        .push(Box::new(SoftFailPipeline::default()));
    orchestrator
        .pipelines
        .push(Box::new(SoftFailPipeline::default()));
    orchestrator
        .pipelines
        .push(Box::new(FailPipeline::default()));
    orchestrator
        .pipelines
        .push(Box::new(SoftFailPipeline::default()));
    orchestrator
        .pipelines
        .push(Box::new(SoftFailPipeline::default()));
    orchestrator
        .pipelines
        .push(Box::new(ForwardPipeline::default()));
    let output = orchestrator.run(input).await;
    assert_eq!(output, Err(MyOrchestratorError::PipelineError(())));
    let ran = orchestrator.ran.read().unwrap();
    assert_eq!(*ran, 3);
}

#[tokio::test]
async fn orchestrator_all_pipeline_soft_fail() {
    let input = "Nice";
    let mut orchestrator = MyOrchestrator::default();
    orchestrator
        .pipelines
        .push(Box::new(SoftFailPipeline::default()));
    orchestrator
        .pipelines
        .push(Box::new(SoftFailPipeline::default()));
    orchestrator
        .pipelines
        .push(Box::new(SoftFailPipeline::default()));
    let output = orchestrator.run(input).await;
    assert_eq!(output, Err(MyOrchestratorError::AllPipelinesSoftFailed));
    let ran = orchestrator.ran.read().unwrap();
    assert_eq!(*ran, 3);
}
