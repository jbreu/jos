use crate::DEBUG;
use crate::get_ns_since_boot;
extern crate alloc;
use core::arch::x86_64::_rdtsc;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber, field::Visit};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Copy, Clone)]
struct TracePoint {
    id: u64,
    time: u64,
    cycles: u64,
    fn_id: u64,
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

// === BEGIN: Static mapping table for tracepoint ids ===
fn trace_id_to_name(id: u64) -> &'static str {
    match id {
        0 => "Unknown",
        1 => "Process::new",
        2 => "Process::initialize",
        3 => "Process::init_process_heap",
        4 => "Process::malloc",
        5 => "Process::launch",
        6 => "Process::activate",
        7 => "Process::passivate",
        8 => "Process::activatable",
        10 => "vga_write_regs",
        11 => "vga_backup_palette_256",
        12 => "vga_restore_palette_256",
        13 => "vga_backup_text_mode_palette",
        14 => "vga_restore_text_mode_palette",
        15 => "vga_backup_vidmem",
        16 => "vga_restore_vidmem",
        17 => "vga_flip",
        18 => "vga_enter",
        19 => "vga_exit",
        20 => "vga_clear_screen",
        21 => "vga_plot_pixel",
        22 => "vga_plot_framebuffer",
        23 => "vga_write_font",
        24 => "vga_vmemwr",
        25 => "vga_set_plane",
        26 => "vga_set_custom_palette",
        30 => "compare_str_to_memory",
        40 => "Userland::new",
        41 => "Userland::process_malloc",
        42 => "Userland::switch_to_userland",
        50 => "set_initial_time",
        51 => "update_clock",
        60 => "allocate_page_frame",
        61 => "allocate_page_frame_for_given_physical_address",
        62 => "map_page_in_page_tables",
        70 => "get_key_for_scancode",
        80 => "set_idt_gate",
        81 => "init_idt",
        90 => "init_gdt",
        91 => "init_tss",
        100 => "FileHandle::new",
        101 => "FileHandle::read",
        102 => "FileHandle::fseek",
        103 => "Ext2FileSystem::new",
        104 => "Ext2FileSystem::read_block",
        105 => "Ext2FileSystem::read_inode",
        106 => "Ext2FileSystem::debug_print_superblock",
        107 => "Ext2FileSystem::read_inode_by_path",
        108 => "Ext2FileSystem::read_file",
        109 => "init_filesystem",
        200 => "syscall_feof",
        201 => "syscall_ftell",
        202 => "syscall_fseek",
        203 => "syscall_fread",
        204 => "syscall_fopen",
        205 => "syscall_malloc",
        206 => "syscall_plot_pixel",
        207 => "syscall_getpid",
        208 => "syscall_write",
        209 => "syscall_plot_framebuffer",
        210 => "syscall_switch_vga_mode",
        211 => "syscall_get_keystate",
        212 => "syscall_get_time",
        213 => "syscall_stat",
        214 => "syscall_chdir",
        215 => "syscall_getcwd",
        // New process.rs functions
        216 => "Process::print_page_table_tree",
        217 => "Process::get_physical_address_for_virtual_address",
        218 => "Process::get_c3_page_map_l4_base_address",
        219 => "Process::get_stack_top_address",
        220 => "Process::get_entry_ip",
        221 => "Process::load_elf_from_bin",
        222 => "Process::set_working_directory",
        223 => "Process::get_working_directory",
        224 => "Process::fopen",
        // New userland.rs functions
        230 => "Userland::switch_process",
        231 => "Userland::get_current_process_id",
        232 => "Userland::get_current_process",
        233 => "schedule",
        _ => {
            DEBUG!("Unknown trace ID: {}", id);
            "unknown"
        }
    }
}
// === END: Static mapping table ===

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
                    trace_id_to_name(trace_point.fn_id)
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
                trace_id_to_name(trace_point.fn_id)
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
    fn_id: 0,
    trace_type: TracePointType::Unknown,
}; TRACE_POINT_MAX];

static TRACE_POINT_COUNT: AtomicUsize = AtomicUsize::new(0);

struct FieldVisitor {
    fn_id: u64,
}

impl FieldVisitor {
    fn new() -> Self {
        Self { fn_id: 0 }
    }
}

impl Visit for FieldVisitor {
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == "fid" {
            self.fn_id = value;
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if field.name() == "fid" {
            self.fn_id = value as u64;
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, _: &dyn core::fmt::Debug) {
        if field.name() == "fid" {
            // Handle cases where fn_id might be passed as a debug value
            //let debug_str = core::fmt::format!("{:?}", value);
            //if let Ok(parsed) = debug_str.parse::<u64>() {
            //    self.fn_id = parsed;
            // }
        }
    }
}

pub struct SerialSubscriber;

impl Subscriber for SerialSubscriber {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &Attributes<'_>) -> Id {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        unsafe {
            let idx = TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed) % TRACE_POINT_MAX;

            let mut visitor = FieldVisitor::new();
            span.record(&mut visitor);
            let fn_id = visitor.fn_id;

            TRACE_POINTS[idx] = TracePoint {
                id: id,
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                fn_id: fn_id,
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
                fn_id: 0,
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
                fn_id: 0,
                trace_type: TracePointType::Follows,
            };
        }
    }

    fn event(&self, _: &Event<'_>) {
        unsafe {
            let idx = TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed) % TRACE_POINT_MAX;
            TRACE_POINTS[idx] = TracePoint {
                id: 0,
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                fn_id: 0,
                trace_type: TracePointType::Event,
            };
        }
    }

    fn enter(&self, _: &Id) {
        // already checked in `new_span`
    }

    fn exit(&self, span: &Id) {
        unsafe {
            let idx = TRACE_POINT_COUNT.fetch_add(1, Ordering::Relaxed) % TRACE_POINT_MAX;
            TRACE_POINTS[idx] = TracePoint {
                id: span.into_u64(),
                time: get_ns_since_boot!(),
                cycles: _rdtsc(),
                fn_id: 0,
                trace_type: TracePointType::Exit,
            };
        }
    }
}
