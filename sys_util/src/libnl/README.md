
# Bindings to libnl for interacting with netlink

The libnl suite is a collection of C libraries providing APIs to netlink protocol based Linux kernel interfaces.

We could implement our own library in rust with libc, so without C bindings, maybe one day we should do or at least investigate what's happening below. For now, there is no time. 

Current coverage of netlink with these RUST native libraries using libc is not complete. LIBNL in C remains the libraries with the more coverage.

See:

- Rust building block for the netlink protocol: https://github.com/little-dude/netlink

- https://docs.rs/netlink-packet/0.1.1/netlink_packet/

- https://github.com/crhino/netlink-rs/tree/master/netlink-rs

- pnetlink - native NetLink library for rust using libpnet: https://github.com/polachok/pnetlink ; https://github.com/libpnet/libpnet

- Golang netlink library: https://github.com/vishvananda/netlink

- Python netlink library: https://github.com/svinota/pyroute2/tree/master/pyroute2/netlink

- http://man7.org/linux/man-pages/man7/netlink.7.html

## Checkout firecracker VIRTIO and use of tap interface (from tun/tap)

- https://unixism.net/2019/10/how-aws-firecracker-works-a-deep-dive/

- https://unix.stackexchange.com/questions/293434/what-is-difference-between-tap-interface-and-normal-interface/293470

- https://github.com/jonhoo/rust-tcp

see virtio-fs coming soon in Linux: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=2d1d25d0a224

## LIBNL Doc

libnl-route: https://www.infradead.org/~tgr/libnl/doc/route.html
libnl-core: https://www.infradead.org/~tgr/libnl/doc/core.html

### Linux vhost

- http://events17.linuxfoundation.org/sites/events/files/slides/vhost_sharing_v6.pdf
- https://www.redhat.com/en/blog/introduction-virtio-networking-and-vhost-net (redhat)
- https://lwn.net/Articles/346267/

## Install libnl

If command ```pkg-config --exists libnl-route-3.0 && echo yes``` does not echo "yes", you probably miss libnl3(-dev)/libnl-route-3(-dev) libraries

On ubuntu 18.04, install with:

```
sudo apt-get install libnl-3-dev
sudo apt-get install libnl-route-3-dev
```

> To check which version is currently installed, you can use `cat /usr/include/libnl3/netlink/version.h`.

It should put the lib (at least its headers) in /usr/include/libnl3.

## VS code properties to have autocomplete in C++

```
{
    "configurations": [
        {
            "name": "Linux",
            "includePath": [
                "${workspaceFolder}/**",
                "/usr/include/libnl3"
            ],
            "defines": [],
            "compilerPath": "/usr/bin/gcc",
            "cStandard": "c11",
            "cppStandard": "c++17",
            "intelliSenseMode": "clang-x64"
        }
    ],
    "version": 4
}
```

See http://www.infradead.org/~tgr/libnl/ for more information

# Is memory well freed by libnl ?

it seems so, for example functions nl_socket_free and nl_cache_free call the C function "free" on the passed pointer. See http://ecomputernotes.com/data-structures/basic-of-data-structure/free-function.
But continue to investigate and we need to test for memory leak. For example, launch toasters all day long and monitor RAM.

# TODO

> Mapp libnl (as we did for nsjail), to see what's going on under the hood and be able to mix what nsjail does with firecracker virtio-net and a tun/tap interface to control toaster internet traffic