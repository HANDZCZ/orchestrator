use super::FnOutput;
use crate::generic::node::NodeOutput;
use std::future::Future;

/// Extension for [`Futures`](Future) that converts future to [`FnOutput`](crate::generic::node::fn_node::FnOutput).
pub trait FnNodeFutureExt: Future {
    /// Converts [`Future`] with output matching `Result<NodeOutput<some output type>, some error type>` to [`FnOutput`](crate::generic::node::fn_node::FnOutput) type.
    fn into_fn_output<'a, Output, Error>(self) -> FnOutput<'a, Output, Error>
    where
        Self: Sized + Send + Sync + 'a,
        Self: Future<Output = Result<NodeOutput<Output>, Error>>,
    {
        Box::pin(self)
    }
}
impl<T: ?Sized> FnNodeFutureExt for T where T: Future {}
