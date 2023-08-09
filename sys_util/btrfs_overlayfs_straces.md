# Resources

- https://fossies.org/linux/btrfs-progs/Documentation/btrfs-ioctl.asciidoc
- https://elixir.bootlin.com/linux/v3.5.7/source/fs/btrfs/ioctl.h
- https://elixir.bootlin.com/linux/latest/source/fs/btrfs/ioctl.c
- https://elixir.bootlin.com/linux/latest/source/include/uapi/linux/btrfs.h#L869
- https://lwn.net/Articles/581558/
- http://man7.org/linux/man-pages/man2/ioctl.2.html
- https://coreutils.gnu.narkive.com/RYdaORhQ/cp-reflink-auto-by-default
- https://unix.stackexchange.com/questions/318705/cp-with-reflink-flag-how-to-determine-if-reflink-is-possible

# BTRFS Straces

## Snapshots

- https://btrfs.wiki.kernel.org/index.php/Incremental_Backup

```sh

# Directly into btrfs parent dir (see below for puting snapshot inside another subvolume)
# /bin/btrfs subvolume snapshot btrmnt/B_1 btrmnt/X
execve("/bin/btrfs", ["/bin/btrfs", "subvolume", "snapshot", "btrmnt/B_1", "btrmnt/X"], 0x7fff94313aa0 /* 40 vars */) = 0
brk(NULL)                               = 0x564d71096000
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
access("/etc/ld.so.preload", R_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/etc/ld.so.cache", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=87625, ...}) = 0
mmap(NULL, 87625, PROT_READ, MAP_PRIVATE, 3, 0) = 0x1481d2ad6000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libuuid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0@\26\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=27112, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x1481d2ad4000
mmap(NULL, 2122112, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1481d26be000
mprotect(0x1481d26c4000, 2093056, PROT_NONE) = 0
mmap(0x1481d28c3000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x5000) = 0x1481d28c3000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libblkid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000\230\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=311720, ...}) = 0
mmap(NULL, 2411776, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1481d2471000
mprotect(0x1481d24b8000, 2097152, PROT_NONE) = 0
mmap(0x1481d26b8000, 20480, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x47000) = 0x1481d26b8000
mmap(0x1481d26bd000, 3328, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x1481d26bd000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libz.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\220\37\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=116960, ...}) = 0
mmap(NULL, 2212016, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1481d2254000
mprotect(0x1481d2270000, 2093056, PROT_NONE) = 0
mmap(0x1481d246f000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1b000) = 0x1481d246f000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/liblzo2.so.2", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000#\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=137256, ...}) = 0
mmap(NULL, 2232416, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1481d2032000
mprotect(0x1481d2053000, 2093056, PROT_NONE) = 0
mmap(0x1481d2252000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x20000) = 0x1481d2252000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/usr/lib/x86_64-linux-gnu/libzstd.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\3604\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=501680, ...}) = 0
mmap(NULL, 2596912, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1481d1db7000
mprotect(0x1481d1e31000, 2093056, PROT_NONE) = 0
mmap(0x1481d2030000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x79000) = 0x1481d2030000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libpthread.so.0", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000b\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=144976, ...}) = 0
mmap(NULL, 2221184, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1481d1b98000
mprotect(0x1481d1bb2000, 2093056, PROT_NONE) = 0
mmap(0x1481d1db1000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x19000) = 0x1481d1db1000
mmap(0x1481d1db3000, 13440, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x1481d1db3000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libc.so.6", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\3\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\260\34\2\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=2030544, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x1481d2ad2000
mmap(NULL, 4131552, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1481d17a7000
mprotect(0x1481d198e000, 2097152, PROT_NONE) = 0
mmap(0x1481d1b8e000, 24576, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1e7000) = 0x1481d1b8e000
mmap(0x1481d1b94000, 15072, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x1481d1b94000
close(3)                                = 0
mmap(NULL, 12288, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x1481d2acf000
arch_prctl(ARCH_SET_FS, 0x1481d2acf8c0) = 0
mprotect(0x1481d1b8e000, 16384, PROT_READ) = 0
mprotect(0x1481d1db1000, 4096, PROT_READ) = 0
mprotect(0x1481d2030000, 4096, PROT_READ) = 0
mprotect(0x1481d2252000, 4096, PROT_READ) = 0
mprotect(0x1481d246f000, 4096, PROT_READ) = 0
mprotect(0x1481d28c3000, 4096, PROT_READ) = 0
mprotect(0x1481d26b8000, 16384, PROT_READ) = 0
mprotect(0x564d7096e000, 20480, PROT_READ) = 0
mprotect(0x1481d2aec000, 4096, PROT_READ) = 0
munmap(0x1481d2ad6000, 87625)           = 0
set_tid_address(0x1481d2acfb90)         = 50
set_robust_list(0x1481d2acfba0, 24)     = 0
rt_sigaction(SIGRTMIN, {sa_handler=0x1481d1b9dcb0, sa_mask=[], sa_flags=SA_RESTORER|SA_SIGINFO, sa_restorer=0x1481d1baa890}, NULL, 8) = 0
rt_sigaction(SIGRT_1, {sa_handler=0x1481d1b9dd50, sa_mask=[], sa_flags=SA_RESTORER|SA_RESTART|SA_SIGINFO, sa_restorer=0x1481d1baa890}, NULL, 8) = 0
rt_sigprocmask(SIG_UNBLOCK, [RTMIN RT_1], NULL, 8) = 0
prlimit64(0, RLIMIT_STACK, NULL, {rlim_cur=RLIM64_INFINITY, rlim_max=RLIM64_INFINITY}) = 0
stat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=14, ...}) = 0
statfs("btrmnt/B_1", {f_type=BTRFS_SUPER_MAGIC, f_bsize=4096, f_blocks=524288, f_bfree=520072, f_bavail=467504, f_files=0, f_ffree=0, f_fsid={val=[2122450911, 1802706900]}, f_namelen=255, f_frsize=4096, f_flags=ST_VALID|ST_RELATIME}) = 0
stat("btrmnt/X", 0x7ffd7dd00a98)        = -1 ENOENT (No such file or directory)
brk(NULL)                               = 0x564d71096000
brk(0x564d710b7000)                     = 0x564d710b7000
statfs("btrmnt", {f_type=BTRFS_SUPER_MAGIC, f_bsize=4096, f_blocks=524288, f_bfree=520072, f_bavail=467504, f_files=0, f_ffree=0, f_fsid={val=[2122450911, 1802706640]}, f_namelen=255, f_frsize=4096, f_flags=ST_VALID|ST_RELATIME}) = 0
stat("btrmnt", {st_mode=S_IFDIR|0755, st_size=18, ...}) = 0
stat("btrmnt", {st_mode=S_IFDIR|0755, st_size=18, ...}) = 0
openat(AT_FDCWD, "btrmnt", O_RDONLY|O_NONBLOCK|O_CLOEXEC|O_DIRECTORY) = 3
fstat(3, {st_mode=S_IFDIR|0755, st_size=18, ...}) = 0
statfs("btrmnt/B_1", {f_type=BTRFS_SUPER_MAGIC, f_bsize=4096, f_blocks=524288, f_bfree=520072, f_bavail=467504, f_files=0, f_ffree=0, f_fsid={val=[2122450911, 1802706900]}, f_namelen=255, f_frsize=4096, f_flags=ST_VALID|ST_RELATIME}) = 0
stat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=14, ...}) = 0
stat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=14, ...}) = 0
openat(AT_FDCWD, "btrmnt/B_1", O_RDONLY|O_NONBLOCK|O_CLOEXEC|O_DIRECTORY) = 4
fstat(4, {st_mode=S_IFDIR|0755, st_size=14, ...}) = 0
fstat(1, {st_mode=S_IFCHR|0620, st_rdev=makedev(136, 1), ...}) = 0
write(1, "Create a snapshot of 'btrmnt/B_1"..., 48Create a snapshot of 'btrmnt/B_1' in 'btrmnt/X'
) = 48
ioctl(3, BTRFS_IOC_SNAP_CREATE_V2, {fd=4, flags=0, name="X"} => {transid=0}) = 0
close(3)                                = 0
close(4)                                = 0
exit_group(0)                           = ?
+++ exited with 0 +++
# ls btrmnt
B_1  B_2  B_3  X
# ls btrmnt/X
bob  hihi
# ls btrmnt/X/bob/alice
test1.txt
# 

# Into a subvolume
# strace /bin/btrfs subvolume snapshot btrmnt/B_1 btrmnt/B_2
execve("/bin/btrfs", ["/bin/btrfs", "subvolume", "snapshot", "btrmnt/B_1", "btrmnt/B_2"], 0x7ffd4bfdf220 /* 40 vars */) = 0
brk(NULL)                               = 0x55e89fe18000
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
access("/etc/ld.so.preload", R_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/etc/ld.so.cache", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=87625, ...}) = 0
mmap(NULL, 87625, PROT_READ, MAP_PRIVATE, 3, 0) = 0x1524b6bfb000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libuuid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0@\26\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=27112, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x1524b6bf9000
mmap(NULL, 2122112, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1524b67e3000
mprotect(0x1524b67e9000, 2093056, PROT_NONE) = 0
mmap(0x1524b69e8000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x5000) = 0x1524b69e8000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libblkid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000\230\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=311720, ...}) = 0
mmap(NULL, 2411776, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1524b6596000
mprotect(0x1524b65dd000, 2097152, PROT_NONE) = 0
mmap(0x1524b67dd000, 20480, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x47000) = 0x1524b67dd000
mmap(0x1524b67e2000, 3328, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x1524b67e2000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libz.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\220\37\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=116960, ...}) = 0
mmap(NULL, 2212016, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1524b6379000
mprotect(0x1524b6395000, 2093056, PROT_NONE) = 0
mmap(0x1524b6594000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1b000) = 0x1524b6594000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/liblzo2.so.2", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000#\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=137256, ...}) = 0
mmap(NULL, 2232416, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1524b6157000
mprotect(0x1524b6178000, 2093056, PROT_NONE) = 0
mmap(0x1524b6377000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x20000) = 0x1524b6377000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/usr/lib/x86_64-linux-gnu/libzstd.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\3604\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=501680, ...}) = 0
mmap(NULL, 2596912, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1524b5edc000
mprotect(0x1524b5f56000, 2093056, PROT_NONE) = 0
mmap(0x1524b6155000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x79000) = 0x1524b6155000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libpthread.so.0", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000b\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=144976, ...}) = 0
mmap(NULL, 2221184, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1524b5cbd000
mprotect(0x1524b5cd7000, 2093056, PROT_NONE) = 0
mmap(0x1524b5ed6000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x19000) = 0x1524b5ed6000
mmap(0x1524b5ed8000, 13440, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x1524b5ed8000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libc.so.6", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\3\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\260\34\2\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=2030544, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x1524b6bf7000
mmap(NULL, 4131552, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1524b58cc000
mprotect(0x1524b5ab3000, 2097152, PROT_NONE) = 0
mmap(0x1524b5cb3000, 24576, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1e7000) = 0x1524b5cb3000
mmap(0x1524b5cb9000, 15072, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x1524b5cb9000
close(3)                                = 0
mmap(NULL, 12288, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x1524b6bf4000
arch_prctl(ARCH_SET_FS, 0x1524b6bf48c0) = 0
mprotect(0x1524b5cb3000, 16384, PROT_READ) = 0
mprotect(0x1524b5ed6000, 4096, PROT_READ) = 0
mprotect(0x1524b6155000, 4096, PROT_READ) = 0
mprotect(0x1524b6377000, 4096, PROT_READ) = 0
mprotect(0x1524b6594000, 4096, PROT_READ) = 0
mprotect(0x1524b69e8000, 4096, PROT_READ) = 0
mprotect(0x1524b67dd000, 16384, PROT_READ) = 0
mprotect(0x55e89e1b8000, 20480, PROT_READ) = 0
mprotect(0x1524b6c11000, 4096, PROT_READ) = 0
munmap(0x1524b6bfb000, 87625)           = 0
set_tid_address(0x1524b6bf4b90)         = 21
set_robust_list(0x1524b6bf4ba0, 24)     = 0
rt_sigaction(SIGRTMIN, {sa_handler=0x1524b5cc2cb0, sa_mask=[], sa_flags=SA_RESTORER|SA_SIGINFO, sa_restorer=0x1524b5ccf890}, NULL, 8) = 0
rt_sigaction(SIGRT_1, {sa_handler=0x1524b5cc2d50, sa_mask=[], sa_flags=SA_RESTORER|SA_RESTART|SA_SIGINFO, sa_restorer=0x1524b5ccf890}, NULL, 8) = 0
rt_sigprocmask(SIG_UNBLOCK, [RTMIN RT_1], NULL, 8) = 0
prlimit64(0, RLIMIT_STACK, NULL, {rlim_cur=RLIM64_INFINITY, rlim_max=RLIM64_INFINITY}) = 0
stat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=14, ...}) = 0
statfs("btrmnt/B_1", {f_type=BTRFS_SUPER_MAGIC, f_bsize=4096, f_blocks=524288, f_bfree=520088, f_bavail=467504, f_files=0, f_ffree=0, f_fsid={val=[2122450911, 1802706900]}, f_namelen=255, f_frsize=4096, f_flags=ST_VALID|ST_RELATIME}) = 0
stat("btrmnt/B_2", {st_mode=S_IFDIR|0755, st_size=0, ...}) = 0
brk(NULL)                               = 0x55e89fe18000
brk(0x55e89fe39000)                     = 0x55e89fe39000
statfs("btrmnt/B_2", {f_type=BTRFS_SUPER_MAGIC, f_bsize=4096, f_blocks=524288, f_bfree=520088, f_bavail=467504, f_files=0, f_ffree=0, f_fsid={val=[2122450911, 1802706903]}, f_namelen=255, f_frsize=4096, f_flags=ST_VALID|ST_RELATIME}) = 0
stat("btrmnt/B_2", {st_mode=S_IFDIR|0755, st_size=0, ...}) = 0
stat("btrmnt/B_2", {st_mode=S_IFDIR|0755, st_size=0, ...}) = 0
openat(AT_FDCWD, "btrmnt/B_2", O_RDONLY|O_NONBLOCK|O_CLOEXEC|O_DIRECTORY) = 3
fstat(3, {st_mode=S_IFDIR|0755, st_size=0, ...}) = 0
statfs("btrmnt/B_1", {f_type=BTRFS_SUPER_MAGIC, f_bsize=4096, f_blocks=524288, f_bfree=520088, f_bavail=467504, f_files=0, f_ffree=0, f_fsid={val=[2122450911, 1802706900]}, f_namelen=255, f_frsize=4096, f_flags=ST_VALID|ST_RELATIME}) = 0
stat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=14, ...}) = 0
stat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=14, ...}) = 0
openat(AT_FDCWD, "btrmnt/B_1", O_RDONLY|O_NONBLOCK|O_CLOEXEC|O_DIRECTORY) = 4
fstat(4, {st_mode=S_IFDIR|0755, st_size=14, ...}) = 0
fstat(1, {st_mode=S_IFCHR|0620, st_rdev=makedev(136, 1), ...}) = 0
write(1, "Create a snapshot of 'btrmnt/B_1"..., 54Create a snapshot of 'btrmnt/B_1' in 'btrmnt/B_2/B_1'
) = 54
ioctl(3, BTRFS_IOC_SNAP_CREATE_V2, {fd=4, flags=0, name="B_1"} => {transid=0}) = 0
close(3)                                = 0
close(4)                                = 0
exit_group(0)                           = ?
+++ exited with 0 +++
# ls btrmnt/B_2
B_1
# ls btrmnt/B_2/B_1
bob  hihi
# 
```

