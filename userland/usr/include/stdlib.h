#include "chartypes.h"
#include "fnmatch.h"
#include "stddef.h"
#include "stdint.h"
#include "sys/resource.h"
#include "sys/wait.h"
#include "time.h"

void exit(int exit_code);
void *malloc(long unsigned size);
void *realloc(void *ptr, size_t size);
void free(void *address);

void *bsearch(const void *key, const void *base, size_t num, size_t size,
              int (*compar)(const void *, const void *));

void abort(void);

int atoi(const char *str);

void *alloca(size_t size);

void qsort(void *base, size_t num, size_t size,
           int (*compar)(const void *, const void *));