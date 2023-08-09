# /proc and PID namespaces

A /proc filesystem shows (in the /proc/[pid] directories) only
processes visible in the PID namespace of the process that performed
the mount, even if the /proc filesystem is viewed from processes in
other namespaces.

After creating a new PID namespace, it is useful for the child to
change its root directory and mount a new procfs instance at /proc so
that tools such as ps(1) work correctly.  If a new mount namespace is
simultaneously created by including CLONE_NEWNS in the flags argument
of clone(2) or unshare(2), then it isn't necessary to change the root
directory: a new procfs instance can be mounted directly over /proc.

From a shell, the command to mount /proc is:

    $ mount -t proc proc /proc

Calling readlink(2) on the path /proc/self yields the process ID of
the caller in the PID namespace of the procfs mount (i.e., the PID
namespace of the process that mounted the procfs).  This can be use‐
ful for introspection purposes, when a process wants to discover its
PID in other namespaces.

# Toaster timeout

Since terminating the init process of a new pid namespace also terminates all processes in that children, this allow to timeout a toaster and be sure to also terminate all children/threads he could have created.

# You must maintain the (dummy or not) init process open

The first process created in a new namespace (i.e., the process
created using clone(2) with the CLONE_NEWPID flag, or the first child
created by a process after a call to unshare(2) using the
CLONE_NEWPID flag) has the PID 1, and is the "init" process for the
namespace (see init(1)).  This process becomes the parent of any
child processes that are orphaned because a process that resides in
this PID namespace terminated (see below for further details).

If the "init" process of a PID namespace terminates, the kernel
terminates all of the processes in the namespace via a SIGKILL
signal.  This behavior reflects the fact that the "init" process is
essential for the correct operation of a PID namespace.  In this
case, a subsequent fork(2) into this PID namespace fail with the
error ENOMEM; it is not possible to create a new process in a PID
namespace whose "init" process has terminated.  Such scenarios can
occur when, for example, a process uses an open file descriptor for a
/proc/[pid]/ns/pid file corresponding to a process that was in a
namespace to setns(2) into that namespace after the "init" process
has terminated.  Another possible scenario can occur after a call to
unshare(2): if the first child subsequently created by a fork(2)
terminates, then subsequent calls to fork(2) fail with ENOMEM.

Only signals for which the "init" process has established a signal
handler can be sent to the "init" process by other members of the PID
namespace.  This restriction applies even to privileged processes,
and prevents other members of the PID namespace from accidentally
killing the "init" process.

Likewise, a process in an ancestor namespace can—subject to the usual
permission checks described in kill(2)—send signals to the "init"
process of a child PID namespace only if the "init" process has
established a handler for that signal.  (Within the handler, the
siginfo_t si_pid field described in sigaction(2) will be zero.)
SIGKILL or SIGSTOP are treated exceptionally: these signals are
forcibly delivered when sent from an ancestor PID namespace.  Neither
of these signals can be caught by the "init" process, and so will
result in the usual actions associated with those signals
(respectively, terminating and stopping the process).

Starting with Linux 3.4, the reboot(2) system call causes a signal to
be sent to the namespace "init" process.  See reboot(2) for more
details.

# Sig handlers

Likewise, a process in an ancestor namespace can—subject to the usual
permission checks described in kill(2)—send signals to the "init"
process of a child PID namespace only if the "init" process has
established a handler for that signal.  (Within the handler, the
siginfo_t si_pid field described in sigaction(2) will be zero.)
SIGKILL or SIGSTOP are treated exceptionally: these signals are
forcibly delivered when sent from an ancestor PID namespace.  Neither
of these signals can be caught by the "init" process, and so will
result in the usual actions associated with those signals
(respectively, terminating and stopping the process).