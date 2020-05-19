//! Thrift components defined in [agent.thrift].
//!
//! [agent.thrift]: https://github.com/uber/jaeger-idl/blob/master/thrift/agent.thrift.

/// `emitBatch` message defined in [agent.thrift].
///
/// [agent.thrift]: https://github.com/uber/jaeger-idl/blob/master/thrift/agent.thrift]
#[derive(Debug, Clone)]
pub struct EmitBatchNotification {
    /// `batch` argument.
    pub batch: crate::thrift::jaeger::Batch,
}

impl From<EmitBatchNotification> for thrift_codec::message::Message {
    fn from(f: EmitBatchNotification) -> Self {
        thrift_codec::message::Message::oneway(
            "emitBatch",
            0,
            thrift_codec::data::Struct::from((thrift_codec::data::Struct::from(f.batch),)),
        )
    }
}
