use crate::Result;
use rand::prelude::*;
use thrift_codec::{BinaryEncode, CompactEncode};

/// Reporter for the agent which accepts jaeger.thrift over compact thrift protocol.
#[derive(Debug)]
pub struct JaegerCompactReporter(JaegerReporter);

impl JaegerCompactReporter {
    /// Makes a new `JaegerCompactReporter` instance.
    ///
    /// If the UDP socket used to report spans can not be bound to `0.0.0.0:0`,
    pub fn new(service_name: &str) -> Result<Self> {
        let inner = JaegerReporter::new(service_name, 6831)?;
        Ok(JaegerCompactReporter(inner))
    }

    /// Sets the address of the report destination agent to `addr`.
    ///
    /// The default address is `127.0.0.1:6831`.
    pub fn set_agent_addr(&mut self, addr: std::net::SocketAddr) -> Result<()> {
        self.0.set_agent_addr(addr)
    }

    /// Reports `spans`.
    pub fn report<F>(&self, spans: &[minitrace::Span], tag_display: F) -> Result<()>
    where
        F: Fn(u32) -> String,
    {
        self.0.report(
            spans,
            |message| {
                let mut bytes = Vec::new();
                message.compact_encode(&mut bytes)?;
                Ok(bytes)
            },
            tag_display,
        )
    }
}

/// Reporter for the agent which accepts jaeger.thrift over binary thrift protocol.
#[derive(Debug)]
pub struct JaegerBinaryReporter(JaegerReporter);

impl JaegerBinaryReporter {
    /// Makes a new `JaegerBinaryReporter` instance.
    pub fn new(service_name: &str) -> Result<Self> {
        let inner = JaegerReporter::new(service_name, 6832)?;
        Ok(JaegerBinaryReporter(inner))
    }

    /// Sets the address of the report destination agent to `addr`.
    ///
    /// The default address is `127.0.0.1:6832`.
    pub fn set_agent_addr(&mut self, addr: std::net::SocketAddr) -> Result<()> {
        self.0.set_agent_addr(addr)
    }

    /// Reports `spans`.
    pub fn report<F>(&self, spans: &[minitrace::Span], tag_display: F) -> Result<()>
    where
        F: Fn(u32) -> String,
    {
        self.0.report(
            spans,
            |message| {
                let mut bytes = Vec::new();
                message.binary_encode(&mut bytes)?;
                Ok(bytes)
            },
            tag_display,
        )
    }
}

#[derive(Debug)]
struct JaegerReporter {
    socket: std::net::UdpSocket,
    agent: std::net::SocketAddr,
    process: crate::thrift::jaeger::Process,
}

impl JaegerReporter {
    fn new(service_name: &str, port: u16) -> Result<Self> {
        let agent = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        let socket = udp_socket(agent)?;

        let process = crate::thrift::jaeger::Process {
            service_name: service_name.to_owned(),
            tags: Vec::new(),
        };

        let reporter = JaegerReporter {
            socket,
            agent,
            process,
        };

        // reporter.process.tags.push(crate::thrift::jaeger::Tag::XXX(key, value));

        Ok(reporter)
    }

    fn set_agent_addr(&mut self, addr: std::net::SocketAddr) -> Result<()> {
        self.socket = udp_socket(addr)?;
        self.agent = addr;

        Ok(())
    }

    fn report<F, TF>(&self, spans: &[minitrace::Span], encode: F, tag_display: TF) -> Result<()>
    where
        F: FnOnce(thrift_codec::message::Message) -> Result<Vec<u8>>,
        TF: Fn(u32) -> String,
    {
        let batch = crate::thrift::jaeger::Batch {
            process: self.process.clone(),
            spans: map_span(spans, tag_display)?,
        };
        let message =
            thrift_codec::message::Message::from(crate::thrift::agent::EmitBatchNotification {
                batch,
            });
        let bytes = encode(message)?;
        self.socket.send_to(&bytes, self.agent)?;
        Ok(())
    }
}

fn udp_socket(agent: std::net::SocketAddr) -> Result<std::net::UdpSocket> {
    Ok(std::net::UdpSocket::bind({
        if agent.is_ipv6() {
            std::net::SocketAddr::new(
                std::net::IpAddr::V6(std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)),
                0,
            )
        } else {
            std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), 0)
        }
    })?)
}

fn map_span<F>(
    spans: &[minitrace::Span],
    tag_display: F,
) -> Result<Vec<crate::thrift::jaeger::Span>>
where
    F: Fn(u32) -> String,
{
    let mut rng = rand::thread_rng();
    let trace_id_low: i64 = rng.gen();
    let trace_id_high: i64 = rng.gen();

    // root
    let start_time_ms = spans
        .iter()
        .find_map(|span| {
            if let minitrace::Link::Root { start_time_ms } = span.link {
                Some(start_time_ms)
            } else {
                None
            }
        })
        .ok_or("can not get root span")?;

    Ok(spans
        .iter()
        .map(|span| {
            let (parent_span_id, references) = match span.link {
                minitrace::Link::Root { .. } => (0, vec![]),
                minitrace::Link::Parent { id } => (
                    id as i64,
                    vec![crate::thrift::jaeger::SpanRef {
                        kind: crate::thrift::jaeger::SpanRefKind::ChildOf,
                        trace_id_low,
                        trace_id_high,
                        span_id: id as i64,
                    }],
                ),
            };

            crate::thrift::jaeger::Span {
                trace_id_low,
                trace_id_high,
                span_id: span.id as i64,
                parent_span_id,
                operation_name: tag_display(span.tag),
                references,
                flags: 1,
                start_time: ((start_time_ms + span.elapsed_start as u64) * 1000) as i64,
                duration: (dbg!(span.elapsed_end - span.elapsed_start) * 1000) as i64,
                tags: vec![],
                logs: vec![],
            }
        })
        .collect())
}
