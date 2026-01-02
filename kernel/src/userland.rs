use spin::Mutex;

use crate::mem_config::USERSPACE_STACK_TOP_ADDRESS;
use crate::process::Process;
use crate::{ERROR, USERLAND};

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

        return self.get_current_process().malloc(size);
    }

    pub fn process_realloc(&mut self, ptr: u64, size: usize) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());

        return self.get_current_process().realloc(ptr, size);
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
                process.initialize("/dash");
            }

            self.current_process = self.processes[0].get_pid() as usize;

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

        // find vector index of current process by iterating through all processes
        let mut current_process_index = self
            .processes
            .iter()
            .position(|p| p.get_pid() == self.current_process as u64)
            .unwrap();

        let last_process = current_process_index;

        loop {
            current_process_index += 1;
            if current_process_index == self.processes.len() {
                current_process_index = 0;
            }
            if self.processes[current_process_index].activatable() {
                break;
            }

            // not a single userspace process ready for execution
            if current_process_index == last_process {
                return;
            }
        }

        self.processes[last_process].passivate();
        self.current_process = self.processes[current_process_index].get_pid() as usize;
        self.processes[self.current_process].activate(false);
    }

    pub fn get_current_process_id(&self) -> usize {
        let _event = core::hint::black_box(crate::instrument!());

        self.current_process
    }

    pub fn get_current_process(&mut self) -> &mut Process {
        let _event = core::hint::black_box(crate::instrument!());

        self.processes
            .iter_mut()
            .find(|p| p.get_pid() == self.current_process as u64)
            .unwrap()
    }

    pub fn get_current_process_parent_id(&mut self) -> usize {
        let _event = core::hint::black_box(crate::instrument!());

        self.get_current_process().get_parent_id() as usize
    }

    pub fn kill_process(&mut self, pid: u64, sig: u32) -> i64 {
        let _event = core::hint::black_box(crate::instrument!());

        if sig != 9 {
            ERROR!("Unsupported signal {}\n", sig);
            return -1;
        }

        if let Some(pos) = self.processes.iter().position(|x| x.get_pid() == pid) {
            self.processes.remove(pos);
            return 0;
        }

        return -1;
    }

    pub fn vfork_current_process(&mut self) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());

        let parent_process = self.get_current_process();
        parent_process.put_to_sleep();

        let mut child_process = Process::new();
        child_process.clone_from_parent(parent_process);

        let child_pid = child_process.get_pid();
        self.processes.push(child_process);

        self.current_process = child_pid as usize;

        // return 0 as we are executing in the child process
        return 0;
    }

    pub fn execve(&mut self, filename: &str) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());

        // cycle through all processes to find the current one
        let current_process = self.get_current_process();

        let filename_str = filename;
        current_process.initialize(filename_str);
        0
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
