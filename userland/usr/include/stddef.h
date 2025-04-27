#ifndef __STDDEF_H__
#define __STDDEF_H__

typedef long unsigned int size_t;

typedef unsigned long int ssize_t;

typedef unsigned long int ptrdiff_t;

#define offsetof(type, member) ((size_t) &(((type *)0)->member))

#endif