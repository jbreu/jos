#ifndef __LIBC_H__
#define __LIBC_H__

#include <inttypes.h> 
#include <stdbool.h>
#include "PureDOOM.h"

#define DOOM_IMPLEMENTATION 

#define DO_SYSCALL(syscall_num, output, r8_val, r9_val, r10_val) \
    asm volatile ( \
        ".intel_syntax noprefix;" \
        "push rdi;" \
        "mov rdi, %[num];" \
        "mov r8, %[r8v];" \
        "mov r9, %[r9v];" \
        "mov r10, %[r10v];" \
        "push r11;" \
        "push rcx;" \
        "syscall;" \
        "pop rcx;" \
        "pop r11;" \
        "pop rdi;" \
        ".att_syntax;" \
        : "=a" (output) \
        : [num] "r" ((uint64_t)syscall_num), \
          [r8v] "r" ((uint64_t)r8_val), \
          [r9v] "r" ((uint64_t)r9_val), \
          [r10v] "r" ((uint64_t)r10_val) \
        : "rdi", "r8", "r9", "r10", "r11", "rcx" \
    )


// String and memory functions
uint64_t strlen( const char* str );
bool strcmp(const char* a, const char* b);

// File I/O functions
void* fopen(const char* filename, const char* options);
void fclose(void* handle);
int fwrite(void* handle, const void* foo, int bar);
int fread(void* handle, void* ptr, int size);
int fseek(void* handle, int offset, doom_seek_t origin);
int ftell(void* handle);
int feof(void* handle);

// System calls and utilities
uint64_t getpid();
void draw_pixel(uint32_t x, uint32_t y, uint8_t color);
void* malloc(int size);
void free(void* address);
void write(uint64_t filedescriptor, const char* payload, uint64_t len);
uint64_t draw_framebuffer(const uint8_t* framebuffer);
uint64_t switch_vga_mode(bool vga_on);
bool get_keystate(int key);
void get_time(int* sec, int* usec);

#endif // __LIBC_H__
