# The idea

The idea to apply quota with btrfs and manage execution overlayfs is as follow:

- Instead of making all mounts privates, as does nsjail in mnt.rs::init_clone_ns, we make them a slave. So mount events won't propagate from child (jail) to parent, but we will be able to propagate overlayfs mount from parent to jailed child mount namespace attributed per userid.

- As for the old TVS, we have a pool of pre-created btrfs subvolume with the default quota already set

- When we want to execute some toaster image, we mount overlay it, with its workdir and upperdir in the btrfs subvolume (and I guess toaster code symlink), to a unique directory in the jail root. Then from inside the jail, we chroot into this directory. This is to say that from inside the child mount namespace, we see repos poping and through the passed fd from parent we get in which one to chroot in order to execute.

# Better manage OS images

## difference between kernel and rootfs

https://stackoverflow.com/questions/54054533/rootfilesystem-vs-kernel-updating:
```
Kernel is usually one image file (like zImage). In ARM systems kernel also needs device tree file, but let's avoid it for now. RootFS, in turn, is a file system that contains all files in your /, like binaries (init, bash), config files (/etc), user home directory, and so on. Sometimes RootFs contains kernel image file, sometimes it doesn't, depends on your particular system.
```

## Firecracker rootfs image creation process

https://github.com/firecracker-microvm/firecracker/blob/master/docs/rootfs-and-kernel-setup.md

See also https://github.com/firecracker-microvm/firecracker/blob/master/docs/getting-started.md

## Check out ELF file format

-> instead of mounting, maybe pass a fd to a linux distri as an ELF file

See https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
See https://en.wikipedia.org/wiki/Vmlinux

## Useful links

- From docker container to Bootable Linux Disk image: https://iximiuz.com/en/posts/from-docker-container-to-bootable-linux-disk-image/

- https://jvns.ca/blog/2019/11/18/how-containers-work--overlayfs/

# Info about disks:

```BASH
cat /proc/partitions

mount | grep ^/

cat /etc/mtab

cat /proc/mounts

fdisk

lsblk
```