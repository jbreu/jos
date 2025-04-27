#include "stddef.h"
#include "time.h"
#include "stdint.h"
#include "chartypes.h"
#include "fnmatch.h"
#include "sys/wait.h"
#include "sys/resource.h"

void exit( int exit_code );
void* malloc(long unsigned  size);
void* realloc (void* ptr, size_t size);
void free(void* address);

void* bsearch (const void* key, const void* base, size_t num, size_t size, int (*compar)(const void*,const void*));

void abort (void);

typedef long long int jmp_buf[8]; // Example, depends on the system
void longjmp( jmp_buf env, int status );
int setjmp(jmp_buf env);

int atoi (const char * str);

void *alloca(size_t size);

void qsort (void* base, size_t num, size_t size, int (*compar)(const void*,const void*));