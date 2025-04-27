#ifndef CUSTOM_CHAR_MACROS_H
#define CUSTOM_CHAR_MACROS_H

#ifndef isalpha
#  define isalpha(c) (((c) >= 'a' && (c) <= 'z') || ((c) >= 'A' && (c) <= 'Z'))
#endif

#ifndef isdigit
#  define isdigit(c) ((c) >= '0' && (c) <= '9')
#endif

#ifndef isalnum
#  define isalnum(c) (isalpha(c) || isdigit(c))
#endif

#ifndef iscntrl
#  define iscntrl(c) (((c) >= 0 && (c) < 32) || ((c) == 127))
#endif

#ifndef isgraph
#  define isgraph(c) ((c) > 32 && (c) < 127)
#endif

#ifndef islower
#  define islower(c) ((c) >= 'a' && (c) <= 'z')
#endif

#ifndef isprint
#  define isprint(c) (isgraph(c) || (c) == ' ')
#endif

#ifndef ispunct
#  define ispunct(c) (isgraph(c) && !isalnum(c))
#endif

#ifndef isspace
#  define isspace(c) ((c) == ' ' || (c) == '\t' || (c) == '\n' || (c) == '\v' || (c) == '\f' || (c) == '\r')
#endif

#ifndef isupper
#  define isupper(c) ((c) >= 'A' && (c) <= 'Z')
#endif

#ifndef isxdigit
#  define isxdigit(c) (((c) >= '0' && (c) <= '9') || ((c) >= 'a' && (c) <= 'f') || ((c) >= 'A' && (c) <= 'F'))
#endif

#ifndef tolower
#  define tolower(c) (isupper(c) ? (c) + ('a' - 'A') : (c))
#endif

#ifndef toupper
#  define toupper(c) (islower(c) ? (c) + ('A' - 'a') : (c))
#endif

#ifndef isblank
#  define isblank(c) ((c) == ' ' || (c) == '\t')
#endif

#endif // CUSTOM_CHAR_MACROS_H
