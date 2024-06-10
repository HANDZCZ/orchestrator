use crate::generic::AnyDebug;

/// Defines which errors can occur in [`GenericPipeline`](crate::generic::pipeline::GenericPipeline).
#[derive(Debug)]
pub enum PipelineError {
    /// Output that is supposed to be returned from pipeline has wrong type.
    WrongOutputTypeForPipeline {
        /// Name of the node which returned the wrong type.
        node_type_name: &'static str,
        /// Data that couldn't be downcasted to pipeline output type.
        ///
        /// You can format them with debug flag!
        data: Box<dyn AnyDebug>,
        /// Name of the type that was expected.
        ///
        /// It's the name of the type you set as the pipeline output.
        expected_type_name: &'static str,
        /// Name of the type that was actually delivered.
        got_type_name: &'static str,
    },
    /// [`Node`](crate::generic::node::Node) with specified type could not be found in pipeline.
    /// Most likely you just forgot to add it to the pipeline.
    NodeWithTypeNotFound {
        /// Name of the node which could not be found.
        node_type_name: &'static str,
    },
}
