
# Difference between clone and unshare

unshare does not put the current process in a new pid namespace

- https://www.toptal.com/linux/separation-anxiety-isolating-your-system-with-linux-namespaces:

```
To create a new PID namespace, one must call the clone() system call with a special flag CLONE_NEWPID. (C provides a wrapper to expose this system call, and so do many other popular languages.) Whereas the other namespaces discussed below can also be created using the unshare() system call, a PID namespace can only be created at the time a new process is spawned using clone()

[...]

Linux provides unshare(). This special system call allows a process to isolate itself from the original namespace, instead of having the parent isolate the child in the first place.
```

- here (https://lwn.net/Articles/532748/) it is said that:

```
Specifying the CLONE_NEWPID flag in a call to unshare() creates a new PID namespace, but does not place the caller in the new namespace. Rather, any children created by the caller will be placed in the new namespace; the first such child will become the init process for the namespace.

[...]

It's worth emphasizing that setns() and unshare() treat PID namespaces specially. For other types of namespaces, these system calls do change the namespace of the caller. The reason that these system calls do not change the PID namespace of the calling process is because becoming a member of another PID namespace would cause the process's idea of its own PID to change, since getpid() reports the process's PID with respect to the PID namespace in which the process resides. Many user-space programs and libraries rely on the assumption that a process's PID (as reported by getpid()) is constant (in fact, the GNU C library getpid() wrapper function caches the PID); those programs would break if a process's PID changed. To put things another way: a process's PID namespace membership is determined when the process is created, and (unlike other types of namespace membership) cannot be changed thereafter.
```

- https://lwn.net/Articles/531381/:

```
The unshare() system call provides functionality similar to clone(), but operates on the calling process: it creates the new namespaces specified by the CLONE_NEW* bits in its flags argument and makes the caller a member of the namespaces. (As with clone(), unshare() provides functionality beyond working with namespaces that we'll ignore here.) The main purpose of unshare() is to isolate namespace (and other) side effects without having to create a new process or thread (as is done by clone()).
```

- https://lwn.net/Articles/531114/:

```
PID namespaces (CLONE_NEWPID, Linux 2.6.24) isolate the process ID number space. In other words, processes in different PID namespaces can have the same PID. One of the main benefits of PID namespaces is that containers can be migrated between hosts while keeping the same process IDs for the processes inside the container. PID namespaces also allow each container to have its own init (PID 1), the "ancestor of all processes" that manages various system initialization tasks and reaps orphaned child processes when they terminate.
```

# More on pid namespaces to know if necessary

- if not in a new pid namespace, a jail process may see extern processes information

- One use of PID namespaces is to implement a package of processes (a container) that behaves like a self-contained Linux system

- https://lwn.net/Articles/531419/:

```
Each process on a Linux system has a /proc/PID directory that contains pseudo-files describing the process. This scheme translates directly into the PID namespaces model. Within a PID namespace, the /proc/PID directories show information only about processes within that PID namespace or one of its descendant namespaces.

However, in order to make the /proc/PID directories that correspond to a PID namespace visible, the proc filesystem ("procfs" for short) needs to be mounted from within that PID namespace. From a shell running inside the PID namespace (perhaps invoked via the system() library function), we can do this using a mount command of the following form:

    # mount -t proc proc /mount_point
Alternatively, a procfs can be mounted using the mount() system call, as is done inside our program's childFunc() function:

    mkdir(mount_point, 0555);       /* Create directory for mount point */
    mount("proc", mount_point, "proc", 0, NULL);
    printf("Mounting procfs at %s\n", mount_point);
```