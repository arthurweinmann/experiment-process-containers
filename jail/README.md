***See src/\*.md for more documentation***

# multithreaded containers

- async/.await of course

- we can build a RUST future executor (see https://rust-lang.github.io/async-book/02_execution/04_executor.html) by using linux epoll on linux pidfd (https://lwn.net/Articles/794707/) which can be created by providing the flag CLONE_PIDFD to the clone syscall (see http://man7.org/linux/man-pages/man2/clone.2.html)

# toaster communication

- Set toaster exe child process stdin to unix socket fd, FIFO (named pipe) fd, etc


# Network manipulations for toaster execution

- see src/net.md


# Security guideline

> Almost every app has bugs, but one big challenge of security engineering is to make bugs unexploitable without knowing where they are

Assume they are bugs and vulnerabilities in places you do not know about, so make sure they are not exploitable. 

One way to do this could be to encapsulate all jail mechanism with kvm (qemu ?). Check out firecracker for inspiration