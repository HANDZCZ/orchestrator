use std::any;

use orchestrator::{
    async_trait,
    generic::{
        node::{
            fn_node::{FnNode, FnNodeFutureExt, FnOutput},
            AnyNode, Node, NodeOutput, Returnable,
        },
        orchestrator::{GenericOrchestrator, OrchestratorError},
        pipeline::{FnPipeline, GenericPipeline, PipelineError, PipelineOutput, PipelineStorage},
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
    WrongPipelineOutput {
        node_type_name: &'static str,
        expected_type_name: &'static str,
        got_type_name: &'static str,
    },
    NodeNotFound(&'static str),
    SomeNodeError,
}

impl From<PipelineError> for MyPipelineError {
    fn from(value: PipelineError) -> Self {
        match value {
            PipelineError::WrongOutputTypeForPipeline {
                node_type_name,
                expected_type_name,
                got_type_name,
                ..
            } => Self::WrongPipelineOutput {
                expected_type_name,
                got_type_name,
                node_type_name,
            },
            PipelineError::NodeWithTypeNotFound { node_type_name } => {
                Self::NodeNotFound(node_type_name)
            }
        }
    }
}

impl From<StringMatcherError> for MyPipelineError {
    fn from(_value: StringMatcherError) -> Self {
        Self::SomeNodeError
    }
}

impl From<StringForwarderError> for MyPipelineError {
    fn from(_value: StringForwarderError) -> Self {
        Self::SomeNodeError
    }
}

impl From<RepeatPipeToStringForwarderError> for MyPipelineError {
    fn from(_value: RepeatPipeToStringForwarderError) -> Self {
        Self::SomeNodeError
    }
}

impl From<NodeEarlyReturnError> for MyPipelineError {
    fn from(_value: NodeEarlyReturnError) -> Self {
        Self::SomeNodeError
    }
}

#[derive(Clone, Debug)]
struct StringMatcher(&'static str);
#[derive(Debug)]
struct StringMatcherError;

#[async_trait]
impl Node for StringMatcher {
    type Input = String;
    type Output = String;
    type Error = StringMatcherError;

    async fn run(
        &mut self,
        input: Self::Input,
        _pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error> {
        if !input.contains(self.0) {
            return Self::soft_fail().into();
        }
        Self::advance(input).into()
    }
}

#[derive(Clone, Debug)]
struct StringForwarder;
#[derive(Debug)]
struct StringForwarderError;

#[async_trait]
impl Node for StringForwarder {
    type Input = String;
    type Output = String;
    type Error = StringForwarderError;

    async fn run(
        &mut self,
        input: Self::Input,
        _pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error> {
        Self::advance(input).into()
    }
}

#[derive(Clone, Debug)]
struct RepeatPipeToStringForwarder {
    times: usize,
}
#[derive(Debug)]
struct RepeatPipeToStringForwarderError;

#[async_trait]
impl Node for RepeatPipeToStringForwarder {
    type Input = String;
    type Output = String;
    type Error = RepeatPipeToStringForwarderError;

    async fn run(
        &mut self,
        input: Self::Input,
        _pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error> {
        if self.times == 0 {
            return Self::return_from_pipeline(input).into();
        }
        self.times -= 1;
        Self::pipe_to::<StringForwarder>(input).into()
    }
}

#[derive(Clone, Debug)]
struct UnitToStringEarlyReturnString;
#[derive(Debug)]
struct NodeEarlyReturnError;

#[async_trait]
impl Node for UnitToStringEarlyReturnString {
    type Input = ();
    type Output = String;
    type Error = NodeEarlyReturnError;

    async fn run(
        &mut self,
        _input: Self::Input,
        _pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error> {
        Self::return_from_pipeline("".to_string()).into()
    }
}
#[derive(Clone, Debug)]
struct StringToUnitEarlyReturnUnit;

#[async_trait]
impl Node for StringToUnitEarlyReturnUnit {
    type Input = String;
    type Output = ();
    type Error = NodeEarlyReturnError;

    async fn run(
        &mut self,
        _input: Self::Input,
        _pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error> {
        Self::return_from_pipeline(()).into()
    }
}

#[tokio::test]
async fn pipeline_success() {
    let pipeline = GenericPipeline::<String, String, MyPipelineError>::new()
        .add_node(StringMatcher("match"))
        .add_node(StringForwarder)
        .add_node(RepeatPipeToStringForwarder { times: 3 })
        .finish();
    let res = pipeline.run("match".into()).await;
    assert_eq!(res, Ok(PipelineOutput::Done("match".to_owned())));
}

#[derive(Clone, Debug)]
struct StringToIsOk;

#[async_trait]
impl Node for StringToIsOk {
    type Input = String;
    type Output = IsOk;
    type Error = NodeEarlyReturnError;

    async fn run(
        &mut self,
        input: Self::Input,
        _pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error> {
        Self::advance(IsOk(input)).into()
    }
}

#[derive(Clone, Debug)]
struct IsOkToString;

#[async_trait]
impl Node for IsOkToString {
    type Input = IsOk;
    type Output = String;
    type Error = NodeEarlyReturnError;

    async fn run(
        &mut self,
        input: Self::Input,
        _pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<Self::Output>, Self::Error> {
        Self::advance(input.0).into()
    }
}

#[tokio::test]
async fn pipeline_io_conversion_success() {
    let pipeline = GenericPipeline::<String, String, MyPipelineError>::new()
        .add_node(StringToIsOk)
        .add_node(StringForwarder)
        .add_node(IsOkToString)
        .add_node(RepeatPipeToStringForwarder { times: 3 })
        .finish();
    let res = pipeline.run("match".into()).await;
    assert_eq!(res, Ok(PipelineOutput::Done("match".to_owned())));
}

#[tokio::test]
async fn soft_fail() {
    let pipeline = GenericPipeline::<String, String, MyPipelineError>::new()
        .add_node(StringMatcher("nomatch"))
        .add_node(StringForwarder)
        .add_node(RepeatPipeToStringForwarder { times: 3 })
        .finish();
    let res = pipeline.run("".into()).await;
    assert_eq!(res, Ok(PipelineOutput::SoftFail));
}

#[tokio::test]
async fn node_not_found() {
    let pipeline = GenericPipeline::<String, String, MyPipelineError>::new()
        .add_node(StringMatcher(""))
        .add_node(RepeatPipeToStringForwarder { times: 3 })
        .finish();
    let res = pipeline.run("match".into()).await;
    assert_eq!(
        res,
        Err(MyPipelineError::NodeNotFound(any::type_name::<
            StringForwarder,
        >()))
    );
}

#[tokio::test]
async fn wrong_output() {
    let pipeline = GenericPipeline::<(), (), MyPipelineError>::new()
        .add_node(UnitToStringEarlyReturnString)
        .add_node(StringToUnitEarlyReturnUnit)
        .finish();
    let res = pipeline.run(()).await;
    assert_eq!(
        res,
        Err(MyPipelineError::WrongPipelineOutput {
            expected_type_name: any::type_name::<()>(),
            got_type_name: any::type_name::<String>(),
            node_type_name: any::type_name::<UnitToStringEarlyReturnString>()
        })
    )
}

#[tokio::test]
async fn orchestrator_success() {
    let pipeline = GenericPipeline::<String, String, MyPipelineError>::new()
        .add_node(StringMatcher("match"))
        .add_node(StringForwarder)
        .add_node(RepeatPipeToStringForwarder { times: 3 })
        .finish();
    let mut orchestrator: GenericOrchestrator<String, String, MyOrchestratorError> =
        GenericOrchestrator::new();
    orchestrator.add_pipeline(pipeline);
    let res = orchestrator.run("match".into()).await;
    assert_eq!(res, Ok("match".to_owned()));
}

#[derive(Debug, Clone, PartialEq)]
struct IsOk(String);

impl From<String> for IsOk {
    fn from(value: String) -> Self {
        IsOk(value)
    }
}
impl From<IsOk> for String {
    fn from(value: IsOk) -> Self {
        value.0
    }
}

#[tokio::test]
async fn orchestrator_io_conversion_success() {
    let pipeline = GenericPipeline::<String, String, MyPipelineError>::new()
        .add_node(StringMatcher("match"))
        .add_node(StringForwarder)
        .add_node(RepeatPipeToStringForwarder { times: 3 })
        .finish();
    let mut orchestrator: GenericOrchestrator<IsOk, IsOk, MyOrchestratorError> =
        GenericOrchestrator::new();
    orchestrator.add_pipeline(pipeline);
    let res = orchestrator.run(IsOk("match".into())).await;
    assert_eq!(res, Ok(IsOk("match".to_owned())));
}

#[tokio::test]
async fn orchestrator_no_pipeline() {
    let pipeline = GenericPipeline::<String, String, MyPipelineError>::new()
        .add_node(StringMatcher("nomatch"))
        .add_node(StringForwarder)
        .add_node(RepeatPipeToStringForwarder { times: 3 })
        .finish();
    let mut orchestrator: GenericOrchestrator<String, String, MyOrchestratorError> =
        GenericOrchestrator::new();
    orchestrator.add_pipeline(pipeline);
    let res = orchestrator.run("".into()).await;
    assert_eq!(res, Err(MyOrchestratorError::AllPipelinesSoftFailed));
}

impl From<()> for MyPipelineError {
    fn from(_value: ()) -> Self {
        MyPipelineError::SomeNodeError
    }
}

#[tokio::test]
async fn fn_node_test() {
    fn normal_async_fn(input: IsOk, _pipeline_storage: &mut PipelineStorage) -> FnOutput<IsOk, ()> {
        async { AnyNode::advance(input).into() }.into_fn_output()
    }
    let normal_async_fn = FnNode::new(normal_async_fn);

    async fn normal_async_fn_with_sugar(
        input: IsOk,
        _pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<IsOk>, ()> {
        AnyNode::advance(input).into()
    }
    let normal_async_fn_with_sugar =
        FnNode::new(|a, b| async move { normal_async_fn_with_sugar(a, b).await }.into_fn_output());

    let closure_with_async =
        FnNode::new(|input: String, _pipeline_storage: &mut PipelineStorage| {
            async move {
                if !input.is_empty() {
                    AnyNode::advance(input).into()
                } else {
                    Err(())
                }
            }
            .into_fn_output()
        });

    let pipeline = GenericPipeline::<String, String, MyPipelineError>::new()
        .add_node(normal_async_fn_with_sugar)
        .add_node(closure_with_async)
        .add_node(normal_async_fn)
        .finish();

    let input = "Not empty".to_string();
    let res = pipeline.run(input.clone()).await;
    assert_eq!(res, Ok(PipelineOutput::Done(input)));
}

impl From<()> for MyOrchestratorError {
    fn from(_value: ()) -> Self {
        MyOrchestratorError::PipelineError(MyPipelineError::SomeNodeError)
    }
}

#[tokio::test]
async fn fn_pipeline_test() {
    async fn normal_async_fn(input: IsOk) -> Result<PipelineOutput<IsOk>, ()> {
        Ok(PipelineOutput::Done(input))
    }

    let closure_with_async = |input: String| async {
        if !input.is_empty() {
            Ok(PipelineOutput::Done(input))
        } else {
            Err(())
        }
    };

    let mut orchestrator = GenericOrchestrator::<String, String, MyOrchestratorError>::new();
    orchestrator.add_pipeline(FnPipeline::new(closure_with_async));
    orchestrator.add_pipeline(FnPipeline::new(normal_async_fn));

    let input = "Not empty".to_string();
    let res = orchestrator.run(input.clone()).await;
    assert_eq!(res, Ok(input));
}

#[tokio::test]
async fn pipeline_storage_test() {
    #[derive(Debug)]
    struct MyVal(usize);

    #[derive(Clone, Debug)]
    struct PipelineStorageTest;

    #[async_trait]
    impl Node for PipelineStorageTest {
        type Input = ();
        type Output = ();
        type Error = ();

        async fn run(
            &mut self,
            input: Self::Input,
            pipeline_storage: &mut PipelineStorage,
        ) -> Result<NodeOutput<Self::Output>, Self::Error> {
            let my_val = pipeline_storage.get_mut::<MyVal>();
            let my_val = my_val.unwrap();
            *my_val = MyVal(my_val.0 + 1);
            Self::advance(input).into()
        }
    }

    let first_node =
        FnNode::<(), (), (), _>::new(|input: (), pipeline_storage: &mut PipelineStorage| {
            async move {
                pipeline_storage.insert(MyVal(0));
                AnyNode::advance(input).into()
            }
            .into_fn_output()
        });

    async fn middle_node(
        input: (),
        pipeline_storage: &mut PipelineStorage,
    ) -> Result<NodeOutput<()>, ()> {
        let my_val = pipeline_storage.get_mut::<MyVal>();
        let my_val = my_val.unwrap();
        *my_val = MyVal(my_val.0 + 1);
        AnyNode::advance(input).into()
    }
    let middle_node = FnNode::new(|a, b| async move { middle_node(a, b).await }.into_fn_output());

    fn last_node(_input: (), pipeline_storage: &mut PipelineStorage) -> FnOutput<usize, ()> {
        async move {
            let my_val = pipeline_storage.remove::<MyVal>();
            let my_val = my_val.unwrap();
            AnyNode::advance(my_val.0).into()
        }
        .into_fn_output()
    }
    let last_node = FnNode::new(last_node);

    let pipeline = GenericPipeline::<(), usize, MyPipelineError>::new()
        .add_node(first_node)
        .add_node(middle_node)
        .add_node(PipelineStorageTest)
        .add_node(last_node)
        .finish();

    let res = pipeline.run(()).await;
    assert_eq!(res, Ok(PipelineOutput::Done(2)));
}
