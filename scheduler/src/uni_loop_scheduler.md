# Uni Loop Scheduler

## TODO

- Use the same UTS namespace between all toastainers (maybe symlink to the mount namespace file in a place known by all toastainer or better mount it in a place known by all, mounting it will keep it open as does keeping a fd from it)

- Investigate this small Rust async runtime for the scheduler, only 1500 lines of code, using epoll, so should be easy to understand and adapt the code if need be: https://github.com/stjepang/smol 

## Maximum number of linux namespaces

```
The files in the /proc/sys/user directory (which is present since
Linux 4.9) expose limits on the number of namespaces of various types
that can be created.
```

See http://man7.org/linux/man-pages/man7/namespaces.7.html and https://lwn.net/Articles/694968/