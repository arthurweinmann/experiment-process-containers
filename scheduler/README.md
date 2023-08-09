# What's this ?

- It is toastate container manager

# PIDFD

- Pidfd support in clone syscall begins with linux kernel 5.2 and ubuntu 18.04 and ubuntu 19.04 only have kernel version 5.0. So we'll have to wait to use it.

# TODO

- Instead of letting run_child read on child fd until it either close it or sends an error message back, put the fd in epoll.