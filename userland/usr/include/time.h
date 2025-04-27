#ifndef __TIME_H__
#define __TIME_H__

typedef unsigned long int time_t;

struct timeval
{
  time_t tv_sec;
  long int tv_usec;
};

#endif