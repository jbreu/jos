#ifndef __SGTTY_H__
#define __SGTTY_H__

struct sgttyb {
    char    sg_ispeed;    /* Input speed (baud rate) */
    char    sg_ospeed;    /* Output speed (baud rate) */
    char    sg_erase;     /* Erase character */
    char    sg_kill;      /* Kill line character */
    int     sg_flags;     /* Mode flags (e.g., echo, raw mode, etc.) */
};

#endif