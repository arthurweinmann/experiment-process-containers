# Net Setup

```bash
ip link add dev tveth0 type veth peer name tveth1
ip link set dev tveth0 up
ip link set dev tveth1 up
ip addr add 10.166.0.1/16 broadcast 10.166.255.255 dev tveth0 # 10.0.0.0/8 is by convention a block of private IP addresses, See http://www.faqs.org/rfcs/rfc1918.html

# On your local PC only
iptables -t nat -A POSTROUTING -s 10.166.0.0/16 -j MASQUERADE

# on TVS only,  do not do this on your PC, it is to prevent Toaster from calling private servers in aws VPC
# iptables -t nat -A POSTROUTING -s 10.166.0.0/16 ! -d 172.16.0.0/12 -j MASQUERADE
# 169.254.169.254 is the aws ip to call to get ec2 metadata info, block it from inside toasters
# iptables -t nat -A PREROUTING -s 10.166.0.0/16 -d 169.254.169.254/32 -j BLACKHOLE
# iptables -t nat -A BLACKHOLE -j DNAT --to-destination 0.0.0.1

# save iptables on tvs
# iptables-save > /home/ubuntu/iptables.v4


echo 'net.ipv4.ip_forward=1' >> /etc/sysctl.conf
sysctl -p /etc/sysctl.conf

# it is tveth1 that will be moved into the child NET namespace

# without one of these line, toaster/container/child in new net namespace, won't be able to resolve hostnames to ip addresses
# echo "nameserver 8.8.8.8" > /chroot/binary/etc/resolv.conf # with a rootfs
# echo "nameserver 8.8.8.8" > /etc/resolv.conf # with native root (no pivot root done)
```

## ip link add <p1-name> type veth peer name <p2-name>

- it is rtnetlink used under the hood

- In the above, `p1-name` and `p2-name` are the names assigned to the two connected end points. Packets transmitted on one device in the pair are immediately received on the other device.  When either devices is down the link state of the pair is down.

- veth device pairs are useful for combining the network facilities of the kernel together in interesting ways.  A particularly interesting use case is to place one end of a veth pair in one network namespace and the other end in another network namespace, thus allowing commu‐ nication between network namespaces.  To do this, one first creates the veth device as above and then moves one side of the pair to the other namespace:

```bash
ip link set <p2-name> netns <p2-namespace>
```

- ethtool(8) can be used to find the peer of a veth network interface, using commands something like:

```bash
    ip link add ve_A type veth peer name ve_B   # Create veth pair
    ethtool -S ve_A         # Discover interface index of peer
        NIC statistics:
        peer_ifindex: 16
    ip link | grep '^16:'   # Look up interface
           16: ve_B@ve_A: <BROADCAST,MULTICAST,M-DOWN> mtu 1500 qdisc ...
```

### ip link add dev tveth0 type veth peer name tveth1

Before executing this command:

```bash
ip addr
    1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN group default qlen 1000
        link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
        inet 127.0.0.1/8 scope host lo
        valid_lft forever preferred_lft forever
        inet6 ::1/128 scope host 
        valid_lft forever preferred_lft forever
    2: wlp59s0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc mq state UP group default qlen 1000
        link/ether 7c:5c:f8:30:00:b1 brd ff:ff:ff:ff:ff:ff
        inet 192.168.1.15/24 brd 192.168.1.255 scope global dynamic noprefixroute wlp59s0
        valid_lft 83879sec preferred_lft 83879sec
        inet6 fe80::b6de:fe7:de36:b8f6/64 scope link noprefixroute 
        valid_lft forever preferred_lft forever
```

After executing it, we have our paired network interfaces:

```bash
ip addr 
    1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN group default qlen 1000
        link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
        inet 127.0.0.1/8 scope host lo
        valid_lft forever preferred_lft forever
        inet6 ::1/128 scope host 
        valid_lft forever preferred_lft forever
    2: wlp59s0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc mq state UP group default qlen 1000
        link/ether 7c:5c:f8:30:00:b1 brd ff:ff:ff:ff:ff:ff
        inet 192.168.1.15/24 brd 192.168.1.255 scope global dynamic noprefixroute wlp59s0
        valid_lft 83834sec preferred_lft 83834sec
        inet6 fe80::b6de:fe7:de36:b8f6/64 scope link noprefixroute 
        valid_lft forever preferred_lft forever
    3: tveth1@tveth0: <BROADCAST,MULTICAST,M-DOWN> mtu 1500 qdisc noop state DOWN group default qlen 1000
        link/ether a6:36:1e:58:c4:05 brd ff:ff:ff:ff:ff:ff
    4: tveth0@tveth1: <BROADCAST,MULTICAST,M-DOWN> mtu 1500 qdisc noop state DOWN group default qlen 1000
        link/ether 5e:e9:3a:f0:e1:65 brd ff:ff:ff:ff:ff:ff

ip link
    1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000
        link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
    2: wlp59s0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc mq state UP mode DORMANT group default qlen 1000
        link/ether 7c:5c:f8:30:00:b1 brd ff:ff:ff:ff:ff:ff
    3: tveth1@tveth0: <BROADCAST,MULTICAST,M-DOWN> mtu 1500 qdisc noop state DOWN mode DEFAULT group default qlen 1000
        link/ether a6:36:1e:58:c4:05 brd ff:ff:ff:ff:ff:ff
    4: tveth0@tveth1: <BROADCAST,MULTICAST,M-DOWN> mtu 1500 qdisc noop state DOWN mode DEFAULT group default qlen 1000
        link/ether 5e:e9:3a:f0:e1:65 brd ff:ff:ff:ff:ff:ff
```

