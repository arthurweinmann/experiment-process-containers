# Resources


- http://man7.org/linux/man-pages/man7/cgroups.7.html

- https://www.kernel.org/doc/Documentation/cgroup-v1/cgroups.txt

# Inheritance

```
A child process created via fork(2) inherits its parent's cgroup
memberships.  A process's cgroup memberships are preserved across
execve(2).
```


# CGROUP V1

## Tasks (threads) versus processes

```
In cgroups v1, a distinction is drawn between processes and tasks.
In this view, a process can consist of multiple tasks (more commonly
called threads, from a user-space perspective, and called such in the
remainder of this man page).  In cgroups v1, it is possible to
independently manipulate the cgroup memberships of the threads in a
process.

The cgroups v1 ability to split threads across different cgroups
caused problems in some cases.  For example, it made no sense for the
memory controller, since all of the threads of a process share a
single address space.  Because of these problems, the ability to
independently manipulate the cgroup memberships of the threads in a
process was removed in the initial cgroups v2 implementation, and
subsequently restored in a more limited form (see the discussion of
"thread mode" below).
```

```
 A process may be moved to this cgroup by writing its PID into the
 cgroup's cgroup.procs file:

   echo $$ > /sys/fs/cgroup/cpu/cg1/cgroup.procs

Only one PID at a time should be written to this file.

Writing the value 0 to a cgroup.procs file causes the writing process
to be moved to the corresponding cgroup.

When writing a PID into the cgroup.procs, all threads in the process
are moved into the new cgroup at once.

Within a hierarchy, a process can be a member of exactly one cgroup.
Writing a process's PID to a cgroup.procs file automatically removes
it from the cgroup of which it was previously a member.

The cgroup.procs file can be read to obtain a list of the processes
that are members of a cgroup.  The returned list of PIDs is not guar‐
anteed to be in order.  Nor is it guaranteed to be free of dupli‐
cates.  (For example, a PID may be recycled while reading from the
list.)

"In cgroups v1, an individual thread can be moved to another cgroup by
writing its thread ID (i.e., the kernel thread ID returned by
clone(2) and gettid(2)) to the tasks file in a cgroup directory.
This file can be read to discover the set of threads that are members
of the cgroup.
```

# CGROUP V2

## Difference with cgroup v1

- The tasks file has been removed.