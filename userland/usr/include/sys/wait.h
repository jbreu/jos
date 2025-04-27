#ifndef __WAIT_H__
#define __WAIT_H__


#include "../unistd.h"
#include "../time.h"

#define	WNOHANG		1	/* Don't block waiting.  */
#define	WUNTRACED	2	/* Report status of stopped children.  */

/* If WIFEXITED(STATUS), the low-order 8 bits of the status.  */
#define WEXITSTATUS(status)	(((status) & 0xff00) >> 8)
/* If WIFSIGNALED(STATUS), the terminating signal.  */
#define	WTERMSIG(status)	((status) & 0x7f)
/* If WIFSTOPPED(STATUS), the signal that stopped the child.  */
#define	WSTOPSIG(status)	WEXITSTATUS(status)
/* Nonzero if STATUS indicates normal termination.  */
#define	WIFEXITED(status)	(WTERMSIG(status) == 0)
/* Nonzero if STATUS indicates termination by a signal.  */
#define WIFSIGNALED(status) \
  (((signed char) (((status) & 0x7f) + 1) >> 1) > 0)
/* Nonzero if STATUS indicates the child is stopped.  */
#define	WIFSTOPPED(status)	(((status) & 0xff) == 0x7f)
/* Nonzero if STATUS indicates the child continued after a stop.  We only
   define this if <bits/waitflags.h> provides the WCONTINUED flag bit.  */
#ifdef WCONTINUED
# define WIFCONTINUED(status)	((status) == W_CONTINUED)
#endif
/* Nonzero if STATUS indicates the child dumped core.  */
#define	WCOREDUMP(status)	((status) & WCOREFLAG)
/* Macros for constructing status values.  */
#define	W_EXITCODE(ret, sig)	((ret) << 8 | (sig))
#define	W_STOPCODE(sig)	((sig) << 8 | 0x7f)
#define W_CONTINUED		0xffff
#define	WCOREFLAG		0x80

struct rusage {
    struct timeval ru_utime; /* user CPU time used */
    struct timeval ru_stime; /* system CPU time used */
    long   ru_maxrss;        /* maximum resident set size */
    long   ru_ixrss;         /* integral shared memory size */
    long   ru_idrss;         /* integral unshared data size */
    long   ru_isrss;         /* integral unshared stack size */
    long   ru_minflt;        /* page reclaims (soft page faults) */
    long   ru_majflt;        /* page faults (hard page faults) */
    long   ru_nswap;         /* swaps */
    long   ru_inblock;       /* block input operations */
    long   ru_oublock;       /* block output operations */
    long   ru_msgsnd;        /* messages sent */
    long   ru_msgrcv;        /* messages received */
    long   ru_nsignals;      /* signals received */
    long   ru_nvcsw;         /* voluntary context switches */
    long   ru_nivcsw;        /* involuntary context switches */
};


pid_t wait3(int *status, int options, struct rusage *rusage);


#endif