
#ifndef SETJMP_H
#define SETJMP_H

#include "ctype.h"
#include "stdint.h"

typedef long int jmp_buf[8];

#define setjmp(env) __builtin_setjmp(env)
#define longjmp(env, val) __builtin_longjmp(env, val)

#endif // SETJMP_H