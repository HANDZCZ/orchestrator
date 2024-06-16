use super::FnOutput;
use crate::generic::node::NodeOutput;
use std::future::Future;

pub trait FnNodeFutureExt: Future {
    fn into_fn_output<'a, Output, Error>(self) -> FnOutput<'a, Output, Error>
    where
        Self: Sized + Send + Sync + 'a,
        Self: Future<Output = Result<NodeOutput<Output>, Error>>,
    {
        Box::pin(self)
    }
}
impl<T: ?Sized> FnNodeFutureExt for T where T: Future {}
