#include <inttypes.h> 

uint64_t strlen( const char* str );
uint64_t getpid();
void draw_pixel(uint32_t x, uint32_t y, uint8_t color);
uint64_t malloc(uint64_t size);
uint64_t free(uint64_t address);
uint64_t fopen(const char* filename);
void fclose(void* handle);
void fwrite(void* handle);
uint64_t fseek(void* handle, uint64_t offset, uint64_t origin);
uint64_t feof(void* handle);
uint64_t ftell(void* handle);
uint64_t fread(void* handle, void* ptr, uint64_t size);

void write(int filedescriptor, const char* payload, int len);
