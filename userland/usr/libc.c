#include "include/libc.h"
/* Provide a plain global _DYNAMIC symbol so linkers can resolve
  references when building static binaries. Some object files
  reference `_DYNAMIC' during configuration/build checks; defining
  it here as NULL avoids undefined reference errors. */
void *_DYNAMIC = 0;

__asm__(".global _start\n"
        ".extern main\n"
        "_start:\n"
        //        "    mov %rsp, %rdi\n" // argc in [rsp]
        //        "    mov (%rdi), %rdi\n"
        //        "    lea 8(%rsp), %rsi\n" // argv
        //        "    mov %rsi, %rdx\n"
        //        "1:\n"
        //        "    cmpq $0,(%rdx)\n"
        //        "    add $8,%rdx\n"
        //        "    jne 1b\n"
        //        "    add $8,%rdx\n" // envp
        "    call main\n"
        //        "    mov %rax,%rdi\n"
        //        "    mov $60,%rax\n" // exit
        //        "    syscall\n"
);

FILE _stdin = {
    .fd = 0,         // File descriptor 0 for stdin
    .buffer = 0,     // Some allocated buffer space
    .bufsize = 1024, // Buffer size, line-buffered for terminal
    .pos = 0,        // Current position in buffer
    .flags = 0,      // Set to read-only
};

FILE *stdin = &_stdin; // Point stdin to the _stdin instance

FILE _stdout = {
    .fd = 1,         // File descriptor 1 for stdout
    .buffer = 0,     // Some allocated buffer space
    .bufsize = 1024, // Buffer size, line-buffered for terminal
    .pos = 0,        // Current position in buffer
    .flags = 0,      // Set to write-only
};

FILE *stdout = &_stdout; // Point stdout to the _stdout instance

FILE _stderr = {
    .fd = 2,         // File descriptor 1 for stdout
    .buffer = 0,     // Some allocated buffer space
    .bufsize = 1024, // Buffer size, line-buffered for terminal
    .pos = 0,        // Current position in buffer
    .flags = 0,      // Set to write-only
};

FILE *stderr = &_stderr; // Point stdout to the _stdout instance

uint64_t strlen(const char *str) {
  int len = 0;

  while (str[len] != '\0') {
    len++;
  }

  return len;
}

bool strcmp(const char *a, const char *b) {
  int i = 0;

  while (a[i] != '\0') {
    if (a[i] == b[i]) {
      i++;
      continue;
    } else {
      return false;
    }
  }

  return true;
}

// Write function using syscall
ssize_t write(int filedescriptor, const void *payload, size_t len) {
  uint64_t result;
  DO_SYSCALL(1, result, filedescriptor, (uintptr_t)payload, len);
}

// Get process ID
pid_t getpid() {
  uint64_t pid;
  DO_SYSCALL(2, pid, 0, 0, 0); // No additional arguments needed
  return pid;
}

// Draw a pixel on screen
void draw_pixel(uint32_t x, uint32_t y, uint8_t color) {
  uint64_t result;
  DO_SYSCALL(3, result, x, y, color);
}

// Allocate memory
void *malloc(long unsigned int size) {
  uint64_t address;
  DO_SYSCALL(4, address, size, 0, 0); // Only size is passed
  return (void *)address;
}

// Free memory (currently does nothing) --> leaks memory
void free(void *address) {
  // TODO: Implement the free function
}

// Open a file
void *fopen(const char *filename, const char *options) {
  write(1, "fopen: ", strlen("fopen: "));
  write(1, filename, strlen(filename));

  uint64_t handle;
  DO_SYSCALL(5, handle, filename, options, 0);
  return (void *)handle;
}

// Close a file (currently does nothing)
int fclose(FILE *stream) {
  // TODO: Implement the fclose function
}

// Write to a file (currently does nothing)
long unsigned int fwrite(const void *, long unsigned int, long unsigned int,
                         void *) {
  write(1, "fwrite: ", strlen("fwrite: "));
  return 0; // TODO: Implement fwrite
}

// Seek within a file
int fseek(FILE *stream, long int offset, int origin) {
  uint64_t result;
  DO_SYSCALL(7, result, offset, origin, 0);
  return result;
}

// Check if end of file
int feof(void *handle) {
  uint64_t eof;
  DO_SYSCALL(9, eof, (uintptr_t)handle, 0, 0);
  return eof;
}

// Get the current position in a file
int ftell(void *handle) {
  uint64_t position;
  DO_SYSCALL(8, position, (uintptr_t)handle, 0, 0);
  return position;
}

