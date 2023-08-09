# to revisit and make nsjail faster

- checkout unshare() and setns() syscall with clone(), maybe we can join an existing and already set namespace and use a CoW (butterfs ?) file system so that each time a same toaster is executed, it just need to join the existing namespace but gets its own view of the filesystem, check out how many of the mount operations nsjail makes we may spare

-> see this serie of articles: https://lwn.net/Articles/531114/

## Maintain a namespace open when there are no processes in it

- https://lwn.net/Articles/531381/:

```
The /proc/PID/ns symbolic links also serve other purposes. If we open one of these files, then the namespace will continue to exist as long as the file descriptor remains open, even if all processes in the namespace terminate. The same effect can also be obtained by bind mounting one of the symbolic links to another location in the file system:

    # touch ~/uts                            # Create mount point
    # mount --bind /proc/27514/ns/uts ~/uts

[...]

Keeping a namespace open when it contains no processes is of course only useful if we intend to later add processes to it. That is the task of the setns() system call, which allows the calling process to join an existing namespace:

    int setns(int fd, int nstype);
More precisely, setns() disassociates the calling process from one instance of a particular namespace type and reassociates the process with another instance of the same namespace type.

[...]

As with unshare(), setns() does not move the caller to the PID namespace; instead, children that are subsequently created by the caller will be placed in the namespace.

[...]

It's worth emphasizing that setns() and unshare() treat PID namespaces specially. For other types of namespaces, these system calls do change the namespace of the caller. The reason that these system calls do not change the PID namespace of the calling process is because becoming a member of another PID namespace would cause the process's idea of its own PID to change
```