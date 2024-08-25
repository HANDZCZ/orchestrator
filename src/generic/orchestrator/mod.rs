mod error;
mod generic_orchestrator;
mod generic_orchestrator_as_node;
mod generic_orchestrator_as_pipeline;
mod internal_pipeline;
mod keyed_generic_orchestrator;

pub use error::*;
pub use generic_orchestrator::*;
pub use generic_orchestrator_as_node::*;
pub use generic_orchestrator_as_pipeline::*;
pub use keyed_generic_orchestrator::*;

use internal_pipeline::{InternalPipeline, InternalPipelineOutput};
type InternalPipelineType<Input, Output, Error> =
    Box<dyn InternalPipeline<Input, InternalPipelineOutput<Output>, Error>>;
