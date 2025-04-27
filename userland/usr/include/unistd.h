#include "sys/stat.h"
#include "stddef.h"
#include "ctype.h"
#include "chartypes.h"


typedef int pid_t;

pid_t getpid(void);
pid_t getppid(void);

ssize_t read(int fd, void *buf, size_t count);
int close(int fd);

int execve(const char *pathname, char *const argv[], char *const envp[]);

char *getcwd(char buf[], size_t size);

int chdir(const char *path);

// Replace [[noreturn]] with __attribute__((noreturn))
void _exit(int status) __attribute__((noreturn));

int pipe(int pipefd[2]);
int dup2(int oldfd, int newfd);

int isatty(int fd);
pid_t getpgrp(void);
pid_t tcgetpgrp(int fd);
int tcsetpgrp(int fd, pid_t pgrp);

int setpgid(pid_t pid, pid_t pgid);

pid_t fork(void);
pid_t vfork(void);

uid_t geteuid(void);
gid_t getegid(void);

ssize_t write(int fd, const void* buf, size_t count);

/*  These may be OR'd together.  */
#define R_OK    4       /* Test for read permission.  */
#define W_OK    2       /* Test for write permission.  */
#define X_OK    1       /* execute permission - unsupported in windows*/
#define F_OK    0       /* Test for existence.  */

int faccessat(int dirfd, const char *pathname, int mode, int flags);

#define AT_FDCWD    -100    /* Special value used to indicate openat should use the current working directory. */
#define AT_EACCESS  0x200    /* Test access permissions for a file relative to a directory.  */

long sysconf(int name);

#define _SC_CLK_TCK 100