## Create subvolume

```sh
# strace /bin/btrfs subvolume create btrmnt/images
execve("/bin/btrfs", ["/bin/btrfs", "subvolume", "create", "btrmnt/images"], 0x7ffda44dd158 /* 40 vars */) = 0
brk(NULL)                               = 0x55c0fc2bf000
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
access("/etc/ld.so.preload", R_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/etc/ld.so.cache", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=87625, ...}) = 0
mmap(NULL, 87625, PROT_READ, MAP_PRIVATE, 3, 0) = 0x14c8b7bc8000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libuuid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0@\26\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=27112, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x14c8b7bc6000
mmap(NULL, 2122112, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c8b77b0000
mprotect(0x14c8b77b6000, 2093056, PROT_NONE) = 0
mmap(0x14c8b79b5000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x5000) = 0x14c8b79b5000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libblkid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000\230\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=311720, ...}) = 0
mmap(NULL, 2411776, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c8b7563000
mprotect(0x14c8b75aa000, 2097152, PROT_NONE) = 0
mmap(0x14c8b77aa000, 20480, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x47000) = 0x14c8b77aa000
mmap(0x14c8b77af000, 3328, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x14c8b77af000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libz.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\220\37\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=116960, ...}) = 0
mmap(NULL, 2212016, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c8b7346000
mprotect(0x14c8b7362000, 2093056, PROT_NONE) = 0
mmap(0x14c8b7561000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1b000) = 0x14c8b7561000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/liblzo2.so.2", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000#\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=137256, ...}) = 0
mmap(NULL, 2232416, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c8b7124000
mprotect(0x14c8b7145000, 2093056, PROT_NONE) = 0
mmap(0x14c8b7344000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x20000) = 0x14c8b7344000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/usr/lib/x86_64-linux-gnu/libzstd.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\3604\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=501680, ...}) = 0
mmap(NULL, 2596912, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c8b6ea9000
mprotect(0x14c8b6f23000, 2093056, PROT_NONE) = 0
mmap(0x14c8b7122000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x79000) = 0x14c8b7122000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libpthread.so.0", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000b\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=144976, ...}) = 0
mmap(NULL, 2221184, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c8b6c8a000
mprotect(0x14c8b6ca4000, 2093056, PROT_NONE) = 0
mmap(0x14c8b6ea3000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x19000) = 0x14c8b6ea3000
mmap(0x14c8b6ea5000, 13440, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x14c8b6ea5000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libc.so.6", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\3\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\260\34\2\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=2030544, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x14c8b7bc4000
mmap(NULL, 4131552, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c8b6899000
mprotect(0x14c8b6a80000, 2097152, PROT_NONE) = 0
mmap(0x14c8b6c80000, 24576, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1e7000) = 0x14c8b6c80000
mmap(0x14c8b6c86000, 15072, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x14c8b6c86000
close(3)                                = 0
mmap(NULL, 12288, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x14c8b7bc1000
arch_prctl(ARCH_SET_FS, 0x14c8b7bc18c0) = 0
mprotect(0x14c8b6c80000, 16384, PROT_READ) = 0
mprotect(0x14c8b6ea3000, 4096, PROT_READ) = 0
mprotect(0x14c8b7122000, 4096, PROT_READ) = 0
mprotect(0x14c8b7344000, 4096, PROT_READ) = 0
mprotect(0x14c8b7561000, 4096, PROT_READ) = 0
mprotect(0x14c8b79b5000, 4096, PROT_READ) = 0
mprotect(0x14c8b77aa000, 16384, PROT_READ) = 0
mprotect(0x55c0fbf7b000, 20480, PROT_READ) = 0
mprotect(0x14c8b7bde000, 4096, PROT_READ) = 0
munmap(0x14c8b7bc8000, 87625)           = 0
set_tid_address(0x14c8b7bc1b90)         = 25
set_robust_list(0x14c8b7bc1ba0, 24)     = 0
rt_sigaction(SIGRTMIN, {sa_handler=0x14c8b6c8fcb0, sa_mask=[], sa_flags=SA_RESTORER|SA_SIGINFO, sa_restorer=0x14c8b6c9c890}, NULL, 8) = 0
rt_sigaction(SIGRT_1, {sa_handler=0x14c8b6c8fd50, sa_mask=[], sa_flags=SA_RESTORER|SA_RESTART|SA_SIGINFO, sa_restorer=0x14c8b6c9c890}, NULL, 8) = 0
rt_sigprocmask(SIG_UNBLOCK, [RTMIN RT_1], NULL, 8) = 0
prlimit64(0, RLIMIT_STACK, NULL, {rlim_cur=RLIM64_INFINITY, rlim_max=RLIM64_INFINITY}) = 0
stat("btrmnt/images", 0x7ffe502486f8)   = -1 ENOENT (No such file or directory)
brk(NULL)                               = 0x55c0fc2bf000
brk(0x55c0fc2e0000)                     = 0x55c0fc2e0000
statfs("btrmnt", {f_type=BTRFS_SUPER_MAGIC, f_bsize=4096, f_blocks=27904, f_bfree=23736, f_bavail=7168, f_files=0, f_ffree=0, f_fsid={val=[3566125521, 2091123962]}, f_namelen=255, f_frsize=4096, f_flags=ST_VALID|ST_RELATIME}) = 0
stat("btrmnt", {st_mode=S_IFDIR|0755, st_size=0, ...}) = 0
stat("btrmnt", {st_mode=S_IFDIR|0755, st_size=0, ...}) = 0
openat(AT_FDCWD, "btrmnt", O_RDONLY|O_NONBLOCK|O_CLOEXEC|O_DIRECTORY) = 3
fstat(3, {st_mode=S_IFDIR|0755, st_size=0, ...}) = 0
fstat(1, {st_mode=S_IFCHR|0620, st_rdev=makedev(136, 3), ...}) = 0
write(1, "Create subvolume 'btrmnt/images'"..., 33Create subvolume 'btrmnt/images'
) = 33
ioctl(3, BTRFS_IOC_SUBVOL_CREATE, {fd=0, name="images"}) = 0
close(3)                                = 0
exit_group(0)                           = ?
+++ exited with 0 +++
# ls btrmnt
images
# 
```

