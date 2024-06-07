#![warn(missing_docs)]
#![forbid(rustdoc::broken_intra_doc_links)]
#![forbid(missing_debug_implementations)]

//! A crate that provides pipelines and an orchestrator.
//!
//! It provides traits [`Pipeline`](crate::pipeline::Pipeline) and [`Orchestrator`](crate::orchestrator::Orchestrator).
//! It also provide generic implementations ([`GenericPipeline`](crate::generic::pipeline::GenericPipeline),
//! [`GenericOrchestrator`](crate::generic::orchestrator::GenericOrchestrator))
//! of these traits.
//! To use the generic types you need to implement the [`Node`](crate::generic::node::Node) trait and mostly follow the compilers nagging.

/// Module that houses stuff related to generics ([`Node`](crate::generic::node::Node),
/// [`GenericPipeline`](crate::generic::pipeline::GenericPipeline),
/// [`GenericOrchestrator`](crate::generic::orchestrator::GenericOrchestrator)).
pub mod generic;
/// Module that houses stuff related to [`Orchestrator`](crate::orchestrator::Orchestrator) trait
pub mod orchestrator;
/// Module that houses stuff related to [`Pipeline`](crate::pipeline::Pipeline) type
pub mod pipeline;
pub use async_trait::async_trait;
