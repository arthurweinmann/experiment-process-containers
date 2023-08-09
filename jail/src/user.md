# uid/gid mapping

if it is not a new custom uid/gid mapping, we write directly to file /proc/{}/gid_map. If it is a custom new one, we use /usr/bin/newgidmap instead.

See:

- http://man7.org/linux/man-pages/man1/newgidmap.1.html
- http://man7.org/linux/man-pages/man5/subgid.5.html

- http://man7.org/linux/man-pages/man7/user_namespaces.7.html

# set_group_deny

## prctl

- http://man7.org/linux/man-pages/man2/prctl.2.html

## /proc/PID/setgroups (since Linux 3.19)

See http://man7.org/linux/man-pages/man7/user_namespaces.7.html

```
/* Linux 3.19 made a change in the handling of setgroups(2) and the
'gid_map' file to address a security issue. The issue allowed
*unprivileged* users to employ user namespaces in order to drop
The upshot of the 3.19 changes is that in order to update the
'gid_maps' file, use of the setgroups() system call in this
user namespace must first be disabled by writing "deny" to one of
the /proc/PID/setgroups files for this namespace.  That is the
purpose of the following function. */
```

# Useful links

An explanation of user namespaces: http://man7.org/conf/meetup/understanding-user-namespaces--Google-Munich-Kerrisk-2019-10-25.pdf