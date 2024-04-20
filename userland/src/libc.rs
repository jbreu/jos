use core::arch::asm;

//pid_t getppid(void);
pub fn getpid() -> u64 {
    let mut _pid = core::u64::MAX;

    unsafe {
        asm!("
            mov rdx, 2

            push r11
            push rcx
        
            syscall
        
            pop rcx
            pop r11
            ",
            out("r12") _pid,
        );
    }

    return _pid;
}

pub fn write(filedescriptor: i64, payload: &[u8]) {
    unsafe {
        asm!("
            mov rdx, 1

            push r11
            push rcx
        
            syscall
        
            pop rcx
            pop r11
        ",
            in("r14") filedescriptor,
            in("r12") payload.as_ptr(),
            in("r13") payload.len(),
            options(nostack,nomem)
        );
    }
}

pub struct Printer {}

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write(1, s.as_bytes());
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
