use crate::DEBUG;
use crate::get_ns_since_boot;
extern crate alloc;
use core::arch::x86_64::_rdtsc;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
const TRACE_POINT_MAX: usize = 16384; // 16k trace points
static TRACE_POINT_COUNT: AtomicUsize = AtomicUsize::new(0);

static mut TRACE_POINTS: [TracePoint; TRACE_POINT_MAX] = [TracePoint {
    call_id: 0,
    time: 0,
    cycles: 0,
    function_name: "Unknown",
    trace_type: TracePointType::Unknown,
}; TRACE_POINT_MAX];

#[derive(Copy, Clone)]
struct TracePoint {
    call_id: u64,
    time: u64,
    cycles: u64,
    function_name: &'static str,
    trace_type: TracePointType,
}

#[derive(Copy, Clone)]
enum TracePointType {
    Unknown,
    Enter,
    Exit,
    _Event,
    _Record,
    _Follows,
}

pub fn log_tracepoints() {
    unsafe {
        let count = TRACE_POINT_COUNT.load(Ordering::Relaxed) % TRACE_POINT_MAX;
        if TRACE_POINTS[count + 1].call_id != 0 {
            for i in (count + 1)..TRACE_POINT_MAX {
                let trace_point = &TRACE_POINTS[i];
                DEBUG!(
                    "[{}ns/{}cyc] {} [{}] {}",
                    trace_point.time,
                    trace_point.cycles,
                    match trace_point.trace_type {
                        TracePointType::Enter => "ENTER",
                        TracePointType::Exit => "EXIT",
                        TracePointType::_Event => "EVENT",
                        TracePointType::_Record => "RECORD",
                        TracePointType::_Follows => "FOLLOWS",
                        _ => "UNKNOWN",
                    },
                    trace_point.call_id,
                    trace_point.function_name
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
                    TracePointType::_Event => "EVENT",
                    TracePointType::_Record => "RECORD",
                    TracePointType::_Follows => "FOLLOWS",
                    _ => "UNKNOWN",
                },
                trace_point.call_id,
                trace_point.function_name
            );
        }
    }
    DEBUG!("Tracepoints logged");
}

// Approach adopted from https://github.com/Compaile/ctrack
pub struct EventHandler {
    function: &'static str,
    call_id: u64,
}

impl EventHandler {
    #[inline(always)]
    pub fn new(function: &'static str) -> Self {
        let call_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let start_clock = unsafe { _rdtsc() };
        let start_time = get_ns_since_boot!();

        unsafe {
            let idx = TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed) % TRACE_POINT_MAX;
            TRACE_POINTS[idx] = TracePoint {
                time: start_time,
                function_name: function,
                call_id,
                cycles: start_clock,
                trace_type: TracePointType::Enter,
            };
        }

        EventHandler { function, call_id }
    }
}

impl Drop for EventHandler {
    #[inline(always)]
    fn drop(&mut self) {
        let end_time = get_ns_since_boot!();
        let end_clock = unsafe { _rdtsc() };

        let trace_point = TracePoint {
            time: end_time,
            function_name: self.function,
            call_id: self.call_id,
            cycles: end_clock,
            trace_type: TracePointType::Exit,
        };

        unsafe {
            TRACE_POINTS[TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed) % TRACE_POINT_MAX] =
                trace_point;
        }
    }
}

#[macro_export]
macro_rules! instrument {
    () => {
        crate::profiling::EventHandler::new(crate::function_name!())
    };
}

#[macro_export]
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            core::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3] // trims "::f"
    }};
}
