typedef unsigned int tcflag_t;
typedef unsigned char cc_t;

/* c_iflag bits */
#define IGNBRK 0000001
#define BRKINT 0000002
#define IGNPAR 0000004
#define PARMRK 0000010
#define INPCK 0000020
#define ISTRIP 0000040
#define INLCR 0000100
#define IGNCR 0000200
#define ICRNL 0000400
#define IXON 0002000
#define IXOFF 0010000

/* c_oflag bits */
#define OPOST 0000001
#define ONLCR 0000004
#define OCRNL 0000010

/* c_lflag bits */
#define ISIG 0000001
#define ICANON 0000002
#define ECHO 0000010
#define ECHOE 0000020
#define ECHOK 0000040
#define ECHONL 0000100
#define NOFLSH 0000200
#define TOSTOP 0000400
#define IEXTEN 0100000

/* c_cflag bits */
#define CSIZE 0000060
#define CS8 0000060
#define PARENB 0000400
#define PARODD 0001000
#define HUPCL 0002000
#define CLOCAL 0004000

struct termios {
  tcflag_t c_iflag; /* input mode flags */
  tcflag_t c_oflag; /* output mode flags */
  tcflag_t c_cflag; /* control mode flags */
  tcflag_t c_lflag; /* local mode flags */
  cc_t c_line;      /* line discipline */
  cc_t c_cc[];      /* control characters */
};

int tcgetattr(int fd, struct termios *termios_p);