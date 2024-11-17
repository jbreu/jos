#ifndef __STDIO_H__
#define __STDIO_H__

#include "ctype.h"

typedef struct {
    int fd;                 // File descriptor for system calls
    unsigned char *buffer;  // Pointer to buffer
    size_t bufsize;         // Buffer size
    size_t pos;             // Current position in buffer
    int flags;              // Flags to indicate mode, errors, EOF, etc.
} FILE;

#define EOF (-1)
#define NULL ((void *)0)

extern FILE *stdout;
extern FILE *stderr;

int fprintf(FILE *stream, const char *format, ...);
#define fputs(str, stream) fprintf(stream, "%s", str)

int sprintf ( char * str, const char * format, ... );

int fputc(int c, FILE *stream);
#define putc(c, stream) fputc(c, stream)

char * strrchr (char * str, int character );
int getopt(int argc, char * const argv[], const char *optstring);
void* fopen(const char* filename, const char* options);
void fclose(void* handle);
char *strerror(int errnum);
int rename(const char* old_filename, const char* new_filename);

int	isalnum(int);
int	isalpha(int);
int	iscntrl(int);
int	isdigit(int);
int	isgraph(int);
int	islower(int);
int	isprint(int);
int	ispunct(int);
int	isspace(int);
int	isupper(int);
int	isxdigit(int);
int	tolower(int);
int	toupper(int);
int	isblank(int);

#endif