#ifndef __SIGNAL_H__
#define __SIGNAL_H__

#include "stddef.h"
#include "unistd.h"

#define NSIG                32

/* ISO C99 signals.  */
#define SIGHUP		1	/* Hangup.  */
#define	SIGINT		2	/* Interactive attention signal.  */
#define	SIGILL		4	/* Illegal instruction.  */
#define	SIGABRT		6	/* Abnormal termination.  */
#define	SIGFPE		8	/* Erroneous arithmetic operation.  */
#define	SIGSEGV		11	/* Invalid access to storage.  */
#define	SIGALRM		14	/* Alarm clock.  */
#define	SIGTERM		15	/* Termination request.  */
#define SIGCHLD		17 /* Child stopped or terminated.  */
#define SIGCONT		19 /* Continue a stopped process.  */
#define SIGTTIN		21 /* Background process attempting to read.  */
#define SIGTTOU		22 /* Background process attempting to write.  */
#define SIGTSTP		23 /* Background process attempting to stop.  */


/* POSIX signals */
#define SIGQUIT     3
#define SIGKILL     9
#define SIGPIPE     13

#define SIG_IGN ((void (*)(int))1)
#define SIG_DFL ((void (*)(int))0)

#define SIG_SETMASK 2

typedef int sig_atomic_t;

#define _SIGSET_NWORDS (1024 / (8 * sizeof (unsigned long int)))
typedef struct
{
  unsigned long int __val[_SIGSET_NWORDS];
} sigset_t;

int sigsetmask(int mask); 
void *signal(int, void (*)(int));

typedef union sigval {
    int sival_int;    // Integer value
    void *sival_ptr;  // Pointer value
} sigval_t;


typedef struct siginfo {
    int si_signo;   // Signal number
    int si_errno;   // Error number associated with the signal (if applicable)
    int si_code;    // Signal-specific code (provides more information)
    pid_t si_pid;   // PID of the sending process (if sent by another process)
    uid_t si_uid;   // UID of the sending process (if sent by another process)
    void *si_addr;  // Address at which fault occurred (for hardware-generated signals)
    int si_status;  // Exit value or signal for child (if SIGCHLD)
    long si_band;   // Band event (for SIGPOLL/SIGIO)
    union sigval si_value; // Signal value (for real-time signals)
} siginfo_t;


struct sigaction {
    void (*sa_handler)(int);         // Pointer to a signal handler function
    void (*sa_sigaction)(int, siginfo_t *, void *); // Alternative handler with more details
    sigset_t sa_mask;               // Signals to block during handler execution
    int sa_flags;                   // Flags to modify signal handling behavior
};


int sigaction(int signum, const struct sigaction *act, struct sigaction *oldact);
int sigsuspend(const sigset_t *mask);
int sigfillset(sigset_t *set);

int sigprocmask(int how, const sigset_t *  set, sigset_t * oldset);

int raise(int sig);
int kill(pid_t pid, int sig);
int killpg(int pgrp, int sig);



#endif