#include "sys/stat.h"

struct dirent64 
  {
#ifndef __USE_FILE_OFFSET64
    ino_t d_ino;
    off_t d_off;
#else
    __ino64_t d_ino;
    __off64_t d_off;
#endif
    unsigned short int d_reclen;
    unsigned char d_type;
    char d_name[256];		/* We must not include limits.h! */
  };

typedef struct dirent64 DIR;

DIR *opendir(const char *name);
DIR *readdir64(DIR *dirp);
int closedir(DIR *dirp);