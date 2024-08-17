#ifndef __LIBC_H__
#define __LIBC_H__

#include <inttypes.h> 
#include <stdbool.h>
#include "PureDOOM.h"

#define DOOM_IMPLEMENTATION 

uint64_t strlen( const char* str );
uint64_t getpid();
void draw_pixel(uint32_t x, uint32_t y, uint8_t color);
uint64_t malloc(uint64_t size);
uint64_t free(uint64_t address);
uint64_t fopen(const char* filename);
void fclose(void* handle);
void fwrite(void* handle);
void fseek(void* handle, uint64_t offset, uint64_t origin);
uint64_t feof(void* handle);
uint64_t ftell(void* handle);
uint64_t fread(void* handle, void* ptr, uint64_t size);
uint64_t draw_framebuffer(const uint8_t* framebuffer);
uint64_t switch_vga_mode(bool vga_on);

void write(uint64_t filedescriptor, const char* payload, uint64_t len);

#endif //__LIBC_H__