> Use BTRFS_IOC_SUBVOL_CREATE_V2 instead of BTRFS_IOC_SUBVOL_CREATE

> openat: If pathname is relative and dirfd is the special value AT_FDCWD, then pathname is interpreted relative to the current working directory of the calling process (like open(2)).

## Set quota

```bash
# strace /bin/btrfs qgroup limit 200M btrmnt/B_1
execve("/bin/btrfs", ["/bin/btrfs", "qgroup", "limit", "200M", "btrmnt/B_1"], 0x7ffdf25cb740 /* 40 vars */) = 0
brk(NULL)                               = 0x55e3594f0000
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
access("/etc/ld.so.preload", R_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/etc/ld.so.cache", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=87625, ...}) = 0
mmap(NULL, 87625, PROT_READ, MAP_PRIVATE, 3, 0) = 0x154063b73000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libuuid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0@\26\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=27112, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x154063b71000
mmap(NULL, 2122112, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x15406375b000
mprotect(0x154063761000, 2093056, PROT_NONE) = 0
mmap(0x154063960000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x5000) = 0x154063960000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libblkid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000\230\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=311720, ...}) = 0
mmap(NULL, 2411776, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x15406350e000
mprotect(0x154063555000, 2097152, PROT_NONE) = 0
mmap(0x154063755000, 20480, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x47000) = 0x154063755000
mmap(0x15406375a000, 3328, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x15406375a000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libz.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\220\37\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=116960, ...}) = 0
mmap(NULL, 2212016, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1540632f1000
mprotect(0x15406330d000, 2093056, PROT_NONE) = 0
mmap(0x15406350c000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1b000) = 0x15406350c000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/liblzo2.so.2", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000#\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=137256, ...}) = 0
mmap(NULL, 2232416, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1540630cf000
mprotect(0x1540630f0000, 2093056, PROT_NONE) = 0
mmap(0x1540632ef000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x20000) = 0x1540632ef000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/usr/lib/x86_64-linux-gnu/libzstd.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\3604\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=501680, ...}) = 0
mmap(NULL, 2596912, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x154062e54000
mprotect(0x154062ece000, 2093056, PROT_NONE) = 0
mmap(0x1540630cd000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x79000) = 0x1540630cd000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libpthread.so.0", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000b\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=144976, ...}) = 0
mmap(NULL, 2221184, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x154062c35000
mprotect(0x154062c4f000, 2093056, PROT_NONE) = 0
mmap(0x154062e4e000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x19000) = 0x154062e4e000
mmap(0x154062e50000, 13440, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x154062e50000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libc.so.6", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\3\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\260\34\2\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=2030544, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x154063b6f000
mmap(NULL, 4131552, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x154062844000
mprotect(0x154062a2b000, 2097152, PROT_NONE) = 0
mmap(0x154062c2b000, 24576, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1e7000) = 0x154062c2b000
mmap(0x154062c31000, 15072, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x154062c31000
close(3)                                = 0
mmap(NULL, 12288, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x154063b6c000
arch_prctl(ARCH_SET_FS, 0x154063b6c8c0) = 0
mprotect(0x154062c2b000, 16384, PROT_READ) = 0
mprotect(0x154062e4e000, 4096, PROT_READ) = 0
mprotect(0x1540630cd000, 4096, PROT_READ) = 0
mprotect(0x1540632ef000, 4096, PROT_READ) = 0
mprotect(0x15406350c000, 4096, PROT_READ) = 0
mprotect(0x154063960000, 4096, PROT_READ) = 0
mprotect(0x154063755000, 16384, PROT_READ) = 0
mprotect(0x55e3583df000, 20480, PROT_READ) = 0
mprotect(0x154063b89000, 4096, PROT_READ) = 0
munmap(0x154063b73000, 87625)           = 0
set_tid_address(0x154063b6cb90)         = 37
set_robust_list(0x154063b6cba0, 24)     = 0
rt_sigaction(SIGRTMIN, {sa_handler=0x154062c3acb0, sa_mask=[], sa_flags=SA_RESTORER|SA_SIGINFO, sa_restorer=0x154062c47890}, NULL, 8) = 0
rt_sigaction(SIGRT_1, {sa_handler=0x154062c3ad50, sa_mask=[], sa_flags=SA_RESTORER|SA_RESTART|SA_SIGINFO, sa_restorer=0x154062c47890}, NULL, 8) = 0
rt_sigprocmask(SIG_UNBLOCK, [RTMIN RT_1], NULL, 8) = 0
prlimit64(0, RLIMIT_STACK, NULL, {rlim_cur=RLIM64_INFINITY, rlim_max=RLIM64_INFINITY}) = 0
stat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=0, ...}) = 0
statfs("btrmnt/B_1", {f_type=BTRFS_SUPER_MAGIC, f_bsize=4096, f_blocks=27904, f_bfree=23712, f_bavail=7168, f_files=0, f_ffree=0, f_fsid={val=[3566125521, 2091124221]}, f_namelen=255, f_frsize=4096, f_flags=ST_VALID|ST_RELATIME}) = 0
statfs("btrmnt/B_1", {f_type=BTRFS_SUPER_MAGIC, f_bsize=4096, f_blocks=27904, f_bfree=23712, f_bavail=7168, f_files=0, f_ffree=0, f_fsid={val=[3566125521, 2091124221]}, f_namelen=255, f_frsize=4096, f_flags=ST_VALID|ST_RELATIME}) = 0
stat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=0, ...}) = 0
stat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=0, ...}) = 0
openat(AT_FDCWD, "btrmnt/B_1", O_RDONLY|O_NONBLOCK|O_CLOEXEC|O_DIRECTORY) = 3
fstat(3, {st_mode=S_IFDIR|0755, st_size=0, ...}) = 0
brk(NULL)                               = 0x55e3594f0000
brk(0x55e359511000)                     = 0x55e359511000
ioctl(3, BTRFS_IOC_QGROUP_LIMIT, {qgroupid=0, lim={flags=BTRFS_QGROUP_LIMIT_MAX_RFER, max_rfer=209715200, max_excl=0, rsv_rfer=0, rsv_excl=0}}) = 0
close(3)                                = 0
exit_group(0)                           = ?
+++ exited with 0 +++
# 
```

