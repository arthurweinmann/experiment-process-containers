# Passing descriptors (file, ...) between processes

From https://stackoverflow.com/questions/28003921/sending-file-descriptor-by-linux-socket

41

Stevens (et al) UNIX® Network Programming, Vol 1: The Sockets Networking API describes the process of transferring file descriptors between processes in Chapter 15 Unix Domain Protocols and specifically §15.7 Passing Descriptors. It's fiddly to describe in full, but it must be done on a Unix domain socket (AF_UNIX or AF_LOCAL), and the sender process uses sendmsg() while the receiver uses recvmsg().

I got this mildly modified (and instrumented) version of the code from the question to work for me on Mac OS X 10.10.1 Yosemite with GCC 4.9.1:

```C
#include "stderr.h"
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

static
void wyslij(int socket, int fd)  // send fd by socket
{
    struct msghdr msg = { 0 };
    char buf[CMSG_SPACE(sizeof(fd))];
    memset(buf, '\0', sizeof(buf));
    struct iovec io = { .iov_base = "ABC", .iov_len = 3 };

    msg.msg_iov = &io;
    msg.msg_iovlen = 1;
    msg.msg_control = buf;
    msg.msg_controllen = sizeof(buf);

    struct cmsghdr * cmsg = CMSG_FIRSTHDR(&msg);
    cmsg->cmsg_level = SOL_SOCKET;
    cmsg->cmsg_type = SCM_RIGHTS;
    cmsg->cmsg_len = CMSG_LEN(sizeof(fd));

    *((int *) CMSG_DATA(cmsg)) = fd;

    msg.msg_controllen = CMSG_SPACE(sizeof(fd));

    if (sendmsg(socket, &msg, 0) < 0)
        err_syserr("Failed to send message\n");
}

static
int odbierz(int socket)  // receive fd from socket
{
    struct msghdr msg = {0};

    char m_buffer[256];
    struct iovec io = { .iov_base = m_buffer, .iov_len = sizeof(m_buffer) };
    msg.msg_iov = &io;
    msg.msg_iovlen = 1;

    char c_buffer[256];
    msg.msg_control = c_buffer;
    msg.msg_controllen = sizeof(c_buffer);

    if (recvmsg(socket, &msg, 0) < 0)
        err_syserr("Failed to receive message\n");

    struct cmsghdr * cmsg = CMSG_FIRSTHDR(&msg);

    unsigned char * data = CMSG_DATA(cmsg);

    err_remark("About to extract fd\n");
    int fd = *((int*) data);
    err_remark("Extracted fd %d\n", fd);

    return fd;
}

int main(int argc, char **argv)
{
    const char *filename = "./z7.c";

    err_setarg0(argv[0]);
    err_setlogopts(ERR_PID);
    if (argc > 1)
        filename = argv[1];
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, sv) != 0)
        err_syserr("Failed to create Unix-domain socket pair\n");

    int pid = fork();
    if (pid > 0)  // in parent
    {
        err_remark("Parent at work\n");
        close(sv[1]);
        int sock = sv[0];

        int fd = open(filename, O_RDONLY);
        if (fd < 0)
            err_syserr("Failed to open file %s for reading\n", filename);

        wyslij(sock, fd);

        close(fd);
        nanosleep(&(struct timespec){ .tv_sec = 1, .tv_nsec = 500000000}, 0);
        err_remark("Parent exits\n");
    }
    else  // in child
    {
        err_remark("Child at play\n");
        close(sv[0]);
        int sock = sv[1];

        nanosleep(&(struct timespec){ .tv_sec = 0, .tv_nsec = 500000000}, 0);

        int fd = odbierz(sock);
        printf("Read %d!\n", fd);
        char buffer[256];
        ssize_t nbytes;
        while ((nbytes = read(fd, buffer, sizeof(buffer))) > 0)
            write(1, buffer, nbytes);
        printf("Done!\n");
        close(fd);
    }
    return 0;
}
```



Stevens (et al) UNIX® Network Programming, Vol 1: The Sockets Networking API describes the process of transferring file descriptors between processes in Chapter 15 Unix Domain Protocols and specifically §15.7 Passing Descriptors. It's fiddly to describe in full, but it must be done on a Unix domain socket (AF_UNIX or AF_LOCAL), and the sender process uses sendmsg() while the receiver uses recvmsg().

