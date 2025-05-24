use spin::Mutex;
use tracing::instrument;

use crate::process::Process;
use crate::USERLAND;

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
    #[instrument]
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            current_process: 0,
        }
    }

    #[instrument]

    pub fn process_malloc(&mut self, size: usize) -> u64 {
        return self.processes[self.current_process].malloc(size);
    }

    #[instrument]
    pub fn switch_to_userland(&mut self, mutex: &Mutex<Userland>) {
        extern "C" {
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
                c3_page_map_l4_base_address,
                self.processes[0].get_stack_top_address(),
                self.processes[0].get_entry_ip(),
            );
        }
    }

    #[instrument]
    pub fn switch_process(&mut self) {
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

    #[instrument]
    pub fn get_current_process_id(&self) -> usize {
        self.current_process
    }

    #[instrument]
    pub fn get_current_process(&mut self) -> &mut Process {
        &mut self.processes[self.current_process]
    }
}

// very simple scheduler
#[instrument]
pub fn schedule() {
    USERLAND.lock().switch_process();
}
