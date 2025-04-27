#ifndef _SYS_STAT_H
#define _SYS_STAT_H

#include "../time.h"

typedef unsigned long long int dev_t;
typedef unsigned long long int ino_t;
typedef unsigned long long int mode_t;
typedef unsigned long long int nlink_t;
typedef unsigned long long int uid_t;
typedef unsigned long long int gid_t;
typedef unsigned long long int off_t;
typedef unsigned long long int blksize_t;
typedef unsigned long long int blkcnt_t;

struct stat {
    dev_t     st_dev;     /* ID of device containing file */
    ino_t     st_ino;     /* inode number */
    mode_t    st_mode;    /* protection */
    nlink_t   st_nlink;   /* number of hard links */
    uid_t     st_uid;     /* user ID of owner */
    gid_t     st_gid;     /* group ID of owner */
    dev_t     st_rdev;    /* device ID (if special file) */
    off_t     st_size;    /* total size, in bytes */
    blksize_t st_blksize; /* blocksize for file system I/O */
    blkcnt_t  st_blocks;  /* number of 512B blocks allocated */
    time_t    st_atime;   /* time of last access */
    time_t    st_mtime;   /* time of last modification */
    time_t    st_ctime;   /* time of last status change */
};

int stat(const char * pathname, struct stat * statbuf);
int lstat(const char *path, struct stat *buf);
int fstat(int fildes, struct stat *buf);
int open(const char *pathname, int flags, ...);
int open64(const char *pathname, int oflag,...); 
int mkdir(const char *path, mode_t mode);
mode_t umask(mode_t mask);


#define 	S_IFMT   00170000
#define 	S_IFSOCK   0140000
#define 	S_IFLNK   0120000
#define 	S_IFREG   0100000
#define 	S_IFBLK   0060000
#define 	S_IFDIR   0040000
#define 	S_IFCHR   0020000
#define 	S_IFIFO   0010000
#define 	S_ISUID   0004000
#define 	S_ISGID   0002000
#define 	S_ISVTX   0001000
#define 	S_ISLNK(m)   (((m) & S_IFMT) == S_IFLNK)
#define 	S_ISREG(m)   (((m) & S_IFMT) == S_IFREG)
#define 	S_ISDIR(m)   (((m) & S_IFMT) == S_IFDIR)
#define 	S_ISCHR(m)   (((m) & S_IFMT) == S_IFCHR)
#define 	S_ISBLK(m)   (((m) & S_IFMT) == S_IFBLK)
#define 	S_ISFIFO(m)   (((m) & S_IFMT) == S_IFIFO)
#define 	S_ISSOCK(m)   (((m) & S_IFMT) == S_IFSOCK)
#define 	S_IRWXU   00700
#define 	S_IRUSR   00400
#define 	S_IWUSR   00200
#define 	S_IXUSR   00100
#define 	S_IRWXG   00070
#define 	S_IRGRP   00040
#define 	S_IWGRP   00020
#define 	S_IXGRP   00010
#define 	S_IRWXO   00007
#define 	S_IROTH   00004
#define 	S_IWOTH   00002
#define 	S_IXOTH   00001

#endif