I got this mildly modified (and instrumented) version of the code from the question to work for me on Mac OS X 10.10.1 Yosemite with GCC 4.9.1:

#include "stderr.h"
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

static
void wyslij(int socket, int fd)  // send fd by socket
{
    struct msghdr msg = { 0 };
    char buf[CMSG_SPACE(sizeof(fd))];
    memset(buf, '\0', sizeof(buf));
    struct iovec io = { .iov_base = "ABC", .iov_len = 3 };

    msg.msg_iov = &io;
    msg.msg_iovlen = 1;
    msg.msg_control = buf;
    msg.msg_controllen = sizeof(buf);

    struct cmsghdr * cmsg = CMSG_FIRSTHDR(&msg);
    cmsg->cmsg_level = SOL_SOCKET;
    cmsg->cmsg_type = SCM_RIGHTS;
    cmsg->cmsg_len = CMSG_LEN(sizeof(fd));

    *((int *) CMSG_DATA(cmsg)) = fd;

    msg.msg_controllen = CMSG_SPACE(sizeof(fd));

    if (sendmsg(socket, &msg, 0) < 0)
        err_syserr("Failed to send message\n");
}

static
int odbierz(int socket)  // receive fd from socket
{
    struct msghdr msg = {0};

    char m_buffer[256];
    struct iovec io = { .iov_base = m_buffer, .iov_len = sizeof(m_buffer) };
    msg.msg_iov = &io;
    msg.msg_iovlen = 1;

    char c_buffer[256];
    msg.msg_control = c_buffer;
    msg.msg_controllen = sizeof(c_buffer);

    if (recvmsg(socket, &msg, 0) < 0)
        err_syserr("Failed to receive message\n");

    struct cmsghdr * cmsg = CMSG_FIRSTHDR(&msg);

    unsigned char * data = CMSG_DATA(cmsg);

    err_remark("About to extract fd\n");
    int fd = *((int*) data);
    err_remark("Extracted fd %d\n", fd);

    return fd;
}

int main(int argc, char **argv)
{
    const char *filename = "./z7.c";

    err_setarg0(argv[0]);
    err_setlogopts(ERR_PID);
    if (argc > 1)
        filename = argv[1];
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, sv) != 0)
        err_syserr("Failed to create Unix-domain socket pair\n");

    int pid = fork();
    if (pid > 0)  // in parent
    {
        err_remark("Parent at work\n");
        close(sv[1]);
        int sock = sv[0];

        int fd = open(filename, O_RDONLY);
        if (fd < 0)
            err_syserr("Failed to open file %s for reading\n", filename);

        wyslij(sock, fd);

        close(fd);
        nanosleep(&(struct timespec){ .tv_sec = 1, .tv_nsec = 500000000}, 0);
        err_remark("Parent exits\n");
    }
    else  // in child
    {
        err_remark("Child at play\n");
        close(sv[0]);
        int sock = sv[1];

        nanosleep(&(struct timespec){ .tv_sec = 0, .tv_nsec = 500000000}, 0);

        int fd = odbierz(sock);
        printf("Read %d!\n", fd);
        char buffer[256];
        ssize_t nbytes;
        while ((nbytes = read(fd, buffer, sizeof(buffer))) > 0)
            write(1, buffer, nbytes);
        printf("Done!\n");
        close(fd);
    }
    return 0;
}

The output from the instrumented but unfixed version of the original code was:

```
$ ./fd-passing
fd-passing: pid=1391: Parent at work
fd-passing: pid=1391: Failed to send message
error (40) Message too long
fd-passing: pid=1392: Child at play
$ fd-passing: pid=1392: Failed to receive message
error (40) Message too long
```

Note that the parent finished before the child, so the prompt appeared in the middle of the output.

The output from the 'fixed' code was:

```
$ ./fd-passing
fd-passing: pid=1046: Parent at work
fd-passing: pid=1048: Child at play
fd-passing: pid=1048: About to extract fd
fd-passing: pid=1048: Extracted fd 3
Read 3!
This is the file z7.c.
It isn't very interesting.
It isn't even C code.
But it is used by the fd-passing program to demonstrate that file
descriptors can indeed be passed between sockets on occasion.
Done!
fd-passing: pid=1046: Parent exits
$
```