// Read from a file
int fread(void *handle, void *ptr, int size) {
  uint64_t read_bytes;
  DO_SYSCALL(6, read_bytes, (uintptr_t)ptr, size, 0);
  return read_bytes;
}

// Draw the framebuffer
uint64_t draw_framebuffer(const uint8_t *framebuffer) {
  uint64_t result;
  DO_SYSCALL(10, result, (uintptr_t)framebuffer, 0, 0);
  return result;
}

// Switch VGA mode
uint64_t switch_vga_mode(bool vga_on) {
  uint64_t result;
  DO_SYSCALL(11, result, vga_on, 0, 0);
  return result;
}

// Get the state of a key
bool get_keystate(int key) {
  uint64_t state;
  DO_SYSCALL(12, state, key, 0, 0);
  return (bool)state;
}

void get_time(int *sec, int *usec) {
  uint64_t result;
  DO_SYSCALL(13, result, sec, usec, 0);
}

// A helper function to print an integer to the specified FILE stream.
void print_int(FILE *stream, int value) {
  if (value < 0) {
    fputc('-', stream);
    value = -value;
  }
  if (value / 10)
    print_int(stream, value / 10);
  fputc((value % 10) + '0', stream);
}

// A helper function to print a string to the specified FILE stream.
void print_str(FILE *stream, const char *str) {
  while (*str) {
    fputc(*str++, stream);
  }
}

// A simplified fprintf function
int fprintf(FILE *stream, const char *format, ...) {
  va_list args;
  va_start(args, format);
  int count = 0;

  while (*format) {
    if (*format == '%' && *(format + 1)) {
      format++; // Skip '%'

      // Handle the format specifiers
      if (*format == 'd') { // Integer
        int i = va_arg(args, int);
        print_int(stream, i);
      } else if (*format == 's') { // String
        const char *s = va_arg(args, const char *);
        print_str(stream, s);
      } else if (*format == 'c') { // Character
        char c = (char)va_arg(args, int);
        fputc(c, stream);
      } else { // Unknown format, print as-is
        fputc('%', stream);
        fputc(*format, stream);
      }
    } else { // Regular character, print as-is
      fputc(*format, stream);
    }
    format++;
    count++;
  }

  va_end(args);
  return count;
}

// TODO very inefficient, one syscall for each character - use buffered approach
int fputc(int c, FILE *stream) {
  unsigned char ch = (unsigned char)c;

  if (write(stream->fd, &ch, 1) != 1) {
    return EOF;
  }

  return (int)ch;
}

void exit(int exit_code) {
  // TODO implement
}

int sprintf(char *str, const char *format, ...) {
  va_list args;
  va_start(args, format);
  char *ptr = str;
  const char *fmt = format;

  while (*fmt != '\0') {
    if (*fmt == '%') {
      fmt++;
      switch (*fmt) {
      case 'd': {
        int num = va_arg(args, int);
        ptr += sprintf(ptr, "%d", num); // Append integer
        break;
      }
      case 's': {
        char *s = va_arg(args, char *);
        ptr += sprintf(ptr, "%s", s); // Append string
        break;
      }
      case 'c': {
        char c = (char)va_arg(args, int); // Get character
        *ptr++ = c;                       // Append character
        *ptr = '\0';                      // Null-terminate
        break;
      }
      default:
        // Handle unknown format specifiers
        *ptr++ = '%';
        *ptr++ = *fmt;
        *ptr = '\0';
        break;
      }
    } else {
      *ptr++ = *fmt; // Copy regular characters
      *ptr = '\0';   // Null-terminate
    }
    fmt++;
  }

  va_end(args);
  return (int)(ptr - str); // Return the length of the formatted string
}

char *strrchr(char *str, int character) {
  while (*str != '\0') {
    if (*str == character) {
      return str;
    }
    str++;
  }
  return NULL;
}

// Globals for getopt
char *optarg = NULL; // Points to the argument of an option if present
int optind = 1;      // Index in argv, starts at 1 to skip the program name

