mod error;
mod fn_pipeline;
mod generic_pipeline;
mod generic_pipeline_as_node;
mod internal_node;
mod output;
mod pipeline_storage;

/// Module that houses stuff related to pipelines that were created from [`Orchestrator`](crate::orchestrator::Orchestrator).
pub mod squash_pipelines;
pub use error::*;
pub use fn_pipeline::*;
pub use generic_pipeline::*;
pub use generic_pipeline_as_node::*;
pub use output::*;
pub use pipeline_storage::*;