## cp reflink (COW)

```bash
# WARNING: btrfs cow needs to open each source and destination file and then to do 1 ioctl syscall per pair of source-destination file


# strace /bin/cp -prT --reflink=always btrmnt/B_1 btrmnt/B_2/tests
execve("/bin/cp", ["/bin/cp", "-prT", "--reflink=always", "btrmnt/B_1", "btrmnt/B_2/tests"], 0x7fff9b790780 /* 40 vars */) = 0
brk(NULL)                               = 0x55dd0e726000
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
access("/etc/ld.so.preload", R_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/etc/ld.so.cache", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=87625, ...}) = 0
mmap(NULL, 87625, PROT_READ, MAP_PRIVATE, 3, 0) = 0x151272ce6000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libselinux.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\20b\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=154832, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x151272ce4000
mmap(NULL, 2259152, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1512728ad000
mprotect(0x1512728d2000, 2093056, PROT_NONE) = 0
mmap(0x151272ad1000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x24000) = 0x151272ad1000
mmap(0x151272ad3000, 6352, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x151272ad3000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libacl.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\340\33\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=31232, ...}) = 0
mmap(NULL, 2126336, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1512726a5000
mprotect(0x1512726ac000, 2093056, PROT_NONE) = 0
mmap(0x1512728ab000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x6000) = 0x1512728ab000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libattr.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\260\20\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=18680, ...}) = 0
mmap(NULL, 2113752, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1512724a0000
mprotect(0x1512724a4000, 2093056, PROT_NONE) = 0
mmap(0x1512726a3000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x3000) = 0x1512726a3000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libc.so.6", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\3\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\260\34\2\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=2030544, ...}) = 0
mmap(NULL, 4131552, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1512720af000
mprotect(0x151272296000, 2097152, PROT_NONE) = 0
mmap(0x151272496000, 24576, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1e7000) = 0x151272496000
mmap(0x15127249c000, 15072, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x15127249c000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libpcre.so.3", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0 \25\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=464824, ...}) = 0
mmap(NULL, 2560264, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x151271e3d000
mprotect(0x151271ead000, 2097152, PROT_NONE) = 0
mmap(0x1512720ad000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x70000) = 0x1512720ad000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libdl.so.2", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0P\16\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=14560, ...}) = 0
mmap(NULL, 2109712, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x151271c39000
mprotect(0x151271c3c000, 2093056, PROT_NONE) = 0
mmap(0x151271e3b000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x2000) = 0x151271e3b000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libpthread.so.0", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000b\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=144976, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x151272ce2000
mmap(NULL, 2221184, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x151271a1a000
mprotect(0x151271a34000, 2093056, PROT_NONE) = 0
mmap(0x151271c33000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x19000) = 0x151271c33000
mmap(0x151271c35000, 13440, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x151271c35000
close(3)                                = 0
mmap(NULL, 12288, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x151272cdf000
arch_prctl(ARCH_SET_FS, 0x151272cdf800) = 0
mprotect(0x151272496000, 16384, PROT_READ) = 0
mprotect(0x151271c33000, 4096, PROT_READ) = 0
mprotect(0x151271e3b000, 4096, PROT_READ) = 0
mprotect(0x1512720ad000, 4096, PROT_READ) = 0
mprotect(0x1512726a3000, 4096, PROT_READ) = 0
mprotect(0x1512728ab000, 4096, PROT_READ) = 0
mprotect(0x151272ad1000, 4096, PROT_READ) = 0
mprotect(0x55dd0cfa2000, 4096, PROT_READ) = 0
mprotect(0x151272cfc000, 4096, PROT_READ) = 0
munmap(0x151272ce6000, 87625)           = 0
set_tid_address(0x151272cdfad0)         = 26
set_robust_list(0x151272cdfae0, 24)     = 0
rt_sigaction(SIGRTMIN, {sa_handler=0x151271a1fcb0, sa_mask=[], sa_flags=SA_RESTORER|SA_SIGINFO, sa_restorer=0x151271a2c890}, NULL, 8) = 0
rt_sigaction(SIGRT_1, {sa_handler=0x151271a1fd50, sa_mask=[], sa_flags=SA_RESTORER|SA_RESTART|SA_SIGINFO, sa_restorer=0x151271a2c890}, NULL, 8) = 0
rt_sigprocmask(SIG_UNBLOCK, [RTMIN RT_1], NULL, 8) = 0
prlimit64(0, RLIMIT_STACK, NULL, {rlim_cur=RLIM64_INFINITY, rlim_max=RLIM64_INFINITY}) = 0
statfs("/sys/fs/selinux", 0x7fff47e997d0) = -1 ENOENT (No such file or directory)
statfs("/selinux", 0x7fff47e997d0)      = -1 ENOENT (No such file or directory)
brk(NULL)                               = 0x55dd0e726000
brk(0x55dd0e747000)                     = 0x55dd0e747000
openat(AT_FDCWD, "/proc/filesystems", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0444, st_size=0, ...}) = 0
read(3, "nodev\tsysfs\nnodev\ttmpfs\nnodev\tbd"..., 1024) = 400
read(3, "", 1024)                       = 0
close(3)                                = 0
access("/etc/selinux/config", F_OK)     = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/usr/lib/locale/locale-archive", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=10281936, ...}) = 0
mmap(NULL, 10281936, PROT_READ, MAP_PRIVATE, 3, 0) = 0x15127104b000
close(3)                                = 0
geteuid()                               = 0
stat("btrmnt/B_2/tests", 0x7fff47e99620) = -1 ENOENT (No such file or directory)
lstat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=36, ...}) = 0
lstat("btrmnt/B_2/tests", 0x7fff47e993a0) = -1 ENOENT (No such file or directory)
mkdir("btrmnt/B_2/tests", 0700)         = 0
lstat("btrmnt/B_2/tests", {st_mode=S_IFDIR|0700, st_size=0, ...}) = 0
openat(AT_FDCWD, "btrmnt/B_1", O_RDONLY|O_NONBLOCK|O_CLOEXEC|O_DIRECTORY) = 3
fstat(3, {st_mode=S_IFDIR|0755, st_size=36, ...}) = 0
getdents(3, /* 4 entries */, 32768)     = 112
getdents(3, /* 0 entries */, 32768)     = 0
close(3)                                = 0
lstat("btrmnt/B_1/test2.txt", {st_mode=S_IFREG|0644, st_size=6, ...}) = 0
openat(AT_FDCWD, "btrmnt/B_1/test2.txt", O_RDONLY|O_NOFOLLOW) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=6, ...}) = 0
openat(AT_FDCWD, "btrmnt/B_2/tests/test2.txt", O_WRONLY|O_CREAT|O_EXCL, 0600) = 4
fstat(4, {st_mode=S_IFREG|0600, st_size=0, ...}) = 0
ioctl(4, BTRFS_IOC_CLONE or FICLONE, 3) = 0
utimensat(4, NULL, [{tv_sec=1587818149, tv_nsec=883389626} /* 2020-04-25T14:35:49.883389626+0200 */, {tv_sec=1587818144, tv_nsec=979432418} /* 2020-04-25T14:35:44.979432418+0200 */], 0) = 0
fgetxattr(3, "system.posix_acl_access", 0x7fff47e98ba0, 132) = -1 ENODATA (No data available)
fstat(3, {st_mode=S_IFREG|0644, st_size=6, ...}) = 0
fsetxattr(4, "system.posix_acl_access", "\2\0\0\0\1\0\6\0\377\377\377\377\4\0\4\0\377\377\377\377 \0\4\0\377\377\377\377", 28, 0) = 0
close(4)                                = 0
close(3)                                = 0
lstat("btrmnt/B_1/test1.txt", {st_mode=S_IFREG|0644, st_size=4, ...}) = 0
openat(AT_FDCWD, "btrmnt/B_1/test1.txt", O_RDONLY|O_NOFOLLOW) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=4, ...}) = 0
openat(AT_FDCWD, "btrmnt/B_2/tests/test1.txt", O_WRONLY|O_CREAT|O_EXCL, 0600) = 4
fstat(4, {st_mode=S_IFREG|0600, st_size=0, ...}) = 0
ioctl(4, BTRFS_IOC_CLONE or FICLONE, 3) = 0
utimensat(4, NULL, [{tv_sec=1587818131, tv_nsec=675548407} /* 2020-04-25T14:35:31.675548407+0200 */, {tv_sec=1587818126, tv_nsec=843590489} /* 2020-04-25T14:35:26.843590489+0200 */], 0) = 0
fgetxattr(3, "system.posix_acl_access", 0x7fff47e98ba0, 132) = -1 ENODATA (No data available)
fstat(3, {st_mode=S_IFREG|0644, st_size=4, ...}) = 0
fsetxattr(4, "system.posix_acl_access", "\2\0\0\0\1\0\6\0\377\377\377\377\4\0\4\0\377\377\377\377 \0\4\0\377\377\377\377", 28, 0) = 0
close(4)                                = 0
close(3)                                = 0
utimensat(AT_FDCWD, "btrmnt/B_2/tests", [{tv_sec=1587818109, tv_nsec=987737129} /* 2020-04-25T14:35:09.987737129+0200 */, {tv_sec=1587818106, tv_nsec=459767782} /* 2020-04-25T14:35:06.459767782+0200 */], 0) = 0
lchown("btrmnt/B_2/tests", 0, 0)        = 0
getxattr("btrmnt/B_1", "system.posix_acl_access", 0x7fff47e98f90, 132) = -1 ENODATA (No data available)
stat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=36, ...}) = 0
getxattr("btrmnt/B_1", "system.posix_acl_default", 0x7fff47e98f90, 132) = -1 ENODATA (No data available)
stat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=36, ...}) = 0
setxattr("btrmnt/B_2/tests", "system.posix_acl_access", "\2\0\0\0\1\0\7\0\377\377\377\377\4\0\5\0\377\377\377\377 \0\5\0\377\377\377\377", 28, 0) = 0
removexattr("btrmnt/B_2/tests", "system.posix_acl_default") = 0
lseek(0, 0, SEEK_CUR)                   = -1 ESPIPE (Illegal seek)
close(0)                                = 0
close(1)                                = 0
close(2)                                = 0
exit_group(0)                           = ?
+++ exited with 0 +++
# ls btrmnt/B_2
tests
# ls btrmnt/B_2/tests
test1.txt  test2.txt
# 



```

