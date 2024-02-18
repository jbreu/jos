use core::arch::asm;

extern "C" {
    fn trigger_syscall();
}

//pid_t getppid(void);
pub fn getpid() -> u64 {
    let mut pid = core::u64::MAX;

    unsafe {
        asm!("
            push rdx
            push rcx

            mov rdx, {0:r}

            call {1}

            mov {2:r}, rdx

            pop rcx
            pop rdx
        ",
            in(reg) 2,
            sym trigger_syscall,
            out(reg) pid
        );
    }

    pid
}

pub fn write(filedescriptor: i64, payload: &[u8]) {
    unsafe {
        asm!("
            push rdx
            push rbx
            push r8
            push r9
            push rcx

            mov rdx, {0:r}
            mov rbx, {1:r}
            mov r8, {2:r}
            mov r9, {3:r}

            call {4}

            pop rcx
            pop r9
            pop r8
            pop rbx
            pop rdx
        ",
            in(reg) 1,
            in(reg) filedescriptor,
            in(reg) payload.as_ptr(),
            in(reg) payload.len(),
            sym trigger_syscall
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
