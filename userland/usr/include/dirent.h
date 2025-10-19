#include "sys/stat.h"

/* File types for `d_type'.  */
#define DT_UNKNOWN 0
#define DT_FIFO 1
#define DT_CHR 2
#define DT_DIR 4
#define DT_BLK 6
#define DT_REG 8
#define DT_LNK 10
#define DT_SOCK 12
#define DT_WHT 14

struct dirent64 {
#ifndef __USE_FILE_OFFSET64
  ino_t d_ino;
  off_t d_off;
#else
  __ino64_t d_ino;
  __off64_t d_off;
#endif
  unsigned short int d_reclen;
  unsigned char d_type;
  char d_name[256]; /* We must not include limits.h! */
};

typedef struct dirent64 DIR;

DIR *opendir(const char *name);
DIR *readdir64(DIR *dirp);
int closedir(DIR *dirp);