The primary significant changes were adding the struct iovec to the data in the struct msghdr in both functions, and providing space in the receive function (odbierz()) for the control message. I reported an intermediate step in debugging where I provided the struct iovec to the parent and the parent's "message too long" error was removed. To prove it was working (a file descriptor was passed), I added code to read and print the file from the passed file descriptor. The original code had sleep(0.5) but since sleep() takes an unsigned integer, this was equivalent to not sleeping. I used C99 compound literals to have the child sleep for 0.5 seconds. The parent sleeps for 1.5 seconds so that the output from the child is complete before the parent exits. I could use wait() or waitpid() too, but was too lazy to do so.

I have not gone back and checked that all the additions were necessary.

The "stderr.h" header declares the err_*() functions. It's code I wrote (first version before 1987) to report errors succinctly. The err_setlogopts(ERR_PID) call prefixes all messages with the PID. For timestamps too, err_setlogopts(ERR_PID|ERR_STAMP) would do the job.
Alignment issues

Nominal Animal suggests in a comment:

    May I suggest you modify the code to copy the descriptor int using memcpy() instead of accessing the data directly? It is not necessarily correctly aligned — which is why the man page example also uses memcpy() — and there are many Linux architectures where unaligned int access causes problems (up to SIGBUS signal killing the process).

And not only Linux architectures: both SPARC and Power require aligned data and often run Solaris and AIX respectively. Once upon a time, DEC Alpha required that too, but they're seldom seen in the field these days.

The code in the manual page cmsg(3) related to this is:

```C
struct msghdr msg = {0};
struct cmsghdr *cmsg;
int myfds[NUM_FD]; /* Contains the file descriptors to pass. */
char buf[CMSG_SPACE(sizeof myfds)];  /* ancillary data buffer */
int *fdptr;

msg.msg_control = buf;
msg.msg_controllen = sizeof buf;
cmsg = CMSG_FIRSTHDR(&msg);
cmsg->cmsg_level = SOL_SOCKET;
cmsg->cmsg_type = SCM_RIGHTS;
cmsg->cmsg_len = CMSG_LEN(sizeof(int) * NUM_FD);
/* Initialize the payload: */
fdptr = (int *) CMSG_DATA(cmsg);
memcpy(fdptr, myfds, NUM_FD * sizeof(int));
/* Sum of the length of all control messages in the buffer: */
msg.msg_controllen = CMSG_SPACE(sizeof(int) * NUM_FD);
```

The assignment to fdptr appears to assume that CMSG_DATA(cmsg) is sufficiently well aligned to be converted to an int * and the memcpy() is used on the assumption that NUM_FD is not just 1. With that said, it is supposed to be pointing at the array buf, and that might not be sufficiently well aligned as Nominal Animal suggests, so it seems to me that the fdptr is just an interloper and it would be better if the example used:

```C
memcpy(CMSG_DATA(cmsg), myfds, NUM_FD * sizeof(int));
```

And the reverse process on the receiving end would then be appropriate. This program only passes a single file descriptor, so the code is modifiable to:

```C
memmove(CMSG_DATA(cmsg), &fd, sizeof(fd));  // Send
memmove(&fd, CMSG_DATA(cmsg), sizeof(fd));  // Receive
```



> I also seem to recall historical issues on various OSes w.r.t. ancillary data with no normal payload data, avoided by sending at least one dummy byte too, but I cannot find any references to verify, so I might remember wrong.

Given that Mac OS X (which has a Darwin/BSD basis) requires at least one struct iovec, even if that describes a zero-length message, I'm willing to believe that the code shown above, which includes a 3-byte message, is a good step in the right general direction. The message should perhaps be a single null byte instead of 3 letters.

I've revised the code to read as shown below. It uses memmove() to copy the file descriptor to and from the cmsg buffer. It transfers a single message byte, which is a null byte.

It also has the parent process read (up to) 32 bytes of the file before passing the file descriptor to the child. The child continues reading where the parent left off. This demonstrates that the file descriptor transferred includes the file offset.