after executing set up commands:

```bash
ip link
    1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000
        link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
    2: wlp59s0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc mq state UP mode DORMANT group default qlen 1000
        link/ether 7c:5c:f8:30:00:b1 brd ff:ff:ff:ff:ff:ff
    3: tveth1@tveth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc noqueue state UP mode DEFAULT group default qlen 1000
        link/ether a6:36:1e:58:c4:05 brd ff:ff:ff:ff:ff:ff
    4: tveth0@tveth1: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc noqueue state UP mode DEFAULT group default qlen 1000
        link/ether 5e:e9:3a:f0:e1:65 brd ff:ff:ff:ff:ff:ff
```

## ip addr add 10.166.0.1/16 broadcast 10.166.255.255 dev tveth0

```bash
root@arthurbuntu:~# ip addr add 10.166.0.1/16 broadcast 10.166.255.255 dev tveth0
root@arthurbuntu:~# ip addr
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
    inet 127.0.0.1/8 scope host lo
       valid_lft forever preferred_lft forever
    inet6 ::1/128 scope host 
       valid_lft forever preferred_lft forever
2: wlp59s0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc mq state UP group default qlen 1000
    link/ether 7c:5c:f8:30:00:b1 brd ff:ff:ff:ff:ff:ff
    inet 192.168.1.15/24 brd 192.168.1.255 scope global dynamic noprefixroute wlp59s0
       valid_lft 83503sec preferred_lft 83503sec
    inet6 fe80::b6de:fe7:de36:b8f6/64 scope link noprefixroute 
       valid_lft forever preferred_lft forever
3: tveth1@tveth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc noqueue state UP group default qlen 1000
    link/ether a6:36:1e:58:c4:05 brd ff:ff:ff:ff:ff:ff
    inet6 fe80::a436:1eff:fe58:c405/64 scope link 
       valid_lft forever preferred_lft forever
4: tveth0@tveth1: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc noqueue state UP group default qlen 1000
    link/ether 5e:e9:3a:f0:e1:65 brd ff:ff:ff:ff:ff:ff
    inet 10.166.0.1/16 brd 10.166.255.255 scope global tveth0 # here
       valid_lft forever preferred_lft forever
    inet6 fe80::5ce9:3aff:fef0:e165/64 scope link 
       valid_lft forever preferred_lft forever
```

## iptables -t nat -A POSTROUTING -s 10.166.0.0/16 ! -d 172.16.0.0/12 -j MASQUERADE

On a local PC for test, we do not restrict 172.16.0.0/12 ip addresses, so we ommit "! -d 172.16.0.0/12".

```bash
root@arthurbuntu:~# iptables -t nat -A POSTROUTING -s 10.166.0.0/16 -j MASQUERADE
root@arthurbuntu:~# iptables -t nat -L
Chain INPUT (policy ACCEPT)
target     prot opt source               destination         

Chain FORWARD (policy ACCEPT)
target     prot opt source               destination         

Chain OUTPUT (policy ACCEPT)
target     prot opt source               destination         
root@arthurbuntu:~# iptables -t nat -L
Chain PREROUTING (policy ACCEPT)
target     prot opt source               destination         

Chain INPUT (policy ACCEPT)
target     prot opt source               destination         

Chain OUTPUT (policy ACCEPT)
target     prot opt source               destination         

Chain POSTROUTING (policy ACCEPT)
target     prot opt source               destination         
MASQUERADE  all  --  10.166.0.0/16        anywhere
```

- veth: http://man7.org/linux/man-pages/man4/veth.4.html

### -t, --table

```
nat: 
This table is consulted when a packet that creates a new connection is encountered. It consists of three built-ins: PREROUTING (for altering packets as soon as they come in), OUTPUT (for altering locally-generated packets before routing), and POSTROUTING (for altering packets as they are about to go out). 
```

- NAT (network address translation) is the modification of the addresses and/or ports of network packets as they pass through a computer. 

- NAT requires connection tracking, and connection tracking only works when the computer sees all the packets. So, take care not to break connection tracking with a firewall

- For static IP, you would use SNAT (source NAT) instead of masquerade. The masquerade is intended for situations where the gateway has a dynamic IP address.

### -s, --source

```
Source specification. Address can be either a network name, a hostname (please note that specifying any name to be resolved with a remote query such as DNS is a really bad idea), a network IP address (with /mask), or a plain IP address. The mask can be either a network mask or a plain number, specifying the number of 1's at the left side of the network mask. Thus, a mask of 24 is equivalent to 255.255.255.0. A "!" argument before the address specification inverts the sense of the address. 
```

