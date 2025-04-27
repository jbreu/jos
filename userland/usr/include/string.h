#ifndef __STRING_H__
#define __STRING_H__

#include "ctype.h"
#include "stdbool.h"
#include "chartypes.h"
#include "stdint.h"
#include "stddef.h"

// String and memory functions
uint64_t strlen( const char* str );
bool strcmp(const char* a, const char* b);
char * strcpy(char * destination, const char * source);
char * strncpy ( char * destination, const char * source, size_t num );
char *stpcpy(char *dest, const char *src);
char * strcat(char * destination, const char * source);
char * strchr ( const char * str, int character );
intmax_t strtoimax( const char * nptr, char ** endptr, int base );
char *strchrnul(const char *s, int c);
size_t strcspn ( const char * str1, const char * str2 );
char *strerror(int errnum);
char * strtok ( char * str, const char * delimiters );
char * strpbrk ( const char * str1, const char * str2 );
char *stpncpy(char *dest, const char *src, size_t n);
char* strstr( const char* str, const char* substr );
char * strdup( const char *str1 );
size_t strspn ( const char * str1, const char * str2 );
double strtod( char const * str, char ** endptr );


int strcasecmp(const char *s1, const char *s2);
int strncasecmp(const char *s1, const char *s2, size_t n);

char *strsignal(int sig);

void* memcpy( void* dest, const void* src, size_t count );
void *mempcpy(void *dest, const void *src, size_t n);
int memcmp ( const void * ptr1, const void * ptr2, size_t num );
void * memmove ( void * destination, const void * source, size_t num );
void * memset ( void * ptr, int value, size_t num );

#endif