int getopt(int argc, char *const argv[], const char *optstring) {
  static int optpos = 1; // Position within the current argv element
  char *current_arg;

  if (optind >= argc || argv[optind][0] != '-' || argv[optind][1] == '\0') {
    return -1; // No more options or not an option
  }

  current_arg = argv[optind];

  if (optpos == 1 && current_arg[1] == '-') {
    optind++;
    return -1; // End of options (e.g., "--")
  }

  char opt = current_arg[optpos];
  char *opt_match = strrchr(optstring, opt);

  if (opt_match == NULL) {
    fprintf(stderr, "Unknown option: -%c\n", opt);
    optpos++;
    if (current_arg[optpos] == '\0') {
      optind++;
      optpos = 1;
    }
    return '?'; // Return '?' for unknown option
  }

  if (*(opt_match + 1) == ':') {
    // Option requires an argument
    if (current_arg[optpos + 1] != '\0') {
      // Argument is in the same argv element (e.g., "-oValue")
      optarg = &current_arg[optpos + 1];
      optind++;
      optpos = 1;
    } else if (optind + 1 < argc) {
      // Argument is in the next argv element (e.g., "-o Value")
      optarg = argv[++optind];
      optind++;
      optpos = 1;
    } else {
      fprintf(stderr, "Option -%c requires an argument\n", opt);
      optpos = 1;
      optind++;
      return '?';
    }
  } else {
    // Option does not require an argument
    optarg = NULL;
    optpos++;
    if (current_arg[optpos] == '\0') {
      optind++;
      optpos = 1;
    }
  }

  return opt;
}

void *memcpy(void *dest, const void *src, size_t n) {
  char *d = dest;
  const char *s = src;

  while (n--) {
    *d++ = *s++;
  }

  return dest;
}

void *mempcpy(void *dest, const void *src, size_t n) {
  return (char *)memcpy(dest, src, n) + n;
}

// implement strchr, strchrnul, strstr, strcspn
char *strchr(const char *s, int c) {
  while (*s != '\0') {
    if (*s == c) {
      return (char *)s;
    }
    s++;
  }
  return NULL;
}

char *strchrnul(const char *s, int c) {
  while (*s != '\0') {
    if (*s == c) {
      return (char *)s;
    }
    s++;
  }
  return (char *)s;
}

// implement strncmp
int strncmp(const char *s1, const char *s2, size_t n) {
  while (n--) {
    if (*s1 != *s2) {
      return *s1 - *s2;
    }
    if (*s1 == '\0') {
      return 0;
    }
    s1++;
    s2++;
  }

  return 0;
}

char *strstr(const char *haystack, const char *needle) {
  size_t needle_len = strlen(needle);

  while (*haystack != '\0') {
    if (strncmp(haystack, needle, needle_len) == 0) {
      return (char *)haystack;
    }
    haystack++;
  }

  return NULL;
}

size_t strcspn(const char *s, const char *reject) {
  size_t count = 0;

  while (*s != '\0') {
    if (strchr(reject, *s) != NULL) {
      return count;
    }
    s++;
    count++;
  }

  return count;
}

// implement strspn
size_t strspn(const char *s, const char *accept) {
  size_t count = 0;

  while (*s != '\0') {
    if (strchr(accept, *s) == NULL) {
      return count;
    }
    s++;
    count++;
  }

  return count;
}

// implement strtod
double strtod(const char *nptr, char **endptr) {
  double result = 0.0;
  int sign = 1;
  int decimal = 0;
  int exponent = 0;
  int exponent_sign = 1;
  int exponent_value = 0;
  int exponent_multiplier = 1;

  // Skip leading whitespace
  while (*nptr == ' ' || *nptr == '\t') {
    nptr++;
  }

  // Parse sign
  if (*nptr == '-') {
    sign = -1;
    nptr++;
  } else if (*nptr == '+') {
    nptr++;
  }

  // Parse integer part
  while (*nptr >= '0' && *nptr <= '9') {
    result = result * 10 + (*nptr - '0');
    nptr++;
  }

  // Parse decimal part
  if (*nptr == '.') {
    nptr++;
    while (*nptr >= '0' && *nptr <= '9') {
      result = result * 10 + (*nptr - '0');
      decimal++;
      nptr++;
    }
  }

  // Parse exponent part
  if (*nptr == 'e' || *nptr == 'E') {
    nptr++;
    if (*nptr == '-') {
      exponent_sign = -1;
      nptr++;
    } else if (*nptr == '+') {
      nptr++;
    }
    while (*nptr >= '0' && *nptr <= '9') {
      exponent_value = exponent_value * 10 + (*nptr - '0');
      nptr++;
    }
  }

  // Calculate the exponent multiplier
  while (exponent_value--) {
    exponent_multiplier *= 10;
  }

  // Calculate the final result
  result = sign * result;
  if (decimal) {
    result /= exponent_multiplier;
  }
  if (exponent_sign == -1) {
    result /= exponent_multiplier;
  } else {
    result *= exponent_multiplier;
  }

  if (endptr) {
    *endptr = (char *)nptr;
  }

  return result;
}

