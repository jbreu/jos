typedef long long clock_t;  // 64-bit version for high precision

struct tms {
    clock_t tms_utime;  /* User time */
    clock_t tms_stime;  /* System time */
    clock_t tms_cutime; /* User time of terminated children */
    clock_t tms_cstime; /* System time of terminated children */
};

clock_t times(struct tms *buf);