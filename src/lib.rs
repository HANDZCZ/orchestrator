#![warn(missing_docs)]
#![warn(unused_doc_comments)]
#![warn(clippy::empty_docs)]
#![warn(clippy::tabs_in_doc_comments)]
#![warn(clippy::suspicious_doc_comments)]
#![warn(clippy::test_attr_in_doctest)]
#![warn(rustdoc::private_intra_doc_links)]
#![warn(clippy::empty_line_after_doc_comments)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![forbid(rustdoc::broken_intra_doc_links)]
#![forbid(missing_debug_implementations)]
#![forbid(invalid_doc_attributes)]
#![cfg_attr(docs_cfg, feature(async_fn_in_trait))]

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
/// Module that houses stuff related to [`Orchestrator`](crate::orchestrator::Orchestrator) trait.
pub mod orchestrator;
/// Module that houses stuff related to [`Pipeline`](crate::pipeline::Pipeline) trait.
pub mod pipeline;
pub use async_trait::async_trait;
