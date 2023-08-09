# DISCLAIMER: THIS IS AN OLD EXPERIMENT MADE PUBLIC

This repository contains an old experiment of mine, so it should not be used as is and probably won't work.
Nevertheless, it may give you some useful information. 

I used it as a way to learn Rust too so it is not very pretty and contains a lot of notes about Rust behaviours and syntax.
Also the version of Rust used is quite old now.

With all that said, enjoy!

It is mainly a port in Rust of the excellent project: https://github.com/google/nsjail

# Usage

## Interactively build rootfs image

- Compile toastainer
- then run the binary with command line argument --create_image={absolute path to rootfs directory}, like:

```bash
./rust/toastainer/target/debug/toastainer --create_image=/home/arthurbuntu/alpine
```

- If you want to use a shell script, do:

```bash
./rust/toastainer/target/debug/toastainer --create_image=/home/arthurbuntu/alpine --use_script={absolute path to sh script}

# like

./rust/toastainer/target/debug/toastainer --create_image=/home/arthurbuntu/alpine --use_script=/home/arthurbuntu/rust/toastainer/rootfs/src/test.sh
```

# Installation

- steps for ubuntu 18.04. Todo: see how to make it work on other distrib, like debian for example.

## libnl

If command ```pkg-config --exists libnl-route-3.0 && echo yes``` does not echo "yes", you probably miss libnl3(-dev)/libnl-route-3(-dev) libraries

On ubuntu 18.04, install with:

```
sudo apt-get install libnl-3-dev
sudo apt-get install libnl-route-3-dev
```

> It should put the lib (at least its headers) in /usr/include/libnl3.

> To check which version is currently installed, you can use `cat /usr/include/libnl3/netlink/version.h`.

## libcap

Used in caps package

`sudo apt-get install libcap-dev`

## newuidmap and newgidmap

if `which newuidmap` and/or `which newgidmap` prints nothing, then you need to install them with `sudo apt install uidmap`.
They should go in /usr/bin/newuidmap and /usr/bin/newgidmap

## Network Setup

```bash
ip link add dev tveth0 type veth peer name tveth1
ip link set dev tveth0 up
ip link set dev tveth1 up
ip addr add 10.166.0.1/16 broadcast 10.166.255.255 dev tveth0 # 10.0.0.0/8 is by convention a block of private IP addresses, See http://www.faqs.org/rfcs/rfc1918.html

# On your local PC only
iptables -t nat -A POSTROUTING -s 10.166.0.0/16 -j MASQUERADE

# on TVS only,  do not do this on your PC, it is to prevent Toaster from calling private toaster servers in aws VPC
# iptables -t nat -A POSTROUTING -s 10.166.0.0/16 ! -d 172.16.0.0/12 -j MASQUERADE

echo 'net.ipv4.ip_forward=1' >> /etc/sysctl.conf
sysctl -p /etc/sysctl.conf

# it is tveth1 that will be moved into the child NET namespace

# without one of these line, toaster/container/child in new net namespace, won't be able to resolve hostnames to ip addresses
# echo "nameserver 8.8.8.8" > /chroot/binary/etc/resolv.conf # with a rootfs
# echo "nameserver 8.8.8.8" > /etc/resolv.conf # with native root (no pivot root done)
```

***See jail/src/net.md for more information***

# Inspiration

## NSJail (C++)

## Firecracker (rust)

## Moby by Docker (golang)

- https://github.com/moby/moby

# Notes

Things still to learn and apply to this package:

- rust async/.await: zero-cost pollable async computation (futures): https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html
- rust PIN: a type that pins data to its location in memory, useful for example for self-referential structs: https://doc.rust-lang.org/std/pin/
- rust std::ptr::NonNull: https://doc.rust-lang.org/std/ptr/struct.NonNull.html#method.dangling
- rust marker: Primitive traits and types representing basic properties of types. For example, PhantomData (Zero-sized type used to mark things that "act like" they own a T) or PhantomPinned	(A marker type which does not implement Unpin): https://doc.rust-lang.org/nightly/std/marker/index.html
- std::cell: https://doc.rust-lang.org/std/cell/index.html and especially UnsafeCell: https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html. Can be used for example when Rust compiler complains about a mutex and you know for example that you only have one thread and actually do not need it.

## For a TCP server with linux epoll and rust async/.await

- https://github.com/Hexilee/async-io-demo: demo for rust asynchronous io: from mio to stackless coroutine

- check out https://github.com/tokio-rs/tokio if it really uses stackless coroutine with epoll and rust futures

- https://github.com/murphysean/betarustasyncawait

# GPU support

- https://lwn.net/Articles/788277/
- https://marmelab.com/blog/2018/03/21/using-nvidia-gpu-within-docker-container.html
- https://github.com/google/cadvisor/blob/master/docs/running.md#hardware-accelerator-monitoring
- https://github.com/firecracker-microvm/firecracker/issues/849
- https://www.phoronix.com/scan.php?page=news_item&px=Linux-Cgroups-GPUs-2019
- https://discuss.linuxcontainers.org/t/gpu-resources-monitoring-for-lxd-containers/5365/2
- https://stgraber.org/2017/03/21/cuda-in-lxd/
- https://marmelab.com/blog/2018/03/21/using-nvidia-gpu-within-docker-container.html

# Read List

- https://utcc.utoronto.ca/~cks/space/blog/unix/ChownDivideAndQuotas
- https://utcc.utoronto.ca/~cks/space/blog/sysadmin/ChownSymlinkSafetyII
- https://utcc.utoronto.ca/~cks/space/blog/

- Nice mini container summary: https://zserge.com/posts/containers/

# Tricks

## Bash

### Print process namespaces

```bash
ls -l /proc/$$/ns | awk '{print $1, $9, $10, $11}'
```