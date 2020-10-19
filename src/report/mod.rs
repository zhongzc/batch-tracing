use crate::local::time_convert::cycle_to_realtime;
use crate::Span as RawSpan;
use rustracing_jaeger::thrift::agent::EmitBatchNotification;
use rustracing_jaeger::thrift::jaeger::{Batch, Process, Span, SpanRef, SpanRefKind};
use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use thrift_codec::message::Message;
use thrift_codec::CompactEncode;

pub struct Reporter {
    agent: SocketAddr,
    service_name: &'static str,
}

impl Reporter {
    pub fn new(agent: SocketAddr, service_name: &'static str) -> Self {
        Reporter {
            agent,
            service_name,
        }
    }

    pub fn report(&self, trace_id: u64, spans: Vec<RawSpan>) -> Result<(), Box<dyn Error>> {
        let local_addr: SocketAddr = if self.agent.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
        .parse()?;

        let udp = UdpSocket::bind(local_addr)?;

        let bn = EmitBatchNotification {
            batch: Batch {
                process: Process {
                    service_name: self.service_name.to_string(),
                    tags: vec![],
                },
                spans: spans
                    .into_iter()
                    .map(|s| {
                        let begin_cycles = cycle_to_realtime(s.begin_cycles);
                        let end_time = cycle_to_realtime(s.end_cycles);
                        Span {
                            trace_id_low: trace_id as i64,
                            trace_id_high: 0,
                            span_id: s.id.0 as i64,
                            parent_span_id: s.parent_id.0 as i64,
                            operation_name: s.event.to_string(),
                            references: vec![SpanRef {
                                kind: SpanRefKind::FollowsFrom,
                                trace_id_low: trace_id as i64,
                                trace_id_high: 0,
                                span_id: s.parent_id.0 as i64,
                            }],
                            flags: 1,
                            start_time: (begin_cycles.ns / 1_000) as i64,
                            duration: ((end_time.ns - begin_cycles.ns) / 1_000) as i64,
                            tags: vec![],
                            logs: vec![],
                        }
                    })
                    .collect(),
            },
        };

        let mut bytes = Vec::new();
        let msg = Message::from(bn);
        msg.compact_encode(&mut bytes)?;

        udp.send_to(&bytes, self.agent)?;

        Ok(())
    }
}
