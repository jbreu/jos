use crate::USERLAND;
use crate::{kprintln, logging::log};
use core::arch::asm;

#[no_mangle]
pub extern "C" fn system_call() {
    let mut syscall_nr: i64;

    unsafe {
        asm!("
            mov {:r}, rdx
        ",
            out(reg) syscall_nr
        );
    }

    match syscall_nr {
        1 => syscall_write(),
        2 => syscall_getpid(),
        _ => {
            kprintln!("Undefined system call triggered");
        }
    }
}

fn syscall_getpid() {
    unsafe {
        asm!("
            nop
        ",
            in("r12") USERLAND.lock().get_current_process_id(),
        );
        //kprintln!("pid: {:x}\n", USERLAND.lock().get_current_process_id())
    }
}

fn syscall_write() {
    let mut filedescriptor: i64;
    let mut payload: i64;
    let mut len: i64;
    let bytes: &str;

    unsafe {
        // TODO this must be possible more elegantly
        asm!("nop",
            out("r14") filedescriptor,
            out("r12") payload,
            out("r13") len
        );

        bytes = core::str::from_utf8(core::slice::from_raw_parts(
            payload as *const u8,
            len as usize,
        ))
        .unwrap();
    }

    match filedescriptor {
        // stdout
        1 => {
            kprintln!("{}", bytes)
        }
        _ => log("Undefined filedescriptor!"),
    }
}
