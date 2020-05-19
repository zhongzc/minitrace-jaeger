//! Thrift components defined in [jaeger.thrift].
//!
//! [jaeger.thrift]: https://github.com/uber/jaeger-idl/blob/master/thrift/jaeger.thrift

/// `Span` represents a named unit of work performed by a service.
#[derive(Debug, Clone)]
pub struct Span {
    /// The least significant 64 bits of a traceID.
    pub trace_id_low: i64,

    /// The most significant 64 bits of a traceID; 0 when only 64bit IDs are used.
    pub trace_id_high: i64,

    /// Unique span id (only unique within a given trace).
    pub span_id: i64,

    /// Since nearly all spans will have parents spans, `ChildOf` refs do not have to be explicit.
    ///
    /// Should be `0` if the current span is a root span.
    pub parent_span_id: i64,

    /// The name of operation.
    pub operation_name: String,

    /// Causal references to other spans.
    pub references: Vec<SpanRef>,

    /// A bit field used to propagate sampling decisions.
    ///
    /// `1` signifies a SAMPLED span, `2` signifies a DEBUG span.
    pub flags: i32,

    /// Start time of this span.
    pub start_time: i64,

    /// Duration of this span.
    pub duration: i64,

    /// Tag list.
    pub tags: Vec<Tag>,

    /// Log list.
    pub logs: Vec<Log>,
}

impl From<Span> for thrift_codec::data::Struct {
    fn from(f: Span) -> Self {
        let mut fields = Vec::with_capacity(11);
        fields.push(thrift_codec::data::Field::new(1, f.trace_id_low));
        fields.push(thrift_codec::data::Field::new(2, f.trace_id_high));
        fields.push(thrift_codec::data::Field::new(3, f.span_id));
        fields.push(thrift_codec::data::Field::new(4, f.parent_span_id));
        fields.push(thrift_codec::data::Field::new(5, f.operation_name));
        if !f.references.is_empty() {
            fields.push(thrift_codec::data::Field::new(
                6,
                thrift_codec::data::List::from(
                    f.references
                        .into_iter()
                        .map(thrift_codec::data::Struct::from)
                        .collect::<Vec<_>>(),
                ),
            ));
        }
        fields.push(thrift_codec::data::Field::new(7, f.flags));
        fields.push(thrift_codec::data::Field::new(8, f.start_time));
        fields.push(thrift_codec::data::Field::new(9, f.duration));
        if !f.tags.is_empty() {
            fields.push(thrift_codec::data::Field::new(
                10,
                thrift_codec::data::List::from(
                    f.tags
                        .into_iter()
                        .map(thrift_codec::data::Struct::from)
                        .collect::<Vec<_>>(),
                ),
            ));
        }
        if !f.logs.is_empty() {
            fields.push(thrift_codec::data::Field::new(
                11,
                thrift_codec::data::List::from(
                    f.logs
                        .into_iter()
                        .map(thrift_codec::data::Struct::from)
                        .collect::<Vec<_>>(),
                ),
            ));
        }
        thrift_codec::data::Struct::new(fields)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpanRefKind {
    ChildOf = 0,
    FollowsFrom = 1,
}

#[derive(Debug, Clone)]
pub struct SpanRef {
    pub kind: SpanRefKind,
    pub trace_id_low: i64,
    pub trace_id_high: i64,
    pub span_id: i64,
}

impl From<SpanRef> for thrift_codec::data::Struct {
    fn from(f: SpanRef) -> Self {
        thrift_codec::data::Struct::from((
            f.kind as i32,
            f.trace_id_low,
            f.trace_id_high,
            f.span_id,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagKind {
    String = 0,
    Double = 1,
    Bool = 2,
    Long = 3,
    Binary = 4,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Tag {
    String { key: String, value: String },
    Double { key: String, value: f64 },
    Bool { key: String, value: bool },
    Long { key: String, value: i64 },
    Binary { key: String, value: Vec<u8> },
}

impl From<Tag> for thrift_codec::data::Struct {
    fn from(f: Tag) -> Self {
        thrift_codec::data::Struct::new(match f {
            Tag::String { key, value } => vec![
                thrift_codec::data::Field::new(1, key),
                thrift_codec::data::Field::new(2, TagKind::String as i32),
                thrift_codec::data::Field::new(3, value),
            ],
            Tag::Double { key, value } => vec![
                thrift_codec::data::Field::new(1, key),
                thrift_codec::data::Field::new(2, TagKind::Double as i32),
                thrift_codec::data::Field::new(4, value),
            ],
            Tag::Bool { key, value } => vec![
                thrift_codec::data::Field::new(1, key),
                thrift_codec::data::Field::new(2, TagKind::Bool as i32),
                thrift_codec::data::Field::new(5, value),
            ],
            Tag::Long { key, value } => vec![
                thrift_codec::data::Field::new(1, key),
                thrift_codec::data::Field::new(2, TagKind::Long as i32),
                thrift_codec::data::Field::new(6, value),
            ],
            Tag::Binary { key, value } => vec![
                thrift_codec::data::Field::new(1, key),
                thrift_codec::data::Field::new(2, TagKind::Binary as i32),
                thrift_codec::data::Field::new(7, value),
            ],
        })
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    pub timestamp: i64,
    pub fields: Vec<Tag>,
}

impl From<Log> for thrift_codec::data::Struct {
    fn from(f: Log) -> Self {
        thrift_codec::data::Struct::from((
            f.timestamp,
            thrift_codec::data::List::from(
                f.fields
                    .into_iter()
                    .map(thrift_codec::data::Struct::from)
                    .collect::<Vec<_>>(),
            ),
        ))
    }
}

/// `Process` describes the traced process/service that emits spans.
#[derive(Debug, Clone)]
pub struct Process {
    /// The name of this service.
    pub service_name: String,

    /// Tag list.
    pub tags: Vec<Tag>,
}

impl From<Process> for thrift_codec::data::Struct {
    fn from(f: Process) -> Self {
        let tags = thrift_codec::data::List::from(
            f.tags
                .into_iter()
                .map(thrift_codec::data::Struct::from)
                .collect::<Vec<_>>(),
        );
        if tags.is_empty() {
            thrift_codec::data::Struct::from((f.service_name,))
        } else {
            thrift_codec::data::Struct::from((f.service_name, tags))
        }
    }
}

/// `Batch` is a collection of spans reported out of process.
#[derive(Debug, Clone)]
pub struct Batch {
    pub process: Process,
    pub spans: Vec<Span>,
}
impl From<Batch> for thrift_codec::data::Struct {
    fn from(f: Batch) -> Self {
        thrift_codec::data::Struct::from((
            thrift_codec::data::Struct::from(f.process),
            thrift_codec::data::List::from(
                f.spans
                    .into_iter()
                    .map(thrift_codec::data::Struct::from)
                    .collect::<Vec<_>>(),
            ),
        ))
    }
}