## Delete subvolume

```bash
# strace /bin/btrfs subvolume delete btrmnt/B_1
execve("/bin/btrfs", ["/bin/btrfs", "subvolume", "delete", "btrmnt/B_1"], 0x7ffea5e1a7d8 /* 40 vars */) = 0
brk(NULL)                               = 0x55e3b476c000
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
access("/etc/ld.so.preload", R_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/etc/ld.so.cache", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=87625, ...}) = 0
mmap(NULL, 87625, PROT_READ, MAP_PRIVATE, 3, 0) = 0x14c4f3355000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libuuid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0@\26\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=27112, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x14c4f3353000
mmap(NULL, 2122112, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c4f2f3d000
mprotect(0x14c4f2f43000, 2093056, PROT_NONE) = 0
mmap(0x14c4f3142000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x5000) = 0x14c4f3142000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libblkid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000\230\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=311720, ...}) = 0
mmap(NULL, 2411776, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c4f2cf0000
mprotect(0x14c4f2d37000, 2097152, PROT_NONE) = 0
mmap(0x14c4f2f37000, 20480, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x47000) = 0x14c4f2f37000
mmap(0x14c4f2f3c000, 3328, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x14c4f2f3c000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libz.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\220\37\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=116960, ...}) = 0
mmap(NULL, 2212016, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c4f2ad3000
mprotect(0x14c4f2aef000, 2093056, PROT_NONE) = 0
mmap(0x14c4f2cee000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1b000) = 0x14c4f2cee000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/liblzo2.so.2", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000#\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=137256, ...}) = 0
mmap(NULL, 2232416, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c4f28b1000
mprotect(0x14c4f28d2000, 2093056, PROT_NONE) = 0
mmap(0x14c4f2ad1000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x20000) = 0x14c4f2ad1000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/usr/lib/x86_64-linux-gnu/libzstd.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\3604\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=501680, ...}) = 0
mmap(NULL, 2596912, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c4f2636000
mprotect(0x14c4f26b0000, 2093056, PROT_NONE) = 0
mmap(0x14c4f28af000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x79000) = 0x14c4f28af000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libpthread.so.0", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000b\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=144976, ...}) = 0
mmap(NULL, 2221184, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c4f2417000
mprotect(0x14c4f2431000, 2093056, PROT_NONE) = 0
mmap(0x14c4f2630000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x19000) = 0x14c4f2630000
mmap(0x14c4f2632000, 13440, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x14c4f2632000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libc.so.6", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\3\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\260\34\2\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=2030544, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x14c4f3351000
mmap(NULL, 4131552, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14c4f2026000
mprotect(0x14c4f220d000, 2097152, PROT_NONE) = 0
mmap(0x14c4f240d000, 24576, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1e7000) = 0x14c4f240d000
mmap(0x14c4f2413000, 15072, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x14c4f2413000
close(3)                                = 0
mmap(NULL, 12288, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x14c4f334e000
arch_prctl(ARCH_SET_FS, 0x14c4f334e8c0) = 0
mprotect(0x14c4f240d000, 16384, PROT_READ) = 0
mprotect(0x14c4f2630000, 4096, PROT_READ) = 0
mprotect(0x14c4f28af000, 4096, PROT_READ) = 0
mprotect(0x14c4f2ad1000, 4096, PROT_READ) = 0
mprotect(0x14c4f2cee000, 4096, PROT_READ) = 0
mprotect(0x14c4f3142000, 4096, PROT_READ) = 0
mprotect(0x14c4f2f37000, 16384, PROT_READ) = 0
mprotect(0x55e3b3c79000, 20480, PROT_READ) = 0
mprotect(0x14c4f336b000, 4096, PROT_READ) = 0
munmap(0x14c4f3355000, 87625)           = 0
set_tid_address(0x14c4f334eb90)         = 58
set_robust_list(0x14c4f334eba0, 24)     = 0
rt_sigaction(SIGRTMIN, {sa_handler=0x14c4f241ccb0, sa_mask=[], sa_flags=SA_RESTORER|SA_SIGINFO, sa_restorer=0x14c4f2429890}, NULL, 8) = 0
rt_sigaction(SIGRT_1, {sa_handler=0x14c4f241cd50, sa_mask=[], sa_flags=SA_RESTORER|SA_RESTART|SA_SIGINFO, sa_restorer=0x14c4f2429890}, NULL, 8) = 0
rt_sigprocmask(SIG_UNBLOCK, [RTMIN RT_1], NULL, 8) = 0
prlimit64(0, RLIMIT_STACK, NULL, {rlim_cur=RLIM64_INFINITY, rlim_max=RLIM64_INFINITY}) = 0
stat("btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=30, ...}) = 0
statfs("btrmnt/B_1", {f_type=BTRFS_SUPER_MAGIC, f_bsize=4096, f_blocks=27904, f_bfree=23712, f_bavail=7168, f_files=0, f_ffree=0, f_fsid={val=[3566125521, 2091124221]}, f_namelen=255, f_frsize=4096, f_flags=ST_VALID|ST_RELATIME}) = 0
brk(NULL)                               = 0x55e3b476c000
brk(0x55e3b478d000)                     = 0x55e3b478d000
getcwd("/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0", 4096) = 68
lstat("/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0/btrmnt", {st_mode=S_IFDIR|0755, st_size=32, ...}) = 0
lstat("/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0/btrmnt/B_1", {st_mode=S_IFDIR|0755, st_size=30, ...}) = 0
statfs("/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0/btrmnt", {f_type=BTRFS_SUPER_MAGIC, f_bsize=4096, f_blocks=27904, f_bfree=23712, f_bavail=7168, f_files=0, f_ffree=0, f_fsid={val=[3566125521, 2091123962]}, f_namelen=255, f_frsize=4096, f_flags=ST_VALID|ST_RELATIME}) = 0
stat("/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0/btrmnt", {st_mode=S_IFDIR|0755, st_size=32, ...}) = 0
stat("/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0/btrmnt", {st_mode=S_IFDIR|0755, st_size=32, ...}) = 0
openat(AT_FDCWD, "/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0/btrmnt", O_RDONLY|O_NONBLOCK|O_CLOEXEC|O_DIRECTORY) = 3
fstat(3, {st_mode=S_IFDIR|0755, st_size=32, ...}) = 0
fstat(1, {st_mode=S_IFCHR|0620, st_rdev=makedev(136, 3), ...}) = 0
write(1, "Delete subvolume (no-commit): '/"..., 111Delete subvolume (no-commit): '/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0/btrmnt/B_1'
) = 111
ioctl(3, BTRFS_IOC_SNAP_DESTROY, {fd=0, name="B_1"}) = 0
close(3)                                = 0
exit_group(0)                           = ?
+++ exited with 0 +++
# 
```


