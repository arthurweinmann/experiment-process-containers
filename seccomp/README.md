# Todo

- code & verify toaster syscall filters
    - linux syscalls list: http://man7.org/linux/man-pages/man2/syscalls.2.html


# lookout

- https://lwn.net/Articles/822256/


# BPF compile once, run everywhere

-> merged in kernal v5.4, soon we'll be able to reuse a pre-compiled seccomp policy for all execution

See:

- http://vger.kernel.org/bpfconf2019_talks/bpf-core.pdf
- https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=ddc7c3042614
- https://kernelnewbies.org/Linux_5.4
- https://lwn.net/Articles/773198/
- https://lwn.net/Articles/797595/
- http://vger.kernel.org/lpc_bpf2018_talks/bpf_compile_once.pdf

For now, we may use a combination of fork + setns so that seccomp policy are inherited without recompilation: 

See http://man7.org/linux/man-pages/man2/prctl.2.html:

PR_SET_SECCOMP (since Linux 2.6.23)
Set the secure computing (seccomp) mode for the calling
thread, to limit the available system calls.  The more recent
seccomp(2) system call provides a superset of the functional‐
ity of PR_SET_SECCOMP.

The seccomp mode is selected via arg2.  (The seccomp constants
are defined in <linux/seccomp.h>.)

With arg2 set to SECCOMP_MODE_STRICT, the only system calls
that the thread is permitted to make are read(2), write(2),
_exit(2) (but not exit_group(2)), and sigreturn(2).  Other
system calls result in the delivery of a SIGKILL signal.
Strict secure computing mode is useful for number-crunching
applications that may need to execute untrusted byte code,
perhaps obtained by reading from a pipe or socket.  This oper‐
ation is available only if the kernel is configured with CON‐
FIG_SECCOMP enabled.

With arg2 set to SECCOMP_MODE_FILTER (since Linux 3.5), the
system calls allowed are defined by a pointer to a Berkeley
Packet Filter passed in arg3.  This argument is a pointer to
struct sock_fprog; it can be designed to filter arbitrary sys‐
tem calls and system call arguments.  This mode is available
only if the kernel is configured with CONFIG_SECCOMP_FILTER
enabled.

If SECCOMP_MODE_FILTER filters permit fork(2), then the sec‐
comp mode is inherited by children created by fork(2); if
execve(2) is permitted, then the seccomp mode is preserved
across execve(2).  If the filters permit prctl() calls, then
additional filters can be added; they are run in order until
the first non-allow result is seen.

For further information, see the kernel source file Documenta‐
tion/userspace-api/seccomp_filter.rst (or Documenta‐
tion/prctl/seccomp_filter.txt before Linux 4.13).

# Why not the new syscall seccomp instead of prctl

- http://man7.org/linux/man-pages/man2/seccomp.2.html

-> apparemment, avec le nouveau, on doit faire une op a la fois, vérifier

# Links

- https://www.kernel.org/doc/html/latest/userspace-api/seccomp_filter.html#adding-architecture-support
- https://www.netronome.com/blog/bpf-ebpf-xdp-and-bpfilter-what-are-these-things-and-what-do-they-mean-enterprise/

# Notes

## To check or not to check clone syscall flags

- not to check, since CLONE_NEWNS flags requires the CAP_SYS_ADMIN capability. See https://linux.die.net/man/2/unshare

- many syscalls should not work if all capabilities are dropped, so investigate that

- it seems the more syscalls you forbide, the more time it takes to compile the seccomp filter. It seems "allow all except" compiles faster than "disallow all except".