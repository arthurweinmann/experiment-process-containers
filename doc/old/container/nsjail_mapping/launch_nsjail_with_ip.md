

```
ip link add dev tveth0 type veth peer name tveth1
ip link set dev tveth0 up
ip link set dev tveth1 up
ip addr add 10.166.0.1/16 broadcast 10.166.255.255 dev tveth0
iptables -t nat -A POSTROUTING -s 10.166.0.0/16 ! -d 172.16.0.0/12 -j MASQUERADE 

echo 'net.ipv4.ip_forward=1' >> /etc/sysctl.conf

nsjail -Mo -u 0 -g 0 -e --keep_caps --chroot / --rw -H toastate -t 0 --rlimit_as hard --rlimit_core hard --rlimit_cpu hard --rlimit_fsize hard --rlimit_nofile hard --rlimit_stack hard --macvlan_iface tveth1 --macvlan_vs_nm 255.255.0.0 --macvlan_vs_gw 10.166.0.1 --macvlan_vs_ip 10.166.55.56 -- go test -v
```