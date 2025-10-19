use spin::Mutex;

use crate::USERLAND;
use crate::mem_config::{KERNEL_STACK_TOP_ADDRESS, USERSPACE_STACK_TOP_ADDRESS};
use crate::process::Process;

extern crate alloc;
use alloc::vec::Vec;

use core::arch::global_asm;
use core::fmt;

global_asm!(include_str!("switch_to_ring3.S"));

//#[derive(Default)]
pub struct Userland {
    processes: Vec<Process>,
    current_process: usize,
}

impl fmt::Debug for Userland {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Userland")
            .field("current_process", &self.current_process)
            .finish()
    }
}

impl Userland {
    pub fn new() -> Self {
        let _event = core::hint::black_box(crate::instrument!());

        Self {
            processes: Vec::new(),
            current_process: 0,
        }
    }

    pub fn process_malloc(&mut self, size: usize) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());

        return self.processes[self.current_process].malloc(size);
    }

    pub fn switch_to_userland(&mut self, mutex: &Mutex<Userland>) {
        let _event = core::hint::black_box(crate::instrument!());

        unsafe extern "C" {
            fn jump_usermode(process_base_address: u64, stack_top_address: u64, entry_address: u64);
        }

        unsafe {
            self.processes.push(Process::new());
            //self.processes.push(Process::new());
            //self.processes.push(Process::new());
            //self.processes.push(Process::new());

            for process in &mut self.processes {
                process.initialize();
            }

            self.current_process = 0;

            self.processes[0].launch();
            //self.processes[1].launch();
            //self.processes[2].launch();
            //self.processes[3].launch();

            let c3_page_map_l4_base_address = self.processes[0].get_c3_page_map_l4_base_address();

            self.processes[0].activate(true);

            // FIXME this feels very wrong!
            mutex.force_unlock();

            jump_usermode(
                c3_page_map_l4_base_address as u64,
                USERSPACE_STACK_TOP_ADDRESS as u64,
                self.processes[0].get_entry_ip() as u64,
            );
        }
    }

    pub fn switch_process(&mut self) {
        let _event = core::hint::black_box(crate::instrument!());

        // TODO for now scheduler is simply going round robin
        let last_process = self.current_process;

        loop {
            self.current_process += 1;
            if self.current_process == self.processes.len() {
                self.current_process = 0;
            }
            if self.processes[self.current_process].activatable() {
                break;
            }

            // not a single userspace process ready for execution
            if self.current_process == last_process {
                return;
            }
        }

        self.processes[last_process].passivate();
        self.processes[self.current_process].activate(false);
    }

    pub fn get_current_process_id(&self) -> usize {
        let _event = core::hint::black_box(crate::instrument!());

        self.current_process
    }

    pub fn get_current_process(&mut self) -> &mut Process {
        let _event = core::hint::black_box(crate::instrument!());

        &mut self.processes[self.current_process]
    }

    pub fn get_current_process_parent_id(&self) -> usize {
        let _event = core::hint::black_box(crate::instrument!());

        self.processes[self.current_process].get_parent_id()
    }
}

// very simple scheduler
pub fn schedule() {
    let _event = core::hint::black_box(crate::instrument!());

    USERLAND.lock().switch_process();
}

pub fn extend_stack() {
    let _event = core::hint::black_box(crate::instrument!());

    USERLAND.lock().get_current_process().extend_stack();
}
