use spin::Mutex;

use crate::kprintln;
use crate::process::Process;
use crate::USERLAND;
use core::arch::asm;
use core::arch::global_asm;

global_asm!(include_str!("switch_to_ring3.S"));

#[derive(Default)]
pub struct Userland {
    process0: Process,
    process1: Process,
    current_process: usize,
}

impl Userland {
    pub fn new() -> Self {
        Self {
            process0: Process::new(),
            process1: Process::new(),
            current_process: 0,
        }
    }

    pub fn switch_to_userland(&mut self, mutex: &Mutex<Userland>) {
        extern "C" {
            fn jump_usermode(process_base_address: u64, stack_top_address: u64, entry_address: u64);
        }

        unsafe {
            self.current_process = 0;

            self.process0.launch();
            self.process1.launch();
            self.process0.activate(true);

            // TODO Investigate: Disable interrupts, because otherwise during jump to usermode interrupt may appear, leading to weird behavior
            asm!("cli");

            // FIXME this feels very wrong!
            mutex.force_unlock();

            jump_usermode(
                self.process0.get_c3_page_map_l4_base_address(),
                self.process0.get_stack_top_address(),
                self.process0.get_entry_ip(),
            );
        }
    }

    pub fn switch_process(&mut self) {
        // TODO for now scheduler is simply going round robin
        let last_process = self.current_process;

        // TDOO ugly for only 2 processes
        match self.current_process {
            0 => {
                self.process0.passivate();
                self.process1.activate(false);
                self.current_process = 1;
                kprintln!("New process: 1");
                return;
            }
            1 => {
                self.process1.passivate();
                self.process0.activate(false);
                self.current_process = 0;
                kprintln!("New process: 0");
                return;
            }
            _ => {
                panic!("This should never happen!")
            }
        }

        // FIXME!!!
        /*loop {
            self.current_process += 1;
            if self.current_process == 2
            /*self.processes.len()*/
            {
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
        self.processes[self.current_process].activate();*/
    }

    pub fn get_current_process_id(&self) -> usize {
        self.current_process
    }
}

// very simple scheduler
pub fn schedule() {
    unsafe {
        USERLAND.force_unlock();
    }

    USERLAND.lock().switch_process();

    // TODO avoid!
    unsafe {
        USERLAND.force_unlock();
    }
}
