use crate::get_ns_since_boot;
use crate::DEBUG;
extern crate alloc;
use alloc::vec::Vec;
use core::arch::x86_64::_rdtsc;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Copy, Clone)]
struct TracePoint {
    id: u64,
    time: u64,
    cycles: u64,
    name: [char; 16],
    trace_type: TracePointType,
}

#[derive(Copy, Clone)]
enum TracePointType {
    Unknown,
    Enter,
    Exit,
    Event,
    Record,
    Follows,
}

pub fn log_tracepoints() {
    unsafe {
        let count = TRACE_POINT_COUNT.load(Ordering::Relaxed) % TRACE_POINT_MAX;
        if TRACE_POINTS[count + 1].id != 0 {
            for i in (count + 1)..TRACE_POINT_MAX {
                let trace_point = &TRACE_POINTS[i];
                DEBUG!(
                    "[{}ns/{}cyc] {} [{}] {}",
                    trace_point.time,
                    trace_point.cycles,
                    match trace_point.trace_type {
                        TracePointType::Enter => "ENTER",
                        TracePointType::Exit => "EXIT",
                        TracePointType::Event => "EVENT",
                        TracePointType::Record => "RECORD",
                        TracePointType::Follows => "FOLLOWS",
                        _ => "UNKNOWN",
                    },
                    trace_point.id,
                    core::str::from_utf8(
                        &trace_point
                            .name
                            .iter()
                            .take_while(|&&c| c != '\0')
                            .map(|&c| c as u8)
                            .collect::<Vec<u8>>()
                    )
                    .unwrap_or(""),
                );
            }
        }
        for i in 0..=count {
            let trace_point = &TRACE_POINTS[i];
            DEBUG!(
                "[{}ns/{}cyc] {} [{}] {}",
                trace_point.time,
                trace_point.cycles,
                match trace_point.trace_type {
                    TracePointType::Enter => "ENTER",
                    TracePointType::Exit => "EXIT",
                    TracePointType::Event => "EVENT",
                    TracePointType::Record => "RECORD",
                    TracePointType::Follows => "FOLLOWS",
                    _ => "UNKNOWN",
                },
                trace_point.id,
                core::str::from_utf8(
                    &trace_point
                        .name
                        .iter()
                        .take_while(|&&c| c != '\0')
                        .map(|&c| c as u8)
                        .collect::<Vec<u8>>()
                )
                .unwrap_or(""),
            );
        }
    }
    DEBUG!("Tracepoints logged");
}

const TRACE_POINT_MAX: usize = 4096;

static mut TRACE_POINTS: [TracePoint; TRACE_POINT_MAX] = [TracePoint {
    id: 0,
    time: 0,
    cycles: 0,
    name: ['\0'; 16],
    trace_type: TracePointType::Unknown,
}; TRACE_POINT_MAX];

static TRACE_POINT_COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct SerialSubscriber;

impl Subscriber for SerialSubscriber {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &Attributes<'_>) -> Id {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        unsafe {
            let idx = TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed) % TRACE_POINT_MAX;
            let name_bytes = span.metadata().name().as_bytes();
            let mut name = ['\0'; 16];
            let len = if name_bytes.len() < 16 {
                name_bytes.len()
            } else {
                16
            };
            for i in 0..len {
                name[i] = name_bytes[i] as char;
            }
            TRACE_POINTS[idx] = TracePoint {
                id,
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                name,
                trace_type: TracePointType::Enter,
            };
        }
        Id::from_u64(id)
    }

    fn record(&self, span: &Id, _values: &Record<'_>) {
        unsafe {
            let idx = TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed) % TRACE_POINT_MAX;
            TRACE_POINTS[idx] = TracePoint {
                id: span.into_u64(),
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                name: ['\0'; 16],
                trace_type: TracePointType::Record,
            };
        }
    }

    fn record_follows_from(&self, span: &Id, _follows: &Id) {
        unsafe {
            let idx = TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed) % TRACE_POINT_MAX;
            TRACE_POINTS[idx] = TracePoint {
                id: span.into_u64(),
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                name: ['\0'; 16],
                trace_type: TracePointType::Follows,
            };
        }
    }

    fn event(&self, event: &Event<'_>) {
        unsafe {
            let idx = TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed) % TRACE_POINT_MAX;
            TRACE_POINTS[idx] = TracePoint {
                id: 0,
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                name: {
                    let mut chars = ['\0'; 16];
                    let bytes = event.metadata().name().as_bytes();
                    for (i, &b) in bytes.iter().take(16).enumerate() {
                        chars[i] = b as char;
                    }
                    chars
                },
                trace_type: TracePointType::Event,
            };
        }
    }

    fn enter(&self, _span: &Id) {
        // Already covered by new_span
        //let time = get_ns_since_boot();
        //let cycles = unsafe { _rdtsc() };
        //DEBUG!("[{}ns/{}cyc] ENTER [{}]", time, cycles, span.into_u64());
    }

    fn exit(&self, span: &Id) {
        unsafe {
            let idx = TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed) % TRACE_POINT_MAX;
            TRACE_POINTS[idx] = TracePoint {
                id: span.into_u64(),
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                name: ['\0'; 16],
                trace_type: TracePointType::Exit,
            };
        }
    }
}