# Overlayfs straces

## Mount overlay

```bash
# strace /bin/mount -t overlay overlay -o lowerdir=lowerdir,upperdir=btrmnt/B_1/upperdir,workdir=btrmnt/B_1/workdir overlaymnt
execve("/bin/mount", ["/bin/mount", "-t", "overlay", "overlay", "-o", "lowerdir=lowerdir,upperdir=btrmn"..., "overlaymnt"], 0x7ffd2d9abea0 /* 40 vars */) = 0
brk(NULL)                               = 0x55a405e56000
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
access("/etc/ld.so.preload", R_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/etc/ld.so.cache", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=87625, ...}) = 0
mmap(NULL, 87625, PROT_READ, MAP_PRIVATE, 3, 0) = 0x14856c0a0000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libmount.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000\253\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=340232, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x14856c09e000
mmap(NULL, 2440288, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14856bc3b000
mprotect(0x14856bc8c000, 2093056, PROT_NONE) = 0
mmap(0x14856be8b000, 12288, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x50000) = 0x14856be8b000
mmap(0x14856be8e000, 3168, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x14856be8e000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libc.so.6", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\3\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\260\34\2\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=2030544, ...}) = 0
mmap(NULL, 4131552, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14856b84a000
mprotect(0x14856ba31000, 2097152, PROT_NONE) = 0
mmap(0x14856bc31000, 24576, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1e7000) = 0x14856bc31000
mmap(0x14856bc37000, 15072, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x14856bc37000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libblkid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000\230\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=311720, ...}) = 0
mmap(NULL, 2411776, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14856b5fd000
mprotect(0x14856b644000, 2097152, PROT_NONE) = 0
mmap(0x14856b844000, 20480, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x47000) = 0x14856b844000
mmap(0x14856b849000, 3328, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x14856b849000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libselinux.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\20b\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=154832, ...}) = 0
mmap(NULL, 2259152, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14856b3d5000
mprotect(0x14856b3fa000, 2093056, PROT_NONE) = 0
mmap(0x14856b5f9000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x24000) = 0x14856b5f9000
mmap(0x14856b5fb000, 6352, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x14856b5fb000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/librt.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\0\"\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=31680, ...}) = 0
mmap(NULL, 2128864, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14856b1cd000
mprotect(0x14856b1d4000, 2093056, PROT_NONE) = 0
mmap(0x14856b3d3000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x6000) = 0x14856b3d3000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libuuid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0@\26\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=27112, ...}) = 0
mmap(NULL, 2122112, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14856afc6000
mprotect(0x14856afcc000, 2093056, PROT_NONE) = 0
mmap(0x14856b1cb000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x5000) = 0x14856b1cb000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libpcre.so.3", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0 \25\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=464824, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x14856c09c000
mmap(NULL, 2560264, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14856ad54000
mprotect(0x14856adc4000, 2097152, PROT_NONE) = 0
mmap(0x14856afc4000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x70000) = 0x14856afc4000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libdl.so.2", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0P\16\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=14560, ...}) = 0
mmap(NULL, 2109712, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14856ab50000
mprotect(0x14856ab53000, 2093056, PROT_NONE) = 0
mmap(0x14856ad52000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x2000) = 0x14856ad52000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libpthread.so.0", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000b\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=144976, ...}) = 0
mmap(NULL, 2221184, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x14856a931000
mprotect(0x14856a94b000, 2093056, PROT_NONE) = 0
mmap(0x14856ab4a000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x19000) = 0x14856ab4a000
mmap(0x14856ab4c000, 13440, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x14856ab4c000
close(3)                                = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x14856c09a000
arch_prctl(ARCH_SET_FS, 0x14856c09b080) = 0
mprotect(0x14856bc31000, 16384, PROT_READ) = 0
mprotect(0x14856ab4a000, 4096, PROT_READ) = 0
mprotect(0x14856ad52000, 4096, PROT_READ) = 0
mprotect(0x14856afc4000, 4096, PROT_READ) = 0
mprotect(0x14856b1cb000, 4096, PROT_READ) = 0
mprotect(0x14856b3d3000, 4096, PROT_READ) = 0
mprotect(0x14856b5f9000, 4096, PROT_READ) = 0
mprotect(0x14856b844000, 16384, PROT_READ) = 0
mprotect(0x14856be8b000, 8192, PROT_READ) = 0
mprotect(0x55a404470000, 4096, PROT_READ) = 0
mprotect(0x14856c0b6000, 4096, PROT_READ) = 0
munmap(0x14856c0a0000, 87625)           = 0
set_tid_address(0x14856c09b350)         = 51
set_robust_list(0x14856c09b360, 24)     = 0
rt_sigaction(SIGRTMIN, {sa_handler=0x14856a936cb0, sa_mask=[], sa_flags=SA_RESTORER|SA_SIGINFO, sa_restorer=0x14856a943890}, NULL, 8) = 0
rt_sigaction(SIGRT_1, {sa_handler=0x14856a936d50, sa_mask=[], sa_flags=SA_RESTORER|SA_RESTART|SA_SIGINFO, sa_restorer=0x14856a943890}, NULL, 8) = 0
rt_sigprocmask(SIG_UNBLOCK, [RTMIN RT_1], NULL, 8) = 0
prlimit64(0, RLIMIT_STACK, NULL, {rlim_cur=RLIM64_INFINITY, rlim_max=RLIM64_INFINITY}) = 0
statfs("/sys/fs/selinux", 0x7ffcc0a94d30) = -1 ENOENT (No such file or directory)
statfs("/selinux", 0x7ffcc0a94d30)      = -1 ENOENT (No such file or directory)
brk(NULL)                               = 0x55a405e56000
brk(0x55a405e77000)                     = 0x55a405e77000
openat(AT_FDCWD, "/proc/filesystems", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0444, st_size=0, ...}) = 0
read(3, "nodev\tsysfs\nnodev\ttmpfs\nnodev\tbd"..., 1024) = 414
read(3, "", 1024)                       = 0
close(3)                                = 0
access("/etc/selinux/config", F_OK)     = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/usr/lib/locale/locale-archive", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=10281936, ...}) = 0
mmap(NULL, 10281936, PROT_READ, MAP_PRIVATE, 3, 0) = 0x148569f62000
close(3)                                = 0
getuid()                                = 0
geteuid()                               = 0
getcwd("/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0", 4096) = 68
lstat("/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0/overlaymnt", {st_mode=S_IFDIR|0755, st_size=4096, ...}) = 0
stat("/sbin/mount.overlay", 0x7ffcc0a93a80) = -1 ENOENT (No such file or directory)
stat("/sbin/fs.d/mount.overlay", 0x7ffcc0a93a80) = -1 ENOENT (No such file or directory)
stat("/sbin/fs/mount.overlay", 0x7ffcc0a93a80) = -1 ENOENT (No such file or directory)
getuid()                                = 0
geteuid()                               = 0
getgid()                                = 0
getegid()                               = 0
prctl(PR_GET_DUMPABLE)                  = 1 (SUID_DUMP_USER)
stat("/run", {st_mode=S_IFDIR|0755, st_size=920, ...}) = 0
lstat("/run/mount/utab", {st_mode=S_IFREG|0644, st_size=1114, ...}) = 0
openat(AT_FDCWD, "/run/mount/utab", O_RDWR|O_CREAT|O_CLOEXEC, 0644) = 3
close(3)                                = 0
mount("overlay", "/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0/overlaymnt", "overlay", MS_MGC_VAL, "lowerdir=lowerdir,upperdir=btrmn"...) = 0
close(1)                                = 0
close(2)                                = 0
exit_group(0)                           = ?
+++ exited with 0 +++
```

