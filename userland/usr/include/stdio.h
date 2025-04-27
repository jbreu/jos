#ifndef __STDIO_H__
#define __STDIO_H__

#include "ctype.h"
#include "stddef.h"
#include "time.h"
#include "sgtty.h"
#include "stdint.h"
#include "chartypes.h"
#include "limits.h"

typedef __builtin_va_list va_list;
#define va_start(v,l)	__builtin_va_start(v,l)
#define va_arg(v,l)	__builtin_va_arg(v,l)
#define va_end(v)	__builtin_va_end(v)

typedef struct {
    int fd;                 // File descriptor for system calls
    unsigned char *buffer;  // Pointer to buffer
    size_t bufsize;         // Buffer size
    size_t pos;             // Current position in buffer
    int flags;              // Flags to indicate mode, errors, EOF, etc.
} FILE;

#define EOF (-1)
#define NULL ((void *)0)

extern FILE *stdin;
extern FILE *stdout;
extern FILE *stderr;

int printf(const char *format, ...);
int vfprintf( FILE* stream, const char* format, va_list vlist );
int vsnprintf (char * s, size_t n, const char * format, va_list arg );

int fprintf(FILE *stream, const char *format, ...);
#define fputs(str, stream) fprintf(stream, "%s", str)

int sprintf ( char * str, const char * format, ... );

int fputc(int c, FILE *stream);
#define putc(c, stream) fputc(c, stream)

char * strrchr (char * str, int character );
int getopt(int argc, char * const argv[], const char *optstring);
void* fopen(const char* filename, const char* options);
int fclose ( FILE * stream );
int fflush ( FILE * stream );
char *strerror(int errnum);
int rename(const char* old_filename, const char* new_filename);

void perror ( const char * str );
int ferror ( FILE * stream );

char *fgets(char *str, int n, FILE *stream);

long unsigned int fwrite(const void *, long unsigned int,  long unsigned int,  void *);

#define SEEK_CUR 1

#endif