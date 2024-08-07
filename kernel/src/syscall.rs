use crate::file::{self, feof, fopen, fread, fseek, ftell};
use crate::vga::{vga_flip, vga_plot_pixel};
use crate::USERLAND;
use crate::{kprintln, logging::log};
use core::arch::asm;

#[no_mangle]
pub extern "C" fn system_call() -> u64 {
    let mut syscall_nr: i64;

    unsafe {
        asm!("
        ",
            out("rdi") syscall_nr
        );
    }

    match syscall_nr {
        1 => return syscall_write(),
        2 => return syscall_getpid(),
        3 => return syscall_plot_pixel(),
        4 => return syscall_malloc(),
        5 => return syscall_fopen(),
        6 => return syscall_fread(),
        7 => return syscall_fseek(),
        8 => return syscall_ftell(),
        9 => return syscall_feof(),
        _ => {
            kprintln!("Undefined system call triggered");
            return 0xdeadbeef;
        }
    }
}

fn syscall_feof() -> u64 {
    return feof();
}

fn syscall_ftell() -> u64 {
    return ftell() as u64;
}

fn syscall_fseek() -> u64 {
    let mut offset: usize;
    let mut origin: usize;

    unsafe {
        asm!("",
            out("r8") offset,
            out("r9") origin,
        );
    }

    fseek(offset, origin);

    return 0;
}

fn syscall_fread() -> u64 {
    let mut ptr: u64;
    let mut num_bytes: usize;

    unsafe {
        asm!("",
            out("r8") ptr,
            out("r9") num_bytes,
        );
    }

    fread(ptr as *mut u8, num_bytes);

    return 0;
}

fn syscall_fopen() -> u64 {
    fopen();

    return 0;
}

fn syscall_malloc() -> u64 {
    let mut size: usize;

    unsafe {
        asm!("",
            out("r8") size
        );
    }

    return USERLAND.lock().process_malloc(size);
}

fn syscall_plot_pixel() -> u64 {
    let mut x: u32;
    let mut y: u32;
    let mut color: u32;

    unsafe {
        // TODO this must be possible more elegantly
        asm!("",
            out("r8") x,
            out("r9") y,
            out("r10") color
        );
    }

    vga_plot_pixel(x, y, color as u8);
    vga_flip();

    return 0;
}

fn syscall_getpid() -> u64 {
    USERLAND.lock().get_current_process_id() as u64
}

fn syscall_write() -> u64 {
    let mut filedescriptor: i64;
    let mut payload: i64;
    let mut len: i64;

    unsafe {
        // TODO this must be possible more elegantly
        asm!("",
            out("r8") filedescriptor,
            out("r9") payload,
            out("r10") len
        );

        match core::str::from_utf8(core::slice::from_raw_parts(
            payload as *const u8,
            len as usize,
        )) {
            Ok(msg) => match filedescriptor {
                // stdout
                1 => {
                    kprintln!("{}", msg)
                }
                _ => log("Undefined filedescriptor!"),
            },
            Err(_) => kprintln!("\nCouldnt reconstruct string!\n"),
        }
    }

    return 0;
}
