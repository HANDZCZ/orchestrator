/// Defines what pipelines return.
#[derive(Debug, PartialEq)]
pub enum PipelineOutput<T> {
    /// Says that the pipeline soft failed.
    ///
    /// For more information look at [`NodeOutput::SoftFail`](crate::generic::node::NodeOutput::SoftFail).
    SoftFail,
    /// Pipeline finished successfully.
    Done(T),
}