The receiver should do more validation on the cmsg before treating it as a file descriptor passing message.

```C
#include "stderr.h"
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

static
void wyslij(int socket, int fd)  // send fd by socket
{
    struct msghdr msg = { 0 };
    char buf[CMSG_SPACE(sizeof(fd))];
    memset(buf, '\0', sizeof(buf));

    /* On Mac OS X, the struct iovec is needed, even if it points to minimal data */
    struct iovec io = { .iov_base = "", .iov_len = 1 };

    msg.msg_iov = &io;
    msg.msg_iovlen = 1;
    msg.msg_control = buf;
    msg.msg_controllen = sizeof(buf);

    struct cmsghdr * cmsg = CMSG_FIRSTHDR(&msg);
    cmsg->cmsg_level = SOL_SOCKET;
    cmsg->cmsg_type = SCM_RIGHTS;
    cmsg->cmsg_len = CMSG_LEN(sizeof(fd));

    memmove(CMSG_DATA(cmsg), &fd, sizeof(fd));

    msg.msg_controllen = CMSG_SPACE(sizeof(fd));

    if (sendmsg(socket, &msg, 0) < 0)
        err_syserr("Failed to send message\n");
}

static
int odbierz(int socket)  // receive fd from socket
{
    struct msghdr msg = {0};

    /* On Mac OS X, the struct iovec is needed, even if it points to minimal data */
    char m_buffer[1];
    struct iovec io = { .iov_base = m_buffer, .iov_len = sizeof(m_buffer) };
    msg.msg_iov = &io;
    msg.msg_iovlen = 1;

    char c_buffer[256];
    msg.msg_control = c_buffer;
    msg.msg_controllen = sizeof(c_buffer);

    if (recvmsg(socket, &msg, 0) < 0)
        err_syserr("Failed to receive message\n");

    struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg);

    err_remark("About to extract fd\n");
    int fd;
    memmove(&fd, CMSG_DATA(cmsg), sizeof(fd));
    err_remark("Extracted fd %d\n", fd);

    return fd;
}

int main(int argc, char **argv)
{
    const char *filename = "./z7.c";

    err_setarg0(argv[0]);
    err_setlogopts(ERR_PID);
    if (argc > 1)
        filename = argv[1];
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, sv) != 0)
        err_syserr("Failed to create Unix-domain socket pair\n");

    int pid = fork();
    if (pid > 0)  // in parent
    {
        err_remark("Parent at work\n");
        close(sv[1]);
        int sock = sv[0];

        int fd = open(filename, O_RDONLY);
        if (fd < 0)
            err_syserr("Failed to open file %s for reading\n", filename);

        /* Read some data to demonstrate that file offset is passed */
        char buffer[32];
        int nbytes = read(fd, buffer, sizeof(buffer));
        if (nbytes > 0)
            err_remark("Parent read: [[%.*s]]\n", nbytes, buffer);

        wyslij(sock, fd);

        close(fd);
        nanosleep(&(struct timespec){ .tv_sec = 1, .tv_nsec = 500000000}, 0);
        err_remark("Parent exits\n");
    }
    else  // in child
    {
        err_remark("Child at play\n");
        close(sv[0]);
        int sock = sv[1];

        nanosleep(&(struct timespec){ .tv_sec = 0, .tv_nsec = 500000000}, 0);

        int fd = odbierz(sock);
        printf("Read %d!\n", fd);
        char buffer[256];
        ssize_t nbytes;
        while ((nbytes = read(fd, buffer, sizeof(buffer))) > 0)
            write(1, buffer, nbytes);
        printf("Done!\n");
        close(fd);
    }
    return 0;
}
```

And a sample run:

```
$ ./fd-passing
fd-passing: pid=8000: Parent at work
fd-passing: pid=8000: Parent read: [[This is the file z7.c.
It isn't ]]
fd-passing: pid=8001: Child at play
fd-passing: pid=8001: About to extract fd
fd-passing: pid=8001: Extracted fd 3
Read 3!
very interesting.
It isn't even C code.
But it is used by the fd-passing program to demonstrate that file
descriptors can indeed be passed between sockets on occasion.
And, with the fully working code, it does indeed seem to work.
Extended testing would have the parent code read part of the file, and
then demonstrate that the child codecontinues where the parent left off.
That has not been coded, though.
Done!
fd-passing: pid=8000: Parent exits
$
```

