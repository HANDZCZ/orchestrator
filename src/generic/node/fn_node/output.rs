use crate::generic::node::NodeOutput;
use std::{future::Future, pin::Pin};

/// Helper type that functions need to return to be able to be used in [`FnNode`](crate::generic::node::fn_node::FnNode).
pub type FnOutput<'a, Output, Error> =
    Pin<Box<dyn Future<Output = Result<NodeOutput<Output>, Error>> + Send + Sync + 'a>>;
