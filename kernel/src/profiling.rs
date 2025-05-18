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
    name: [char; 32],
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
        let count = TRACE_POINT_COUNT.load(Ordering::Relaxed);
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
    name: ['\0'; 32],
    trace_type: TracePointType::Unknown,
}; TRACE_POINT_MAX];

static mut TRACE_POINT_COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct SerialSubscriber;

impl Subscriber for SerialSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &Attributes<'_>) -> Id {
        let id: u64 = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        unsafe {
            TRACE_POINTS[TRACE_POINT_COUNT.load(Ordering::Relaxed)] = TracePoint {
                id,
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                name: {
                    let mut chars = ['\0'; 32];
                    let bytes = span.metadata().name().as_bytes();
                    for (i, &b) in bytes.iter().take(32).enumerate() {
                        chars[i] = b as char;
                    }
                    chars
                },
                trace_type: TracePointType::Enter,
            };
            TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed);
            if TRACE_POINT_COUNT.load(Ordering::Relaxed) >= TRACE_POINT_MAX {
                TRACE_POINT_COUNT.store(0, Ordering::Relaxed);
            }
            Id::from_u64(id)
        }
    }

    fn record(&self, span: &Id, values: &Record<'_>) {
        unsafe {
            TRACE_POINTS[TRACE_POINT_COUNT.load(Ordering::Relaxed)] = TracePoint {
                id: span.into_u64(),
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                name: ['\0'; 32],
                trace_type: TracePointType::Record,
            };
            TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed);
            if TRACE_POINT_COUNT.load(Ordering::Relaxed) >= TRACE_POINT_MAX {
                TRACE_POINT_COUNT.store(0, Ordering::Relaxed);
            }
        }
    }

    fn record_follows_from(&self, span: &Id, follows: &Id) {
        unsafe {
            TRACE_POINTS[TRACE_POINT_COUNT.load(Ordering::Relaxed)] = TracePoint {
                id: span.into_u64(),
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                name: ['\0'; 32],
                trace_type: TracePointType::Follows,
            };
            TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed);
            if TRACE_POINT_COUNT.load(Ordering::Relaxed) >= TRACE_POINT_MAX {
                TRACE_POINT_COUNT.store(0, Ordering::Relaxed);
            }
        }
    }

    fn event(&self, event: &Event<'_>) {
        unsafe {
            TRACE_POINTS[TRACE_POINT_COUNT.load(Ordering::Relaxed)] = TracePoint {
                id: 0,
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                name: {
                    let mut chars = ['\0'; 32];
                    let bytes = event.metadata().name().as_bytes();
                    for (i, &b) in bytes.iter().take(32).enumerate() {
                        chars[i] = b as char;
                    }
                    chars
                },
                trace_type: TracePointType::Event,
            };
            TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed);
            if TRACE_POINT_COUNT.load(Ordering::Relaxed) >= TRACE_POINT_MAX {
                TRACE_POINT_COUNT.store(0, Ordering::Relaxed);
            }
        }
    }

    fn enter(&self, span: &Id) {
        // Already covered by new_span
        //let time = get_ns_since_boot();
        //let cycles = unsafe { _rdtsc() };
        //DEBUG!("[{}ns/{}cyc] ENTER [{}]", time, cycles, span.into_u64());
    }

    fn exit(&self, span: &Id) {
        unsafe {
            TRACE_POINTS[TRACE_POINT_COUNT.load(Ordering::Relaxed)] = TracePoint {
                id: span.into_u64(),
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                name: ['\0'; 32],
                trace_type: TracePointType::Exit,
            };
            TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed);
            if TRACE_POINT_COUNT.load(Ordering::Relaxed) >= TRACE_POINT_MAX {
                TRACE_POINT_COUNT.store(0, Ordering::Relaxed);
            }
        }
    }
}
