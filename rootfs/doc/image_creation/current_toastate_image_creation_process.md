# Build images

## Basics

Un serveur (Boite en carton suffit pour go et binary, supercalculateur recommandé pour node)

- Binary => pas de mini
- Golang mini 500mo de ram, Va plus vite avec + de cpus (En général 2 minutes max)

```bash
#!/bin/bash
set -eu;
set -o pipefail;

# SETUP
echo "Start SETUP"
apt-get update
apt-get upgrade -y
apt-get install -y build-essential protobuf-compiler libprotobuf-dev pkg-config bison flex awscli > /dev/null

# NSJAIL
echo "Start NSJAIL"
export STTEMP="$(mktemp -d)"
cd $STTEMP
git clone https://github.com/google/nsjail.git -b 2.6 nsjail > /dev/null
cd nsjail
make -j$(nproc) > /dev/null
mv nsjail /usr/bin
cd /home/ubuntu
rm -r "$STTEMP"

# Chroots
echo "Start chroots"
mkdir -p /chroot
cd /chroot

# Net
echo "Start network"
ip link add dev tveth0 type veth peer name tveth1
ip link set dev tveth0 up
ip link set dev tveth1 up
ip addr add 10.166.0.1/16 broadcast 10.166.255.255 dev tveth0
iptables -t nat -A POSTROUTING -s 10.166.0.0/16 ! -d 172.16.0.0/12 -j MASQUERADE
echo 'net.ipv4.ip_forward=1' >> /etc/sysctl.conf
echo "nameserver 8.8.8.8" > /etc/resolv2.conf
sysctl -p
```

## Image binary

C'est l'image de base alpine

