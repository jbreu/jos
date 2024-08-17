#ifndef __LIBC_H__
#define __LIBC_H__

#include <inttypes.h> 
#include <stdbool.h>
#include "PureDOOM.h"

#define DOOM_IMPLEMENTATION 

uint64_t strlen( const char* str );
uint64_t getpid();
void draw_pixel(uint32_t x, uint32_t y, uint8_t color);
void * malloc(int size);
void free(void * address);
void * fopen(const char* filename, const char* options);
void fclose(void* handle);
int fwrite(void* handle, const void * foo, int bar);
int fseek(void* handle, int offset, doom_seek_t origin);
int feof(void* handle);
int ftell(void* handle);
int fread(void* handle, void* ptr, int size);
uint64_t draw_framebuffer(const uint8_t* framebuffer);
uint64_t switch_vga_mode(bool vga_on);

void write(uint64_t filedescriptor, const char* payload, uint64_t len);

#endif //__LIBC_H__