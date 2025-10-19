use elf::file;

use crate::ERROR;
use crate::kprintln;
use crate::{USERLAND, time};
use crate::{keyboard, vga};
use core::arch::asm;

#[unsafe(no_mangle)]
pub extern "C" fn system_call() -> u64 {
    let mut syscall_nr: i64;
    let mut arg0: u64;
    let mut arg1: u64;
    let mut arg2: u64;
    let mut _arg3: u64;
    let mut _arg4: u64;
    let mut _arg5: u64;

    unsafe {
        asm!("",
            out("rdi") syscall_nr,
            out("r8") arg0,
            out("r9") arg1,
            out("r10") arg2,
            out("r11") _arg3,
            out("r12") _arg4,
            out("r13") _arg5,
        );
    }

    match syscall_nr {
        1 => return syscall_write(arg0, arg1, arg2),
        2 => return syscall_getpid(),
        3 => return syscall_plot_pixel(arg0 as u32, arg1 as u32, arg2 as u32),
        4 => return syscall_malloc(arg0 as usize),
        5 => return syscall_fopen(arg0 as *const u64, arg1 as *mut u32),
        6 => return syscall_fread(arg0, arg1, arg2 as usize),
        7 => return syscall_fseek(arg0, arg1 as usize, arg2 as usize),
        8 => return syscall_ftell(arg0),
        9 => return syscall_feof(arg0),
        10 => return syscall_plot_framebuffer(arg0),
        11 => return syscall_switch_vga_mode(arg0),
        12 => return syscall_get_keystate(arg0 as usize),
        13 => return syscall_get_time(arg0 as *mut u32, arg1 as *mut u32),
        14 => return syscall_stat(arg0 as *const u64, arg1 as *mut u64),
        15 => return syscall_chdir(arg0 as *const u64),
        16 => return syscall_getcwd(arg0 as *mut u64, arg1),
        17 => return syscall_getppid(),
        _ => {
            ERROR!("Undefined system call triggered: {}", syscall_nr);
            return 0xdeadbeef;
        }
    }
}

fn syscall_feof(_handle: u64) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    todo!();
}

fn syscall_ftell(_handle: u64) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    todo!();
}

fn syscall_fseek(handle: u64, offset: usize, origin: usize) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    return USERLAND
        .lock()
        .get_current_process()
        .fseek(handle, offset, origin as u32);
}

fn syscall_fread(handle: u64, ptr: u64, num_bytes: usize) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    USERLAND
        .lock()
        .get_current_process()
        .fread(handle, ptr as *mut u8, num_bytes)
}

fn syscall_fopen(filename: *const u64, mode: *mut u32) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());

    if filename.is_null() {
        return 0;
    }

    match unsafe { core::str::from_utf8(core::slice::from_raw_parts(filename as *const u8, 256)) } {
        Ok(path_str) => match path_str.split('\0').next() {
            Some(path_str) => match unsafe {
                core::str::from_utf8(core::slice::from_raw_parts(mode as *const u8, 8))
            } {
                Ok(mode_str) => USERLAND
                    .lock()
                    .get_current_process()
                    .fopen(path_str, mode_str),
                Err(_) => 0,
            },
            None => return 0,
        },
        Err(_) => 0,
    }
}

fn syscall_malloc(size: usize) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    return USERLAND.lock().process_malloc(size);
}

fn syscall_plot_pixel(x: u32, y: u32, color: u32) -> u64 {
    //let _event = core::hint::black_box(crate::instrument!()); // too much noise
    vga::vga_plot_pixel(x, y, color as u8);
    vga::vga_flip();
    return 0;
}

fn syscall_getpid() -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    USERLAND.lock().get_current_process_id() as u64
}

fn syscall_write(filedescriptor: u64, payload: u64, len: u64) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());

    if len == 0 {
        return 0;
    }

    unsafe {
        match core::str::from_utf8(core::slice::from_raw_parts(
            payload as *const u8,
            len as usize,
        )) {
            Ok(msg) => match filedescriptor {
                // stdout
                1 => {
                    kprintln!("{}", msg)
                }
                _ => ERROR!("Undefined filedescriptor!"),
            },
            Err(_) => ERROR!("\nCouldnt reconstruct string!\n"),
        }
    }
    return 0;
}

fn syscall_plot_framebuffer(framebuffer: u64) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    vga::vga_plot_framebuffer(framebuffer as *const u8);
    vga::vga_flip();
    return 0;
}

fn syscall_switch_vga_mode(vga_on: u64) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    if vga_on != 0 {
        vga::vga_enter();
        vga::vga_clear_screen();
    } else {
        vga::vga_exit();
    }
    return 0;
}

fn syscall_get_keystate(key: usize) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    let keystate;
    unsafe {
        keystate = keyboard::KEYSTATES[key];
        keyboard::KEYSTATES[key] = false;
    }
    return keystate as u64;
}

fn syscall_get_time(sec: *mut u32, usec: *mut u32) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    unsafe {
        let (s, us) = time::get_time();
        core::ptr::write_unaligned(sec, s);
        core::ptr::write_unaligned(usec, us);
    }
    return 1;
}

fn syscall_stat(_pathname: *const u64, _statbuf: *mut u64) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    todo!();
}

fn syscall_chdir(pathname: *const u64) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    // get string from pathname pointer
    match unsafe { core::str::from_utf8(core::slice::from_raw_parts(pathname as *const u8, 256)) } {
        Ok(pathname) => {
            return USERLAND
                .lock()
                .get_current_process()
                .set_working_directory(pathname);
        }
        Err(_) => return u64::MAX,
    }
}

fn syscall_getcwd(buf: *mut u64, size: u64) -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    let cwd = USERLAND
        .lock()
        .get_current_process()
        .get_working_directory();
    // copy cwd to buf
    let cwd_bytes = cwd.as_bytes();
    let cwd_len = cwd_bytes.len();
    let buf_slice = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, size as usize) };
    for i in 0..core::cmp::min(cwd_len, size as usize) {
        buf_slice[i] = cwd_bytes[i];
    }
    return cwd_len as u64;
}

fn syscall_getppid() -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    USERLAND.lock().get_current_process_parent_id() as u64
}
