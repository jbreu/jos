#ifndef __STRING_H__
#define __STRING_H__

#include "ctype.h"
#include "stdbool.h"

// String and memory functions
uint64_t strlen( const char* str );
bool strcmp(const char* a, const char* b);
char * strcpy(char * destination, const char * source);
char * strcat(char * destination, const char * source);

#endif