## umount overlay

```bash
# strace /bin/umount -f overlaymnt
execve("/bin/umount", ["/bin/umount", "-f", "overlaymnt"], 0x7fff13b420d0 /* 40 vars */) = 0
brk(NULL)                               = 0x557da5516000
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
access("/etc/ld.so.preload", R_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/etc/ld.so.cache", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=87625, ...}) = 0
mmap(NULL, 87625, PROT_READ, MAP_PRIVATE, 3, 0) = 0x1464f8a00000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libmount.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000\253\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=340232, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x1464f89fe000
mmap(NULL, 2440288, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1464f859b000
mprotect(0x1464f85ec000, 2093056, PROT_NONE) = 0
mmap(0x1464f87eb000, 12288, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x50000) = 0x1464f87eb000
mmap(0x1464f87ee000, 3168, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x1464f87ee000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libc.so.6", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\3\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\260\34\2\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=2030544, ...}) = 0
mmap(NULL, 4131552, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1464f81aa000
mprotect(0x1464f8391000, 2097152, PROT_NONE) = 0
mmap(0x1464f8591000, 24576, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1e7000) = 0x1464f8591000
mmap(0x1464f8597000, 15072, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x1464f8597000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libblkid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000\230\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=311720, ...}) = 0
mmap(NULL, 2411776, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1464f7f5d000
mprotect(0x1464f7fa4000, 2097152, PROT_NONE) = 0
mmap(0x1464f81a4000, 20480, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x47000) = 0x1464f81a4000
mmap(0x1464f81a9000, 3328, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x1464f81a9000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libselinux.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\20b\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=154832, ...}) = 0
mmap(NULL, 2259152, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1464f7d35000
mprotect(0x1464f7d5a000, 2093056, PROT_NONE) = 0
mmap(0x1464f7f59000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x24000) = 0x1464f7f59000
mmap(0x1464f7f5b000, 6352, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x1464f7f5b000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/librt.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\0\"\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=31680, ...}) = 0
mmap(NULL, 2128864, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1464f7b2d000
mprotect(0x1464f7b34000, 2093056, PROT_NONE) = 0
mmap(0x1464f7d33000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x6000) = 0x1464f7d33000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libuuid.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0@\26\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=27112, ...}) = 0
mmap(NULL, 2122112, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1464f7926000
mprotect(0x1464f792c000, 2093056, PROT_NONE) = 0
mmap(0x1464f7b2b000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x5000) = 0x1464f7b2b000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libpcre.so.3", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0 \25\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=464824, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x1464f89fc000
mmap(NULL, 2560264, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1464f76b4000
mprotect(0x1464f7724000, 2097152, PROT_NONE) = 0
mmap(0x1464f7924000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x70000) = 0x1464f7924000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libdl.so.2", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0P\16\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=14560, ...}) = 0
mmap(NULL, 2109712, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1464f74b0000
mprotect(0x1464f74b3000, 2093056, PROT_NONE) = 0
mmap(0x1464f76b2000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x2000) = 0x1464f76b2000
close(3)                                = 0
access("/etc/ld.so.nohwcap", F_OK)      = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libpthread.so.0", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0000b\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0755, st_size=144976, ...}) = 0
mmap(NULL, 2221184, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x1464f7291000
mprotect(0x1464f72ab000, 2093056, PROT_NONE) = 0
mmap(0x1464f74aa000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x19000) = 0x1464f74aa000
mmap(0x1464f74ac000, 13440, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x1464f74ac000
close(3)                                = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x1464f89fa000
arch_prctl(ARCH_SET_FS, 0x1464f89fb080) = 0
mprotect(0x1464f8591000, 16384, PROT_READ) = 0
mprotect(0x1464f74aa000, 4096, PROT_READ) = 0
mprotect(0x1464f76b2000, 4096, PROT_READ) = 0
mprotect(0x1464f7924000, 4096, PROT_READ) = 0
mprotect(0x1464f7b2b000, 4096, PROT_READ) = 0
mprotect(0x1464f7d33000, 4096, PROT_READ) = 0
mprotect(0x1464f7f59000, 4096, PROT_READ) = 0
mprotect(0x1464f81a4000, 16384, PROT_READ) = 0
mprotect(0x1464f87eb000, 8192, PROT_READ) = 0
mprotect(0x557da44bd000, 4096, PROT_READ) = 0
mprotect(0x1464f8a16000, 4096, PROT_READ) = 0
munmap(0x1464f8a00000, 87625)           = 0
set_tid_address(0x1464f89fb350)         = 55
set_robust_list(0x1464f89fb360, 24)     = 0
rt_sigaction(SIGRTMIN, {sa_handler=0x1464f7296cb0, sa_mask=[], sa_flags=SA_RESTORER|SA_SIGINFO, sa_restorer=0x1464f72a3890}, NULL, 8) = 0
rt_sigaction(SIGRT_1, {sa_handler=0x1464f7296d50, sa_mask=[], sa_flags=SA_RESTORER|SA_RESTART|SA_SIGINFO, sa_restorer=0x1464f72a3890}, NULL, 8) = 0
rt_sigprocmask(SIG_UNBLOCK, [RTMIN RT_1], NULL, 8) = 0
prlimit64(0, RLIMIT_STACK, NULL, {rlim_cur=RLIM64_INFINITY, rlim_max=RLIM64_INFINITY}) = 0
statfs("/sys/fs/selinux", 0x7ffe0fd73350) = -1 ENOENT (No such file or directory)
statfs("/selinux", 0x7ffe0fd73350)      = -1 ENOENT (No such file or directory)
brk(NULL)                               = 0x557da5516000
brk(0x557da5537000)                     = 0x557da5537000
openat(AT_FDCWD, "/proc/filesystems", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0444, st_size=0, ...}) = 0
read(3, "nodev\tsysfs\nnodev\ttmpfs\nnodev\tbd"..., 1024) = 414
read(3, "", 1024)                       = 0
close(3)                                = 0
access("/etc/selinux/config", F_OK)     = -1 ENOENT (No such file or directory)
openat(AT_FDCWD, "/usr/lib/locale/locale-archive", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=10281936, ...}) = 0
mmap(NULL, 10281936, PROT_READ, MAP_PRIVATE, 3, 0) = 0x1464f68c2000
close(3)                                = 0
getuid()                                = 0
geteuid()                               = 0
getuid()                                = 0
geteuid()                               = 0
getgid()                                = 0
getegid()                               = 0
prctl(PR_GET_DUMPABLE)                  = 1 (SUID_DUMP_USER)
stat("/run", {st_mode=S_IFDIR|0755, st_size=920, ...}) = 0
openat(AT_FDCWD, "/proc/self/mountinfo", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0444, st_size=0, ...}) = 0
read(3, "731 599 8:2 / / rw,relatime - ex"..., 1024) = 1024
lstat("/proc", {st_mode=S_IFDIR|0555, st_size=0, ...}) = 0
lstat("/proc/self", {st_mode=S_IFLNK|0777, st_size=0, ...}) = 0
readlink("/proc/self", "55", 4095)      = 2
lstat("/proc/55", {st_mode=S_IFDIR|0555, st_size=0, ...}) = 0
lstat("/proc/55/mountinfo", {st_mode=S_IFREG|0444, st_size=0, ...}) = 0
read(3, "mpfs tmpfs rw,size=803852k,mode="..., 1024) = 1024
read(3, "/gnome-system-monitor/127 ro,nod"..., 1024) = 1024
read(3, "=remount-ro\n965 731 0:54 / /sys "..., 1024) = 468
read(3, "", 1024)                       = 0
close(3)                                = 0
getuid()                                = 0
geteuid()                               = 0
getgid()                                = 0
getegid()                               = 0
prctl(PR_GET_DUMPABLE)                  = 1 (SUID_DUMP_USER)
stat("/run", {st_mode=S_IFDIR|0755, st_size=920, ...}) = 0
stat("/run/mount/utab", {st_mode=S_IFREG|0644, st_size=1114, ...}) = 0
openat(AT_FDCWD, "/run/mount/utab", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=1114, ...}) = 0
read(3, "SRC=/dev/loop1 TARGET=/snap/gnom"..., 4096) = 1114
read(3, "", 4096)                       = 0
close(3)                                = 0
getcwd("/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0", 4096) = 68
stat("/sbin/umount.overlay", 0x7ffe0fd72170) = -1 ENOENT (No such file or directory)
stat("/sbin/fs.d/umount.overlay", 0x7ffe0fd72170) = -1 ENOENT (No such file or directory)
stat("/sbin/fs/umount.overlay", 0x7ffe0fd72170) = -1 ENOENT (No such file or directory)
lstat("/run/mount/utab", {st_mode=S_IFREG|0644, st_size=1114, ...}) = 0
openat(AT_FDCWD, "/run/mount/utab", O_RDWR|O_CREAT|O_CLOEXEC, 0644) = 3
close(3)                                = 0
stat("/run/mount/utab", {st_mode=S_IFREG|0644, st_size=1114, ...}) = 0
umount2("/home/ubuntu/toastate/build/tests/net_emulation/regions/eu/server_0/overlaymnt", MNT_FORCE) = 0
getpid()                                = 55
rt_sigprocmask(SIG_BLOCK, ~[RTMIN RT_1], [], 8) = 0
openat(AT_FDCWD, "/run/mount/utab.lock", O_RDONLY|O_CREAT|O_CLOEXEC, 0644) = 3
fchmod(3, 0644)                         = 0
flock(3, LOCK_EX)                       = 0
stat("/run/mount/utab", {st_mode=S_IFREG|0644, st_size=1114, ...}) = 0
openat(AT_FDCWD, "/run/mount/utab", O_RDONLY|O_CLOEXEC) = 4
fstat(4, {st_mode=S_IFREG|0644, st_size=1114, ...}) = 0
read(4, "SRC=/dev/loop1 TARGET=/snap/gnom"..., 4096) = 1114
read(4, "", 4096)                       = 0
close(4)                                = 0
close(3)                                = 0
rt_sigprocmask(SIG_SETMASK, [], NULL, 8) = 0
close(1)                                = 0
close(2)                                = 0
exit_group(0)                           = ?
+++ exited with 0 +++
# 
```