### -d, --destination

```
Destination specification. See the description of the -s (source) flag for a detailed description of the syntax. A "!" argument before the address specification inverts the sense of the address.
```

### -j, -jump

```
This specifies the target of the rule; i.e., what to do if the packet matches it. The target can be a user-defined chain (other than the one this rule is in), one of the special builtin targets which decide the fate of the packet immediately, or an extension (see EXTENSIONS below). If this option is omitted in a rule (and -g is not used), then matching the rule will have no effect on the packet's fate, but the counters on the rule will be incremented. 
```

### MASQUERADE

```
This target is only valid in the nat table, in the POSTROUTING chain. It should only be used with dynamically assigned IP (dialup) connections: if you have a static IP address, you should use the SNAT target. Masquerading is equivalent to specifying a mapping to the IP address of the interface the packet is going out, but also has the effect that connections are forgotten when the interface goes down. This is the correct behavior when the next dialup is unlikely to have the same interface address (and hence any established connections are lost anyway). It takes one option: 
```

## echo 'net.ipv4.ip_forward=1' >> /etc/sysctl.conf

- See http://www.ducea.com/2006/08/01/how-to-enable-ip-forwarding-in-linux/:

By default any modern Linux distributions will have IP Forwarding disabled. This is normally a good idea, as most peoples will not need IP Forwarding, but if we are setting up a Linux router/gateway or maybe a VPN server (pptp or ipsec) or just a plain dial-in server then we will need to enable forwarding.

```bash
root@arthurbuntu:~# sysctl net.ipv4.ip_forward
net.ipv4.ip_forward = 0

# Or

root@arthurbuntu:~# cat /proc/sys/net/ipv4/ip_forward
0

# Enabling on the fly
sysctl -w net.ipv4.ip_forward=1

# or 
echo 1 > /proc/sys/net/ipv4/ip_forward

# Permanent setting using /etc/sysctl.conf
echo 'net.ipv4.ip_forward=1' >> /etc/sysctl.conf

# if you already have an entry net.ipv4.ip_forward with the value 0 you can change that 1.
# To see the file, use sysctl -p

# To enable the changes made in sysctl.conf you will need to run the command:
sysctl -p /etc/sysctl.conf
```


## Example settings

- macvlan_iface tveth1 -> jconf.iface_vs

- macvlan_vs_nm 255.255.0.0 -> jconf.iface_vs_nm

- macvlan_vs_gw 10.166.0.1 -> jconf.iface_vs_gw

- macvlan_vs_ip 10.166.55.56 -> jconf.iface_vs_ip

```
nsjail -Mo -u 0 -g 0 -e --keep_caps --chroot / --rw -H toastate -t 0 --rlimit_as hard --rlimit_core hard --rlimit_cpu hard --rlimit_fsize hard --rlimit_nofile hard --rlimit_stack hard --macvlan_iface tveth1 --macvlan_vs_nm 255.255.0.0 --macvlan_vs_gw 10.166.0.1 --macvlan_vs_ip 10.166.55.56 -- go test -v
```

# /etc/resolv.conf

using 127.0.0.53 in /etc/resolv.conf, which is the default, does not allow container (toaster) to resolve hostnames. See https://github.com/containers/libpod/issues/3277.

Default content:

```bash
cat /etc/resolv.conf
# This file is managed by man:systemd-resolved(8). Do not edit.
#
# This is a dynamic resolv.conf file for connecting local clients to the
# internal DNS stub resolver of systemd-resolved. This file lists all
# configured search domains.
#
# Run "systemd-resolve --status" to see details about the uplink DNS servers
# currently in use.
#
# Third party programs must not access this file directly, but only through the
# symlink at /etc/resolv.conf. To manage man:resolv.conf(5) in a different way,
# replace this symlink by a static file or a different symlink.
#
# See man:systemd-resolved.service(8) for details about the supported modes of
# operation for /etc/resolv.conf.

nameserver 127.0.0.53
options edns0
search home

# add this: nameserver 8.8.8.8
```

We need to add a DNS nameserver, like google one, into /etc/resolv.conf if no underlying rootfs (nameserver 8.8.8.8), or into toastate rootfs image if one (/chroot/binary/etc/resolv.conf).

# TODO

## Checkout how to rate limit toaster internet queries, for example dns one

- http://blog.programster.org/rate-limit-requests-with-iptables#:~:targetText=You%20can%20rate%20limit%20connections,of%20connection%2C%20based%20on%20port.

- TUN/TAP linux interface

- VIRTIO-NET (see firecracker)

# Other links


- See http://man7.org/linux/man-pages/man8/ip-netns.8.html
- https://making.pusher.com/per-ip-rate-limiting-with-iptables/
- https://unix.stackexchange.com/questions/163657/set-packet-rate-limit-via-iptables

- https://unix.stackexchange.com/questions/180256/state-of-network-loopback
- https://www.toptal.com/linux/separation-anxiety-isolating-your-system-with-linux-namespaces

# LIBNL

Nsjail uses http://www.infradead.org/~tgr/libnl/

See sys_util/src/libnl