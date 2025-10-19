#include "sys/stat.h"

struct passwd {
  char *pw_name;   /* user name */
  char *pw_passwd; /* user password */
  uid_t pw_uid;    /* user ID */
  gid_t pw_gid;    /* group ID */
  char *pw_gecos;  /* real name */
  char *pw_dir;    /* home directory */
  char *pw_shell;  /* shell program */
};

struct passwd *getpwnam(const char *name);