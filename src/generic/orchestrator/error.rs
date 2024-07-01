use std::{error::Error, fmt::Display};

/// Defines which errors can occur in [`GenericOrchestrator`](crate::generic::orchestrator::GenericOrchestrator).
#[derive(Debug)]
pub enum OrchestratorError {
    /// All the pipelines added to orchestrator soft failed.
    ///
    /// For more information about soft fail look at [`NodeOutput::SoftFail`](crate::generic::node::NodeOutput::SoftFail).
    AllPipelinesSoftFailed,
}

impl Display for OrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            OrchestratorError::AllPipelinesSoftFailed => "all pipelines in orchestrator failed",
        };
        f.write_str(txt)
    }
}

impl Error for OrchestratorError {}
