#ifndef __LIBC_H__
#define __LIBC_H__

#include "stdio.h"
#include "ctype.h"
#include "errno.h"
#include "stdint.h"
#include "stdlib.h"
#include "string.h"
#include "stdbool.h"

typedef __builtin_va_list va_list;
#define va_start(v,l)	__builtin_va_start(v,l)
#define va_arg(v,l)	__builtin_va_arg(v,l)
#define va_end(v)	__builtin_va_end(v)

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




// File I/O functions
long unsigned int fwrite(const void *, long unsigned int,  long unsigned int,  void *);
int fread(void* handle, void* ptr, int size);
int fseek ( FILE * stream, long int offset, int origin );
int ftell(void* handle);
int feof(void* handle);

// System calls and utilities
void draw_pixel(uint32_t x, uint32_t y, uint8_t color);

int write(uint64_t filedescriptor, const char* payload, uint64_t len);
uint64_t draw_framebuffer(const uint8_t* framebuffer);
uint64_t switch_vga_mode(bool vga_on);
bool get_keystate(int key);
void get_time(int* sec, int* usec);

#endif // __LIBC_H__
