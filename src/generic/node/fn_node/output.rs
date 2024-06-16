use crate::generic::node::NodeOutput;
use std::{future::Future, pin::Pin};

pub type FnOutput<'a, Output, Error> =
    Pin<Box<dyn Future<Output = Result<NodeOutput<Output>, Error>> + Send + Sync + 'a>>;
