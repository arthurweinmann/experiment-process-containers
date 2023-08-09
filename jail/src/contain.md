# PRCTL

## prctl(PR_SET_PDEATHSIG, SIGKILL, 0, 0, 0)

- check implication with our system of epoll future with pidfd

> http://man7.org/linux/man-pages/man2/prctl.2.html

*PR_SET_PDEATHSIG (since Linux 2.1.57)*

Set the parent-death signal of the calling process to arg2
(either a signal value in the range 1..maxsig, or 0 to clear).
This is the signal that the calling process will get when its
parent dies.

Warning: the "parent" in this case is considered to be the
thread that created this process.  In other words, the signal
will be sent when that thread terminates (via, for example,
pthread_exit(3)), rather than after all of the threads in the
parent process terminate.

The parent-death signal is sent upon subsequent termination of
the parent thread and also upon termination of each subreaper
process (see the description of PR_SET_CHILD_SUBREAPER above)
to which the caller is subsequently reparented.  If the parent
thread and all ancestor subreapers have already terminated by
the time of the PR_SET_PDEATHSIG operation, then no parent-
death signal is sent to the caller.

The parent-death signal is process-directed (see signal(7))
and, if the child installs a handler using the sigaction(2)
SA_SIGINFO flag, the si_pid field of the siginfo_t argument of
the handler contains the PID of the terminating parent
process.

The parent-death signal setting is cleared for the child of a
fork(2).  It is also (since Linux 2.4.36 / 2.6.23) cleared
when executing a set-user-ID or set-group-ID binary, or a
binary that has associated capabilities (see capabilities(7));
otherwise, this value is preserved across execve(2).

# setpriority(PRIO_PROCESS, 0, nsjconf->nice_level)

- https://linux.die.net/man/3/setpriority

It sets the nice value of a process.

Nice value is a user-space and priority PR is the process's actual priority that use by Linux kernel. In linux system priorities are 0 to 139 in which 0 to 99 for real time and 100 to 139 for users. nice value range is -20 to +19 where -20 is highest, 0 default and +19 is lowest. relation between nice value and priority is :

```
PR = 20 + NI
```

PR -- Priority The scheduling priority of the task. If you see 'rt' in this field, it means the task is running under 'real time' scheduling priority.

NI -- Nice Value The nice value of the task. A negative nice value means higher priority, whereas a positive nice value means lower priority.Zero in this field simply means priority will not be adjusted in determining a task's dispatch-ability

See https://www.nixtutor.com/linux/changing-priority-on-linux-processes/

# setsid

- https://linux.die.net/man/3/setsid

Create a new session with the calling process as its leader. The process group IDs of the session and the calling process are set to the process ID of the calling process, which is returned.

The setsid() function shall create a new session, if the calling process is not a process group leader. Upon return the calling process shall be the session leader of this new session, shall be the process group leader of a new process group, and shall have no controlling terminal. The process group ID of the calling process shall be set equal to the process ID of the calling process. The calling process shall be the only process in the new process group and the only process in the new session.

# /proc/self/fd

See http://man7.org/linux/man-pages/man5/proc.5.html

Programs that take a filename as a command-line argument, but
don't take input from standard input if no argument is sup‐
plied, and programs that write to a file named as a command-
line argument, but don't send their output to standard output
if no argument is supplied, can nevertheless be made to use
standard input or standard output by using /proc/[pid]/fd
files as command-line arguments.  For example, assuming that
-i is the flag designating an input file and -o is the flag
designating an output file:

    $ foobar -i /proc/self/fd/0 -o /proc/self/fd/1 ...

and you have a working filter.

/proc/self/fd/N is approximately the same as /dev/fd/N in some
UNIX and UNIX-like systems.  Most Linux MAKEDEV scripts sym‐
bolically link /dev/fd to /proc/self/fd, in fact.

Most systems provide symbolic links /dev/stdin, /dev/stdout,
and /dev/stderr, which respectively link to the files 0, 1,
and 2 in /proc/self/fd.  Thus the example command above could
be written as:

    $ foobar -i /dev/stdin -o /dev/stdout ...