// implement strtoimax
intmax_t strtoimax(const char *nptr, char **endptr, int base) {
  intmax_t result = 0;
  int sign = 1;

  // Skip leading whitespace
  while (*nptr == ' ' || *nptr == '\t') {
    nptr++;
  }

  // Parse sign
  if (*nptr == '-') {
    sign = -1;
    nptr++;
  } else if (*nptr == '+') {
    nptr++;
  }

  // Parse integer part
  while (*nptr >= '0' && *nptr <= '9') {
    result = result * base + (*nptr - '0');
    nptr++;
  }

  if (endptr) {
    *endptr = (char *)nptr;
  }

  return sign * result;
}

// implement strcpy, strpcy, strncpy
char *strcpy(char *dest, const char *src) {
  char *d = dest;

  while (*src != '\0') {
    *d++ = *src++;
  }

  *d = '\0';

  return dest;
}

char *strncpy(char *dest, const char *src, size_t n) {
  char *d = dest;

  while (n-- && *src != '\0') {
    *d++ = *src++;
  }

  *d = '\0';

  return dest;
}

// implement strpcpy
char *strpcpy(char *dest, const char *src) {
  char *d = dest;

  while (*src != '\0') {
    *d++ = *src++;
  }

  *d = '\0';

  return d;
}

// implement stpncpy
char *stpncpy(char *dest, const char *src, size_t n) {
  char *d = dest;

  while (n-- && *src != '\0') {
    *d++ = *src++;
  }

  *d = '\0';

  return d;
}

// implement strtoumax
uintmax_t strtoumax(const char *nptr, char **endptr, int base) {
  uintmax_t result = 0;

  // Skip leading whitespace
  while (*nptr == ' ' || *nptr == '\t') {
    nptr++;
  }

  // Parse integer part
  while (*nptr >= '0' && *nptr <= '9') {
    result = result * base + (*nptr - '0');
    nptr++;
  }

  if (endptr) {
    *endptr = (char *)nptr;
  }

  return result;
}

// implement stpcpy
char *stpcpy(char *dest, const char *src) {
  while (*src != '\0') {
    *dest++ = *src++;
  }

  *dest = '\0';

  return dest;
}

// implement strcasecmp
int strcasecmp(const char *s1, const char *s2) {
  while (*s1 != '\0' && *s2 != '\0') {
    char c1 = *s1++;
    char c2 = *s2++;
    if (c1 >= 'A' && c1 <= 'Z') {
      c1 += 32;
    }
    if (c2 >= 'A' && c2 <= 'Z') {
      c2 += 32;
    }
    if (c1 != c2) {
      return c1 - c2;
    }
  }

  return *s1 - *s2;
}

// implement strerror
char *strerror(int errnum) {
  switch (errnum) {
  case 0:
    return "Success";
  case 1:
    return "Operation not permitted";
  case 2:
    return "No such file or directory";
  case 3:
    return "No such process";
  case 4:
    return "Interrupted system call";
  case 5:
    return "I/O error";
  case 6:
    return "No such device or address";
  case 7:
    return "Argument list too long";
  case 8:
    return "Exec format error";
  case 9:
    return "Bad file number";
  case 10:
    return "No child processes";
  case 11:
    return "Try again";
  case 12:
    return "Out of memory";
  case 13:
    return "Permission denied";
  default:
    return "Unknown error";
  }
}

// Locates the first occurrence in the string s of any of the bytes in the
// string accept.
char *strpbrk(const char *s, const char *accept) {
  while (*s != '\0') {
    if (strchr(accept, *s) != NULL) {
      return (char *)s;
    }
    s++;
  }

  return NULL;
}

// implement strtok
char *strtok(char *str, const char *delim) {
  static char *last = NULL;
  if (str != NULL) {
    last = str;
  } else if (last == NULL) {
    return NULL;
  }

  char *start = last;
  while (*last != '\0') {
    if (strchr(delim, *last) != NULL) {
      *last = '\0';
      last++;
      return start;
    }
    last++;
  }

  last = NULL;
  return start;
}

// Mock close
int close(int fd) {
  // TODO: Implement the close function
  return 0;
}

