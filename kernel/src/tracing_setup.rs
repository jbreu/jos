use crate::time::get_ns_since_boot;
use crate::DEBUG;
use core::arch::x86_64::_rdtsc;
use core::sync::atomic::{AtomicU64, Ordering};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Metadata, Subscriber};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub struct SerialSubscriber;

impl Subscriber for SerialSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &Attributes<'_>) -> Id {
        let id: u64 = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let time = get_ns_since_boot();
        let cycles = unsafe { _rdtsc() };
        DEBUG!(
            "[{}ns/{}cyc] ENTER [{}] {}",
            time,
            cycles,
            id,
            span.metadata().name()
        );
        Id::from_u64(id)
    }

    fn record(&self, span: &Id, values: &Record<'_>) {
        let time = get_ns_since_boot();
        let cycles = unsafe { _rdtsc() };
        DEBUG!(
            "[{}ns/{}cyc] RECORD [{}] {:?}",
            time,
            cycles,
            span.into_u64(),
            values
        );
    }

    fn record_follows_from(&self, span: &Id, follows: &Id) {
        let time = get_ns_since_boot();
        let cycles = unsafe { _rdtsc() };
        DEBUG!(
            "[{}ns/{}cyc] FOLLOWS [{}] -> [{}]",
            time,
            cycles,
            span.into_u64(),
            follows.into_u64()
        );
    }

    fn event(&self, event: &Event<'_>) {
        let time = get_ns_since_boot();
        let cycles = unsafe { _rdtsc() };
        DEBUG!(
            "[{}ns/{}cyc] EVENT [{}] {}",
            time,
            cycles,
            event.metadata().level(),
            event.metadata().name()
        );
    }

    fn enter(&self, span: &Id) {
        let time = get_ns_since_boot();
        let cycles = unsafe { _rdtsc() };
        DEBUG!("[{}ns/{}cyc] ENTER [{}]", time, cycles, span.into_u64());
    }

    fn exit(&self, span: &Id) {
        let time = get_ns_since_boot();
        let cycles = unsafe { _rdtsc() };
        DEBUG!("[{}ns/{}cyc] EXIT [{}]", time, cycles, span.into_u64());
    }
}
