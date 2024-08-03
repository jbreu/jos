#include <inttypes.h> 

uint64_t strlen( const char* str ) {
    int len = 0;

    while (str[len] != '\0') {
        len++;
    }

    return len;
}

// TODO: Reduce code duplication

void write(uint64_t filedescriptor, const char* payload, uint64_t len) {
    register uint64_t r8 asm("r8") = filedescriptor;
    register uintptr_t r9 asm("r9") = (uintptr_t) payload;
    register uint64_t r10 asm("r10") = len;

    asm volatile (
        ".intel_syntax noprefix;"
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

uint64_t getpid() {
    uint64_t _pid = 0xdeadbeef;

    asm volatile (
        ".intel_syntax noprefix;"
        "push rdi;"
        "mov rdi, 2;"

        "push r11;"
        "push rcx;"
        
        "syscall;"
        
        "pop rcx;"
        "pop r11;"
        "pop rdi;"

        ".att_syntax;"
        : "=a" (_pid)
        
    );

    return _pid;
}

void draw_pixel(uint32_t x, uint32_t y, uint8_t color) {
    register uint64_t r8 asm("r8") = x;
    register uint64_t r9 asm("r9") = y;
    register uint64_t r10 asm("r10") = color;

    asm volatile (
        ".intel_syntax noprefix;"
        "push rdi;"
        "mov rdi, 3;"
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

uint64_t malloc(uint64_t size) {
    uint64_t address = 0;
    register uint64_t r8 asm("r8") = size;

    asm volatile (
        ".intel_syntax noprefix;"
        "push rdi;"
        "mov rdi, 4;"
        "push r11;"
        "push rcx;"
        "syscall;"
        "pop rcx;"
        "pop r11;"
        "pop rdi;"
        ".att_syntax;"
        : "=a" (address)
        : "r" (r8)
        : "rdi", "r11", "rcx"
    );

    return address;
}

uint64_t free(uint64_t address) {
    // TODO: does nothing for now
}

void fopen() {
    asm volatile (
        ".intel_syntax noprefix;"
        "push rdi;"
        "mov rdi, 5;"
        "push r11;"
        "push rcx;"
        "syscall;"
        "pop rcx;"
        "pop r11;"
        "pop rdi;"
        ".att_syntax;"
        :
        :
        : "rdi", "r11", "rcx"
    );
}

void fclose() {
    // TODO: does nothing for now
}


void fwrite() {
    // TODO: does nothing for now
}

void fseek(uint64_t offset, uint64_t origin) {
    register uint64_t r8 asm("r8") = offset;
    register uint64_t r9 asm("r9") = origin;

    asm volatile (
        ".intel_syntax noprefix;"
        "push rdi;"
        "mov rdi, 7;"
        "push r11;"
        "push rcx;"
        "syscall;"
        "pop rcx;"
        "pop r11;"
        "pop rdi;"
        ".att_syntax;"
        :
        : "r" (r8), "r" (r9)
        : "rdi", "r11", "rcx"
    );
}


uint64_t feof() {
    uint64_t eof = 0;

    asm volatile (
        ".intel_syntax noprefix;"
        "push rdi;"
        "mov rdi, 9;"
        "push r11;"
        "push rcx;"
        "syscall;"
        "pop rcx;"
        "pop r11;"
        "pop rdi;"
        ".att_syntax;"
        : "=a" (eof)
        :
        : "rdi", "r11", "rcx"
    );

    return eof;
}

uint64_t ftell() {
    uint64_t position = 0;

    asm volatile (
        ".intel_syntax noprefix;"
        "push rdi;"
        "mov rdi, 8;"
        "push r11;"
        "push rcx;"
        "syscall;"
        "pop rcx;"
        "pop r11;"
        "pop rdi;"
        ".att_syntax;"
        : "=a" (position)
        :
        : "rdi", "r11", "rcx"
    );

    return position;
}

void fread(uint8_t* ptr, uint64_t size, uint64_t nmemb) {
    register uintptr_t r8 asm("r8") = (uintptr_t) ptr;
    register uint64_t r9 asm("r9") = size * nmemb;

    asm volatile (
        ".intel_syntax noprefix;"
        "push rdi;"
        "mov rdi, 6;"
        "push r11;"
        "push rcx;"
        "syscall;"
        "pop rcx;"
        "pop r11;"
        "pop rdi;"
        ".att_syntax;"
        :
        : "r" (r8), "r" (r9)
        : "rdi", "r11", "rcx"
    );
}
