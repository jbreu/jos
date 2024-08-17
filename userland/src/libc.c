#include "libc.h"
#include <inttypes.h> 
#include <stdbool.h>

uint64_t strlen( const char* str ) {
    int len = 0;

    while (str[len] != '\0') {
        len++;
    }

    return len;
}


doom_boolean strcmp(const char* a, const char* b) {

    int i =0;

    while (a[i] != '\0') {
        if (a[i]==b[i]) {
            i++;
            continue;
        } else {
            return false;
        }
    }

    return true;
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

void * malloc(int size) {
    // DBG write(1, "malloc: ", strlen("malloc: "));
    // DBG write(1, doom_itoa(size, 10), strlen(doom_itoa(size, 10)));

    uint64_t address = 0;
    register int r8 asm("r8") = size;

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

    return (void *) address;
}

void free(void * address) {
    // TODO: does nothing for now
}

void * fopen(const char* filename, const char* options) {

    // TODO Hack
    if (!strcmp(filename, "devdatadoom1.wad")) {
        return 0;
    }

    write(1, "fopen: ", strlen("fopen: "));
    write(1, filename, strlen(filename));

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

    return (void *) 1; // TODO return non-null; dont hardcode
}

void fclose(void* handle) {
    // TODO: does nothing for now
}


int fwrite(void* handle, const void * foo, int bar) {
    // TODO: does nothing for now
    write(1, "fwrite: ", strlen("fwrite: "));
}

int fseek(void* handle, int offset, doom_seek_t origin) {
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


int feof(void* handle) {
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

int ftell(void* handle) {
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


int fread(void* handle, void* ptr, int size) {
    register uintptr_t r8 asm("r8") = (uintptr_t) ptr;
    register uintptr_t r9 asm("r9") = size;
    int read_bytes = 0;

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
        : "=a" (read_bytes)
        : "r" (r8), "r" (r9)
        : "rdi", "r11", "rcx"
    );

    return read_bytes;
}

uint64_t draw_framebuffer(const uint8_t* framebuffer) {
    register uintptr_t r8 asm("r8") = (uintptr_t) framebuffer;

    asm volatile (
        ".intel_syntax noprefix;"
        "push rdi;"
        "mov rdi, 10;"
        "push r11;"
        "push rcx;"
        "syscall;"
        "pop rcx;"
        "pop r11;"
        "pop rdi;"
        ".att_syntax;"
        :
        : "r" (r8)
        : "rdi", "r11", "rcx"
    );

    return 0;
}

uint64_t switch_vga_mode(bool vga_on) {
    register uint64_t r8 asm("r8") = vga_on;

    asm volatile (
        ".intel_syntax noprefix;"
        "push rdi;"
        "mov rdi, 11;"
        "push r11;"
        "push rcx;"
        "syscall;"
        "pop rcx;"
        "pop r11;"
        "pop rdi;"
        ".att_syntax;"
        :
        : "r" (r8)
        : "rdi", "r11", "rcx"
    );

    return 0;
}

bool get_keystate(int key) {
    register uint64_t r8 asm("r8") = key;
    int state = 0;

    asm volatile (
        ".intel_syntax noprefix;"
        "push rdi;"
        "mov rdi, 12;"
        "push r11;"
        "push rcx;"
        "syscall;"
        "pop rcx;"
        "pop r11;"
        "pop rdi;"
        ".att_syntax;"
        : "=a" (state)
        : "r" (r8)
        : "rdi", "r11", "rcx"
    );

    return (bool) state;
}