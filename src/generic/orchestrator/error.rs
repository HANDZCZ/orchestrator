/// Defines which errors can occur in [`GenericOrchestrator`](crate::generic::orchestrator::GenericOrchestrator).
#[derive(Debug)]
pub enum OrchestratorError {
    /// All the pipelines added to orchestrator soft failed.
    ///
    /// For more information about soft fail look at [`NodeOutput::SoftFail`](crate::generic::node::NodeOutput::SoftFail).
    AllPipelinesSoftFailed,
}
