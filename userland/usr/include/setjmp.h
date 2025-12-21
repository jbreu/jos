
#ifndef SETJMP_H
#define SETJMP_H

#include "ctype.h"

typedef struct {
  uint64_t rsp;
  uint64_t rip;
  uint64_t rbx;
  uint64_t rbp;
  uint64_t r12;
  uint64_t r13;
  uint64_t r14;
  uint64_t r15;
} jmp_buf_struct;

typedef jmp_buf_struct jmp_buf[1];

void longjmp(jmp_buf env, int status);
int setjmp(jmp_buf env);

#endif // SETJMP_H