(Wget depuis https://alpinelinux.org/downloads/ mini root filesystems X86_64 )

```bash
mkdir -p /chroot/binary
cd /chroot/binary
wget http://dl-cdn.alpinelinux.org/alpine/v3.9/releases/x86_64/alpine-minirootfs-3.9.4-x86_64.tar.gz
tar xvf alpine-minirootfs-3.9.4-x86_64.tar.gz
rm alpine-minirootfs-3.9.4-x86_64.tar.gz
cd ..
cp /etc/resolv2.conf /chroot/binary/etc/resolv.conf
nsjail -Mo -u 0 -g 0 --keep_caps -c /chroot/binary --rw -H toastate -D /root -t 0 -E PATH=/bin:/usr/bin:/usr/local/go/bin:/sbin -R /etc/resolv2.conf:/etc/resolv.conf --disable_proc --rlimit_as hard --rlimit_core hard --rlimit_cpu hard --rlimit_fsize hard --rlimit_nofile hard --rlimit_stack hard --macvlan_iface tveth1 --macvlan_vs_ip 10.166.55.56 --macvlan_vs_nm 255.255.0.0 --macvlan_vs_gw 10.166.0.1 -- /bin/sh -i
apk add --no-cache ca-certificates
```
Ctrl-D
`tar cvf binary.tar binary`

## Image golang

(Wget depuis https://alpinelinux.org/downloads/ mini root filesystems X86_64 )

```bash
mkdir -p /chroot/golang-112
cd /chroot/golang-112
wget http://dl-cdn.alpinelinux.org/alpine/v3.9/releases/x86_64/alpine-minirootfs-3.9.4-x86_64.tar.gz
tar xvf alpine-minirootfs-3.9.4-x86_64.tar.gz
rm alpine-minirootfs-3.9.4-x86_64.tar.gz
cd ..
cp /etc/resolv2.conf /chroot/golang-112/etc/resolv.conf
nsjail -Mo -u 0 -g 0 --keep_caps -c /chroot/golang-112 --rw -H toastate -D /root -t 0 -E PATH=/bin:/usr/bin:/usr/local/go/bin:/sbin --disable_proc --rlimit_as hard --rlimit_core hard --rlimit_cpu hard --rlimit_fsize hard --rlimit_nofile hard --rlimit_stack hard --macvlan_iface tveth1 --macvlan_vs_ip 10.166.55.55 --macvlan_vs_nm 255.255.0.0 --macvlan_vs_gw 10.166.0.1 -- /bin/sh -i
```

Disclaimer: La suite est pompée de https://github.com/docker-library/golang/blob/69f2d2a132565bf60afc91813801a3bdcc981526/1.10/alpine3.8/Dockerfile
Ne pas jouer avec les variables d'environnement, si c'est pas obligatoire on évite

Executer les commandes une par une

```bash
echo 'hosts: files dns' > /etc/nsswitch.conf
apk add --no-cache ca-certificates git g++ gcc

apk add --no-cache --virtual .build-deps bash gcc musl-dev openssl go
export GOROOT_BOOTSTRAP="$(go env GOROOT)" GOOS="$(go env GOOS)" GOARCH="$(go env GOARCH)" GOHOSTOS="$(go env GOHOSTOS)" GOHOSTARCH="$(go env GOHOSTARCH)"
wget -O go.tgz "https://golang.org/dl/go1.10.4.src.tar.gz"
echo '6fe44965ed453cd968a81988523e9b0e794d3a478f91fd7983c28763d52d5781 *go.tgz' | sha256sum -c -
tar -C /usr/local -xzf go.tgz
rm go.tgz
cd /usr/local/go/src
./make.bash
rm -rf /usr/local/go/pkg/bootstrap /usr/local/go/pkg/obj
apk del .build-deps
go version
exit
```

`tar cvf golang-112.tar golang-112`

Récupérer le .tar et le balancer sur S3


## Image nodejs

(Wget depuis https://alpinelinux.org/downloads/ mini root filesystems X86_64 )

```bash
mkdir -p /chroot/nodejs-10
cd /chroot/nodejs-10
wget http://dl-cdn.alpinelinux.org/alpine/v3.8/releases/x86_64/alpine-minirootfs-3.8.1-x86_64.tar.gz
tar xvf alpine-minirootfs-3.8.1-x86_64.tar.gz
rm alpine-minirootfs-3.8.1-x86_64.tar.gz
cd ..
nsjail -Mo -u 0 -g 0 --keep_caps -c /chroot/nodejs-10 --rw -H toastate -D /root -t 0 -E PATH=/bin:/usr/bin:/usr/local/go/bin:/sbin:/usr/local/bin -E HOME=/root -R /etc/resolv2.conf:/etc/resolv.conf -R /dev --rlimit_as hard --rlimit_core hard --rlimit_cpu hard --rlimit_fsize hard --rlimit_nofile hard --rlimit_stack hard --macvlan_iface tveth1 --macvlan_vs_ip 10.166.55.55 --macvlan_vs_nm 255.255.0.0 --macvlan_vs_gw 10.166.0.1 -- /bin/sh -i
```

Disclaimer: La suite est pompée de https://github.com/nodejs/docker-node/blob/45fa3ebe94598758b9c9e4a382236fc7e879e2e6/10/alpine/Dockerfile
et 
https://github.com/nodejs/node#release-team

Ne pas jouer avec les variables d'environnement, si c'est pas obligatoire on évite

Executer les commandes une par une (Sauf le for bien sur)

```bash
apk add --no-cache ca-certificates libstdc++
apk add --no-cache --virtual .build-deps binutils-gold curl g++ gcc gnupg libgcc linux-headers make python
curl -fsSLO --compressed "https://nodejs.org/dist/v10.15.0/node-v10.15.0.tar.xz"
curl -fsSLO --compressed "https://nodejs.org/dist/v10.15.0/SHASUMS256.txt.asc"
for key in \
    94AE36675C464D64BAFA68DD7434390BDBE9B9C5 \
    FD3A5288F042B6850C66B31F09FE44734EB7990E \
    71DCFD284A79C3B38668286BC97EC7A07EDE3FC1 \
    DD8F2338BAE7501E3DD5AC78C273792F7D83545D \
    C4F0DFFF4E8C1A8236409D08E73BC641CC11F4C8 \
    B9AE9905FFD7803F25714661B63B535A4C206CA9 \
    56730D5401028683275BD23C23EFEFE93C4CFFFE \
    77984A986EBC2AA786BC0F66B01FBB92821C587A \
    8FCCA13FEF1D0C2E91008E09770F7A9A5AE15600 \
; do \
    gpg --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys "$key" || \
    gpg --keyserver hkp://ipv4.pool.sks-keyservers.net --recv-keys "$key" || \
    gpg --keyserver hkp://pgp.mit.edu:80 --recv-keys "$key" ; \
done
gpg --batch --decrypt --output SHASUMS256.txt SHASUMS256.txt.asc
grep " node-v10.15.0.tar.xz\$" SHASUMS256.txt | sha256sum -c -
tar -xf "node-v10.15.0.tar.xz"
cd "node-v10.15.0"
./configure
make -j$(getconf _NPROCESSORS_ONLN)
make install
apk del .build-deps
```

`tar cvf nodejs-10.tar nodejs-10`

Récupérer le .tar et le balancer sur S3

Additional notes: 
```
mount -t overlay overlay -olowerdir=/chroot/binary,upperdir=/chroot/urust,workdir=/chroot/ofsworkrust /chroot/rust-131/
mount -t overlay overlay -olowerdir=/chroot/binary,upperdir=/chroot/unodejs,workdir=/chroot/ofsworknode /chroot/nodejs-12/
mount -t overlay overlay -olowerdir=/chroot/binary,upperdir=/chroot/ugolang,workdir=/chroot/ofsworkgolang /chroot/golang-112
mount -t overlay overlay -olowerdir=/chroot/binary,upperdir=/chroot/ulua,workdir=/chroot/ofsworklua /chroot/luajit210/
mount -t overlay overlay -olowerdir=/chroot/binary:/chroot/ugolang:/chroot/ulua:/chroot/urust:/chroot/unodejs,upperdir=/chroot/utoastate,workdir=/chroot/ofsworktoastate /chroot/toastate
```