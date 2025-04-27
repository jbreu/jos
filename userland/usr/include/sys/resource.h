#ifndef __RESOURCE_H__
#define __RESOURCE_H__

typedef unsigned long rlim_t;

#define RLIM_INFINITY (~(rlim_t)0)

struct rlimit {
    rlim_t rlim_cur; /* Soft limit: current limit */
    rlim_t rlim_max; /* Hard limit: maximum value for rlim_cur */
};

int getrlimit(int resource, struct rlimit *rlim);
int setrlimit(int resource, const struct rlimit *rlim);

#endif