void write(int filedescriptor, const char* payload, int len) {
    register int r8 asm("r8") = filedescriptor;
    register int r9 asm("r9") = (int) payload;
    register int r10 asm("r10") = len;

    asm volatile (
        ".intel_syntax noprefix;"
        "mov %1, %0;"
        "push rdi;"
        "mov rdi, 1;"
        "push r11;"
        "push rcx;"
        "syscall;"
        "pop rcx;"
        "pop r11;"
        "pop rdi;"
        ".att_syntax;"            
        :: "r" (r8), "r" (r9), "r" (r10)
        );
}
/*
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
    let mut address: u64 = 0;

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
    let mut eof: u64 = 0;

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
    let mut position: u64 = 0;

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
}*/