// implement bsearch
void *bsearch(const void *key, const void *base, size_t nmemb, size_t size,
              int (*compar)(const void *, const void *)) {
  size_t left = 0;
  size_t right = nmemb - 1;

  while (left <= right) {
    size_t middle = left + (right - left) / 2;
    void *middle_element = (char *)base + middle * size;
    int comparison = compar(key, middle_element);

    if (comparison == 0) {
      return middle_element;
    } else if (comparison < 0) {
      right = middle - 1;
    } else {
      left = middle + 1;
    }
  }

  return NULL;
}

// implement atoi
int atoi(const char *nptr) {
  int result = 0;
  int sign = 1;

  // Skip leading whitespace
  while (*nptr == ' ' || *nptr == '\t') {
    nptr++;
  }

  // Parse sign
  if (*nptr == '-') {
    sign = -1;
    nptr++;
  } else if (*nptr == '+') {
    nptr++;
  }

  // Parse integer part
  while (*nptr >= '0' && *nptr <= '9') {
    result = result * 10 + (*nptr - '0');
    nptr++;
  }

  return sign * result;
}

// implement memmove
void *memmove(void *dest, const void *src, size_t n) {
  char *d = dest;
  const char *s = src;

  if (d < s) {
    while (n--) {
      *d++ = *s++;
    }
  } else {
    d += n;
    s += n;
    while (n--) {
      *--d = *--s;
    }
  }

  return dest;
}

// implement memset
void *memset(void *s, int c, size_t n) {
  unsigned char *p = s;
  unsigned char value = c;

  while (n--) {
    *p++ = value;
  }

  return s;
}

// implement strdup
char *strdup(const char *s) {
  size_t len = strlen(s) + 1;
  char *new_s = malloc(len);
  if (new_s == NULL) {
    return NULL;
  }

  return (char *)memcpy(new_s, s, len);
}

int stat(const char *pathname, struct stat *statbuf) {
  int result;
  DO_SYSCALL(14, result, pathname, statbuf, 0);
  return result;
}

// lstat() is identical to stat(), except that if path is a symbolic link, then
// the link itself is stat-ed, not the file that it refers to.
int lstat(const char *pathname, struct stat *statbuf) {
  // TODO different handling for symbolic links
  return stat(pathname, statbuf);
}

int chdir(const char *path) {
  int result;
  DO_SYSCALL(15, result, path, 0, 0);
  return result;
}

char *getcwd(char *buf, size_t size) {
  char result;
  DO_SYSCALL(16, result, buf, size, 0);

  if (result == 0) {
    return NULL;
  } else {
    return buf;
  }
}

int memcmp(const void *s1, const void *s2, size_t n) {
  const unsigned char *p1 = s1;
  const unsigned char *p2 = s2;

  while (n--) {
    if (*p1 != *p2) {
      return *p1 - *p2;
    }
    p1++;
    p2++;
  }

  return 0;
}

void qsort(void *base, size_t num, size_t size,
           int (*compar)(const void *, const void *)) {
  if (num < 2) {
    return; // No need to sort
  }

  char *pivot =
      (char *)base + (num - 1) * size; // Choose the last element as pivot
  size_t i = 0;                        // Index of smaller element

  for (size_t j = 0; j < num - 1; j++) {
    char *current = (char *)base + j * size;
    if (compar(current, pivot) < 0) {
      if (i != j) {
        char *smaller = (char *)base + i * size;
        // Swap current and smaller
        for (size_t k = 0; k < size; k++) {
          char temp = smaller[k];
          smaller[k] = current[k];
          current[k] = temp;
        }
      }
      i++;
    }
  }

  // Place pivot in the correct position
  char *smaller = (char *)base + i * size;
  for (size_t k = 0; k < size; k++) {
    char temp = smaller[k];
    smaller[k] = pivot[k];
    pivot[k] = temp;
  }

  // Recursively sort elements before and after partition
  qsort(base, i, size, compar);
  qsort((char *)base + (i + 1) * size, num - i - 1, size, compar);
}

pid_t getppid(void) {
  uint64_t ppid;
  DO_SYSCALL(17, ppid, 0, 0, 0);
  return ppid;
}

char **environ = NULL;

uid_t getuid(void) {
  // TODO implement
  return 1234;
}

uid_t geteuid(void) {
  // TODO implement
  return 12345;
}

clock_t times(struct tms *buf) {
  // TODO implement
  buf->tms_utime = 0;
  buf->tms_stime = 0;
  buf->tms_cutime = 0;
  buf->tms_cstime = 0;
  return 0;
}

long sysconf(int name) {
  // TODO implement
  return -1;
}
