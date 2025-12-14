#include "stddef.h"
#include "wctype.h"

#ifndef __WCHAR_H__
#define __WCHAR_H__

typedef struct {
  int __count;
  union {
    __WINT_TYPE__ __wch;
    char __wchb[4];
  } __value; /* Value so far.  */
} mbstate_t;

size_t mbrlen(const char *s, size_t n2, mbstate_t *ps);
size_t mbrtowc(wchar_t *pwc, const char *s, size_t n, mbstate_t *ps);
wchar_t *wcschr(const wchar_t *wcs, wchar_t wc);
int iswspace(wchar_t wc);
int iswblank(wint_t wc);
size_t mbsrtowcs(wchar_t *dest, const char **src, size_t len, mbstate_t *ps);

#endif /* wchar.h  */