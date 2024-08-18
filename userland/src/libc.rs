extern crate alloc;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;

pub struct ProcessAllocator {}

impl ProcessAllocator {
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {}
}

unsafe impl GlobalAlloc for ProcessAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc(layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        todo!();
    }
}

#[global_allocator]
static ALLOCATOR: ProcessAllocator = ProcessAllocator {};

pub fn getpid() -> u64 {
    let mut _pid = 0xdeadbeef;

    unsafe {
        asm!("
            push rdi
            mov rdi, 2

            push r11
            push rcx
        
            syscall
        
            pop rcx
            pop r11
            pop rdi
            ",
            out("rax") _pid,
            options(nostack)
        );
    }

    return _pid;
}

pub fn write(filedescriptor: i64, payload: *const u64, len: usize) {
    unsafe {
        asm!("
            push rdi
            mov rdi, 1

            push r11
            push rcx
        
            syscall
        
            pop rcx
            pop r11
            
            pop rdi
        ",
            in("r8") filedescriptor,
            in("r9") payload as u64,
            in("r10") len,
            options(nostack),
            clobber_abi("C")
        );
    }
}

pub fn draw_pixel(x: u32, y: u32, color: u8) {
    unsafe {
        asm!("
            push rdi
            mov rdi, 3

            push r11
            push rcx
        
            syscall
        
            pop rcx
            pop r11
            
            pop rdi
        ",
            in("r8") x,
            in("r9") y,
            in("r10") color as u64,
            options(nostack),
            clobber_abi("C")
        );
    }
}

pub fn malloc(size: usize) -> u64 {
    let mut address: u64;

    unsafe {
        asm!("
            push rdi
            mov rdi, 4

            push r11
            push rcx

            syscall

            pop rcx
            pop r11

            pop rdi
        ",
            in("r8") size,
            out("rax") address,
            options(nostack),
            clobber_abi("C")
        );
    }

    return address;
}

pub fn fopen() {
    unsafe {
        asm!(
            "
            push rdi
            mov rdi, 5

            push r11
            push rcx
        
            syscall
        
            pop rcx
            pop r11
            pop rdi
            ",
            options(nostack)
        );
    }
}

pub fn fclose() {
    // TODO does nothing for now
}

pub fn fwrite() {
    // TODO does nothing for now
}

pub fn fseek(offset: usize, origin: usize) {
    unsafe {
        asm!(
            "
            push rdi
            mov rdi, 7

            push r11
            push rcx
        
            syscall
        
            pop rcx
            pop r11
            pop rdi
            ",
            options(nostack),
            in("r8") offset,
            in("r9") origin,
        );
    }
}

pub fn feof() -> u64 {
    let mut eof: u64;

    unsafe {
        asm!(
            "
            push rdi
            mov rdi, 9

            push r11
            push rcx
        
            syscall
        
            pop rcx
            pop r11
            pop rdi
            ",
            options(nostack),
            out("rax") eof,
        );
    }

    return eof;
}

pub fn ftell() -> u64 {
    let mut position: u64;

    unsafe {
        asm!(
            "
            push rdi
            mov rdi, 8

            push r11
            push rcx
        
            syscall
        
            pop rcx
            pop r11
            pop rdi
            ",
            options(nostack),
            out("rax") position,
        );
    }

    return position;
}

//size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream);
pub fn fread(ptr: *mut u8, size: usize, nmemb: usize) {
    unsafe {
        asm!(
            "
            push rdi
            mov rdi, 6

            push r11
            push rcx
        
            syscall
        
            pop rcx
            pop r11
            pop rdi
            ",
            options(nostack),
            in("r8") ptr,
            in("r9") size*nmemb,
        );
    }
}

pub struct Printer {}

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write(1, s.as_bytes().as_ptr() as *const u64, s.len());
        Ok(())
    }
}

#[macro_export]
macro_rules! printf {
    () => {    };
    ($($arg:tt)*) => {{
        let mut printer = crate::libc::Printer {};
        core::fmt::write(&mut printer, core::format_args!($($arg)*)).unwrap();
    }};
}