From UNIX Network Programming Volume 1, Third Edition: The Sockets Networking API:

```C
/* include read_fd */
#include	"unp.h"

ssize_t
read_fd(int fd, void *ptr, size_t nbytes, int *recvfd)
{
	struct msghdr	msg;
	struct iovec	iov[1];
	ssize_t			n;

#ifdef	HAVE_MSGHDR_MSG_CONTROL
	union {
	  struct cmsghdr	cm;
	  char				control[CMSG_SPACE(sizeof(int))];
	} control_un;
	struct cmsghdr	*cmptr;

	msg.msg_control = control_un.control;
	msg.msg_controllen = sizeof(control_un.control);
#else
	int				newfd;

	msg.msg_accrights = (caddr_t) &newfd;
	msg.msg_accrightslen = sizeof(int);
#endif

	msg.msg_name = NULL;
	msg.msg_namelen = 0;

	iov[0].iov_base = ptr;
	iov[0].iov_len = nbytes;
	msg.msg_iov = iov;
	msg.msg_iovlen = 1;

	if ( (n = recvmsg(fd, &msg, 0)) <= 0)
		return(n);

#ifdef	HAVE_MSGHDR_MSG_CONTROL
	if ( (cmptr = CMSG_FIRSTHDR(&msg)) != NULL &&
	    cmptr->cmsg_len == CMSG_LEN(sizeof(int))) {
		if (cmptr->cmsg_level != SOL_SOCKET)
			err_quit("control level != SOL_SOCKET");
		if (cmptr->cmsg_type != SCM_RIGHTS)
			err_quit("control type != SCM_RIGHTS");
		*recvfd = *((int *) CMSG_DATA(cmptr));
	} else
		*recvfd = -1;		/* descriptor was not passed */
#else
/* *INDENT-OFF* */
	if (msg.msg_accrightslen == sizeof(int))
		*recvfd = newfd;
	else
		*recvfd = -1;		/* descriptor was not passed */
/* *INDENT-ON* */
#endif

	return(n);
}
/* end read_fd */

ssize_t
Read_fd(int fd, void *ptr, size_t nbytes, int *recvfd)
{
	ssize_t		n;

	if ( (n = read_fd(fd, ptr, nbytes, recvfd)) < 0)
		err_sys("read_fd error");

	return(n);
}
```

```C
/* include write_fd */
#include	"unp.h"

ssize_t
write_fd(int fd, void *ptr, size_t nbytes, int sendfd)
{
	struct msghdr	msg;
	struct iovec	iov[1];

#ifdef	HAVE_MSGHDR_MSG_CONTROL
	union {
	  struct cmsghdr	cm;
	  char				control[CMSG_SPACE(sizeof(int))];
	} control_un;
	struct cmsghdr	*cmptr;

	msg.msg_control = control_un.control;
	msg.msg_controllen = sizeof(control_un.control);

	cmptr = CMSG_FIRSTHDR(&msg);
	cmptr->cmsg_len = CMSG_LEN(sizeof(int));
	cmptr->cmsg_level = SOL_SOCKET;
	cmptr->cmsg_type = SCM_RIGHTS;
	*((int *) CMSG_DATA(cmptr)) = sendfd;
#else
	msg.msg_accrights = (caddr_t) &sendfd;
	msg.msg_accrightslen = sizeof(int);
#endif

	msg.msg_name = NULL;
	msg.msg_namelen = 0;

	iov[0].iov_base = ptr;
	iov[0].iov_len = nbytes;
	msg.msg_iov = iov;
	msg.msg_iovlen = 1;

	return(sendmsg(fd, &msg, 0));
}
/* end write_fd */

ssize_t
Write_fd(int fd, void *ptr, size_t nbytes, int sendfd)
{
	ssize_t		n;

	if ( (n = write_fd(fd, ptr, nbytes, sendfd)) < 0)
		err_sys("write_fd error");

	return(n);
}
```


Code example are from: https://github.com/unpbook/unpv13e/tree/master/lib