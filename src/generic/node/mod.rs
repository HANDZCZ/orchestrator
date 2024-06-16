mod node_trait;
mod output;
mod returnable;

/// Module that houses stuff related to [`FnNode`](crate::generic::node::fn_node::FnNode).
pub mod fn_node;
pub use node_trait::*;
pub use output::*;
pub use returnable::*;
