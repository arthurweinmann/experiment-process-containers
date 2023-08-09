# Useful shell commands

```bash
findmnt

cat /proc/self/mountinfo | sed 's/ - .*//'
```

# Useful links

- Kernel documentation: https://www.kernel.org/doc/Documentation/filesystems/sharedsubtree.txt
- linux man mount: http://man7.org/linux/man-pages/man8/mount.8.html & http://man7.org/linux/man-pages/man2/mount.2.html

# LWN articles on mount namespaces

- https://lwn.net/Articles/689856/
- https://lwn.net/Articles/690679/

# Pivot Root

- http://man7.org/linux/man-pages/man2/pivot_root.2.html

# Bind mount

- https://unix.stackexchange.com/questions/198590/what-is-a-bind-mount:

<div class="post-text" itemprop="text">
<h2>What is a bind mount?</h2>

<p>A <em>bind mount</em> is an alternate view of a directory tree. Classically, mounting creates a view of a storage device as a directory tree. A bind mount instead takes an existing directory tree and replicates it under a different point. The directories and files in the bind mount are the same as the original. Any modification on one side is immediately reflected on the other side, since the two views show the same data.</p>

<p>For example, after issuing the Linux command</p>

<pre><code>mount --bind /some/where /else/where
</code></pre>

<p>the directories <code>/some/where</code> and <code>/else/where</code> have the same content.</p>

<p>Unlike a hard link or symbolic link, a bind mount doesn't affect what is stored on the filesystem. It's a property of the live system.</p>

<h2>How do I create a bind mount?</h2>

<h3>bindfs</h3>

<p>The <a href="http://bindfs.org/" rel="noreferrer"><code>bindfs</code></a> filesystem is a <a href="http://en.wikipedia.org/wiki/Filesystem_in_Userspace" rel="noreferrer">FUSE</a> filesystem which creates a view of a directory tree. For example, the command</p>

<pre><code>bindfs /some/where /else/where
</code></pre>

<p>makes <code>/else/where</code> a mount point under which the contents of <code>/some/where</code> are visible.</p>

<p>Since bindfs is a separate filesystem, the files <code>/some/where/foo</code> and <code>/else/where/foo</code> appear as different files to applications (the bindfs filesystem has its own <code>st_dev</code> value). Any change on one side is “magically” reflected on the other side, but the fact that the files are the same is only apparent when one knows how bindfs operates.</p>

<p>Bindfs has no knowledge of mount points, so if there is a mount point under <code>/some/where</code>, it appears as just another directory under <code>/else/where</code>. Mounting or unmounting a filesystem underneath <code>/some/where</code> appears under <code>/else/where</code> as a change of the corresponding directory.</p>

<p>Bindfs can alter some of the file metadata: it can show fake permissions and ownership for files. See the <a href="http://bindfs.org/docs/bindfs.1.html" rel="noreferrer">manual</a> for details, and see below for examples.</p>

<p>A bindfs filesystem can be mounted as a non-root user, you only need the privilege to mount FUSE filesystems. Depending on your distribution, this may require being in the <code>fuse</code> group or be allowed to all users. To unmount a FUSE filesystem, use <code>fusermount -u</code> instead of <code>umount</code>, e.g.</p>

<pre><code>fusermount -u /else/where
</code></pre>

<h3>nullfs</h3>

<p>FreeBSD provides the <a href="https://www.freebsd.org/cgi/man.cgi?query=nullfs&amp;sektion=5" rel="noreferrer"><code>nullfs</code></a> filesystem which creates an alternate view of a filesystem. The following two commands are equivalent:</p>

<pre><code>mount -t nullfs /some/where /else/where
mount_nullfs /some/where /else/where
</code></pre>

<p>After issuing either command, <code>/else/where</code> becomes a mount point at which the contents of <code>/some/where</code> are visible.</p>

<p>Since nullfs is a separate filesystem, the files <code>/some/where/foo</code> and <code>/else/where/foo</code> appear as different files to applications (the nullfs filesystem has its own <code>st_dev</code> value). Any change on one side is “magically” reflected on the other side, but the fact that the files are the same is only apparent when one knows how nullfs operates.</p>

<p>Unlike the FUSE bindfs, which acts at the level of the directory tree, FreeBSD's nullfs acts deeper in the kernel, so mount points under <code>/else/where</code> are not visible: only the tree that is part of the same mount point as <code>/some/where</code> is reflected under <code>/else/where</code>.</p>

<p>The nullfs filesystem may be usable under other BSD variants (OS&nbsp;X, OpenBSD, NetBSD) but it is not compiled as part of the default system.</p>

<h3>Linux bind mount</h3>

<p>Under Linux, bind mounts are available as a kernel feature. You can create one with the <a href="http://man7.org/linux/man-pages/man8/mount.8.html" rel="noreferrer"><code>mount</code></a> command, by passing either the <code>--bind</code> command line option or the <code>bind</code> mount option. The following two commands are equivalent:</p>

<pre><code>mount --bind /some/where /else/where
mount -o bind /some/where /else/where
</code></pre>

<p>Here, the “device” <code>/some/where</code> is not a disk partition like in the case of an on-disk filesystem, but an existing directory. The mount point <code>/else/where</code> must be an existing directory as usual. Note that no filesystem type is specified either way: making a bind mount doesn't involve a filesystem driver, it copies the kernel data structures from the original mount.</p>

<p><code>mount --bind</code> also support mounting a non-directory onto a non-directory: <code>/some/where</code> can be a regular file (in which case <code>/else/where</code> needs to be a regular file too).</p>

<p>A Linux bind mount is mostly indistinguishable from the original. The command <code>df -T /else/where</code> shows the same device and the same filesystem type as <code>df -T /some/where</code>. The files <code>/some/where/foo</code> and <code>/else/where/foo</code> are indistinguishable, as if they were hard links. It is possible to unmount <code>/some/where</code>, in which case <code>/else/where</code> remains mounted.</p>

<p>With older kernels (I don't know exactly when, I think until some 3.x), bind mounts were truly indistinguishable from the original. Recent kernels do track bind mounts and expose the information through PID/mountinfo, which allows <a href="https://unix.stackexchange.com/questions/295525/how-is-findmnt-able-to-list-bind-mounts"><code>findmnt</code> to indicate bind mount as such</a>.</p>

<p>You can put bind mount entries in <code>/etc/fstab</code>. Just include <code>bind</code> (or <code>rbind</code> etc.) in the options, together with any other options you want. The “device” is the existing tree. The filesystem column can contain <code>none</code> or <code>bind</code> (it's ignored, but using a filesystem name would be confusing). For example:</p>

<pre><code>/some/where /readonly/view none bind,ro
</code></pre>

<p>If there are mount points under <code>/some/where</code>, their contents are not visible under <code>/else/where</code>. Instead of <code>bind</code>, you can use <code>rbind</code>, also replicate mount points underneath <code>/some/where</code>. For example, if <code>/some/where/mnt</code> is a mount point then</p>

<pre><code>mount --rbind /some/where /else/where
</code></pre>

<p>is equivalent to</p>

<pre><code>mount --bind /some/where /else/where
mount --bind /some/where/mnt /else/where/mnt
</code></pre>

<p>In addition, Linux allows mounts to be declared as <em>shared</em>, <em>slave</em>, <em>private</em> or <em>unbindable</em>. This affects whether that mount operation is reflected under a bind mount that replicates the mount point. For more details, see <a href="https://www.kernel.org/doc/Documentation/filesystems/sharedsubtree.txt" rel="noreferrer">the kernel documentation</a>.</p>

<p>Linux also provides a way to move mounts: where <code>--bind</code> copies, <code>--move</code> moves a mount point.</p>

<p>It is possible to have different mount options in two bind-mounted directories. There is a quirk, however: making the bind mount and setting the mount options cannot be done atomically, they have to be two successive operations. (Older kernels  did not allow this.) For example, the following commands create a read-only view, but there is a small window of time during which <code>/else/where</code> is read-write:</p>

<pre><code>mount --bind /some/where /else/where
mount -o remount,ro,bind /else/where
</code></pre>

<h3>I can't get bind mounts to work!</h3>

<p>If your system doesn't support FUSE, a classical trick to achieve the same effect is to run an NFS server, make it export the files you want to expose (allowing access to <code>localhost</code>) and mount them on the same machine. This has a significant overhead in terms of memory and performance, so bind mounts have a definite advantage where available (which is on most Unix variants thanks to FUSE).</p>

<h2>Use cases</h2>

<h3>Read-only view</h3>

<p>It can be useful to create a read-only view of a filesystem, either for security reasons or just as a layer of safety to ensure that you won't accidentally modify it.</p>

<p>With bindfs:</p>

<pre><code>bindfs -r /some/where /mnt/readonly
</code></pre>

<p>With Linux, the simple way:</p>

<pre><code>mount --bind /some/where /mnt/readonly
mount -o remount,ro,bind /mnt/readonly
</code></pre>

<p>This leaves a short interval of time during which <code>/mnt/readonly</code> is read-write. If this is a security concern, first create the bind mount in a directory that only root can access, make it read-only, then move it to a public mount point. In the snippet below, note that it's important that <code>/root/private</code> (the directory above the mount point) is private; the original permissions on <code>/root/private/mnt</code> are irrelevant since they are hidden behind the mount point.</p>

<pre><code>mkdir -p /root/private/mnt
chmod 700 /root/private
mount --bind /some/where /root/private/mnt
mount -o remount,ro,bind /root/private/mnt
mount --move /root/private/mnt /mnt/readonly
</code></pre>

<h3>Remapping users and groups</h3>

<p>Filesystems record users and groups by their numerical ID. Sometimes you end up with multiple systems which assign different user IDs to the same person. This is not a problem with network access, but it makes user IDs meaningless when you carry data from one system to another on a disk. Suppose that you have a disk created with a multi-user filesystem (e.g. ext4, btrfs, zfs, UFS, …) on a system where Alice has user ID 1000 and Bob has user ID 1001, and you want to make that disk accessible on a system where Alice has user ID 1001 and Bob has user ID 1000. If you mount the disk directly, Alice's files will appear as owned by Bob (because the user ID is 1001) and Bob's files will appear as owned by Alice (because the user ID is 1000).</p>

<p>You can use bindfs to remap user IDs. First mount the disk partition in a private directory, where only root can access it. Then create a bindfs view in a public area, with user ID and group ID remapping that swaps Alice's and Bob's user IDs and group IDs.</p>

<pre><code>mkdir -p /root/private/alice_disk /media/alice_disk
chmod 700 /root/private
mount /dev/sdb1 /root/private/alice_disk
bindfs --map=1000/1001:1001/1000:@1000/1001:@1001/1000 /root/private/alice_disk /media/alice_disk
</code></pre>

<p>See <a href="https://unix.stackexchange.com/questions/190866/how-does-one-permissibly-access-files-on-non-booted-systems-users-home-folder">How does one permissibly access files on non-booted system's user's home folder?</a> and <a href="https://unix.stackexchange.com/questions/115377/mount-bind-other-user-as-myself">mount --bind other user as myself</a> another examples.</p>

<h3>Mounting in a jail or container</h3>

<p>A <a href="http://en.wikipedia.org/wiki/Chroot" rel="noreferrer">chroot jail</a> or <a href="https://en.wikipedia.org/wiki/OS-level_virtualisation" rel="noreferrer">container</a> runs a process in a subtree of the system's directory tree. This can be useful to run a program with restricted access, e.g. run a network server with access to only its own files and the files that it serves, but not to other data stored on the same computer). A limitation of chroot is that the program is confined to one subtree: it can't access independent subtrees. Bind mounts allow grafting other subtrees onto that main tree. This makes them fundamental to most practical usage of containers under Linux.</p>

<p>For example, suppose that a machine runs a service <code>/usr/sbin/somethingd</code> which should only have access to data under <code>/var/lib/something</code>. The smallest directory tree that contains both of these files is the root. How can the service be confined? One possibility is to make hard links to all the files that the service needs (at least <code>/usr/sbin/somethingd</code> and several shared libraries) under <code>/var/lib/something</code>. But this is cumbersome (the hard links need to be updated whenever a file is upgraded), and doesn't work if <code>/var/lib/something</code> and <code>/usr</code> are on different filesystems. A better solution is to create an ad hoc root and populate it with using mounts:</p>

<pre><code>mkdir /run/something
cd /run/something
mkdir -p etc/something lib usr/lib usr/sbin var/lib/something
mount --bind /etc/something etc/something
mount --bind /lib lib
mount --bind /usr/lib usr/lib
mount --bind /usr/sbin usr/sbin
mount --bind /var/lib/something var/lib/something
mount -o remount,ro,bind etc/something
mount -o remount,ro,bind lib
mount -o remount,ro,bind usr/lib
mount -o remount,ro,bind usr/sbin
chroot . /usr/sbin/somethingd &amp;
</code></pre>

<p>Linux's <a href="http://lwn.net/2001/0301/a/namespaces.php3" rel="noreferrer">mount namespaces</a> generalize chroots. Bind mounts are how namespaces can be populated in flexible ways. See <a href="https://unix.stackexchange.com/questions/81003/making-a-process-read-a-different-file-for-the-same-filename">Making a process read a different file for the same filename</a> for an example.</p>

<h3>Running a different distribution</h3>

<p>Another use of chroots is to install a different distribution in a directory and run programs from it, even when they require files at hard-coded paths that are not present or have different content on the base system. This can be useful, for example, to install a 32-bit distribution on a 64-bit system that doesn't support mixed packages, to install older releases of a distribution or other distributions to test compatibility, to install a newer release to test the latest features while maintaining a stable base system, etc. See <a href="https://unix.stackexchange.com/questions/12956/how-do-i-run-32-bit-programs-on-a-64-bit-debian-ubuntu">How do I run 32-bit programs on a 64-bit Debian/Ubuntu?</a> for an example on Debian/Ubuntu.</p>

<p>Suppose that you have an installation of your distribution's latest packages under the directory <code>/f/unstable</code>, where you run programs by switching to that directory with <code>chroot /f/unstable</code>. To make home directories available from this installations, bind mount them into the chroot:</p>

<pre><code>mount --bind /home /f/unstable/home
</code></pre>

<p>The program <a href="https://packages.debian.org/jessie/schroot" rel="noreferrer">schroot</a> does this automatically.</p>

<h3>Accessing files hidden behind a mount point</h3>

<p>When you mount a filesystem on a directory, this hides what is behind the directory. The files in that directory become inaccessible until the directory is unmounted. Because BSD nullfs and Linux bind mounts operate at a lower level than the mount infrastructure, a nullfs mount or a bind mount of a filesystem exposes directories that were hidden behind submounts in the original.</p>

<p>For example, suppose that you have a tmpfs filesystem mounted at <code>/tmp</code>. If there were files under <code>/tmp</code> when the tmpfs filesystem was created, these files may still remain, effectively inaccessible but taking up disk space. Run</p>

<pre><code>mount --bind / /mnt
</code></pre>

<p>(Linux) or</p>

<pre><code>mount -t nullfs / /mnt
</code></pre>

<p>(FreeBSD) to create a view of the root filesystem at <code>/mnt</code>. The directory <code>/mnt/tmp</code> is the one from the root filesystem.</p>

<h3>NFS exports at different paths</h3>

<p>Some NFS servers (such as the Linux kernel NFS server before NFSv4) always advertise the actual directory location when they export a directory. That is, when a client requests <code>server:/requested/location</code>, the server serves the tree at the location <code>/requested/location</code>. It is sometimes desirable to allow clients to request <code>/request/location</code> but actually serve files under <code>/actual/location</code>. If your NFS server doesn't support serving an alternate location, you can create a bind mount for the expected request, e.g.</p>

<pre><code>/requested/location *.localdomain(rw,async)
</code></pre>

<p>in <code>/etc/exports</code> and the following in <code>/etc/fstab</code>:</p>

<pre><code>/actual/location /requested/location bind bind
</code></pre>

<h3>A substitute for symbolic links</h3>

<p>Sometimes you'd like to make symbolic link to make a file <code>/some/where/is/my/file</code> appear under <code>/else/where</code>, but the application that uses <code>file</code> expands symbolic links and rejects <code>/some/where/is/my/file</code>. A bind mount can work around this: bind-mount <code>/some/where/is/my</code> to <code>/else/where/is/my</code>, and then <a href="http://man7.org/linux/man-pages/man3/realpath.3.html" rel="noreferrer"><code>realpath</code></a> will report <code>/else/where/is/my/file</code> to be under <code>/else/where</code>, not under <code>/some/where</code>.</p>

<h2>Side effects of bind mounts</h2>

<h3>Recursive directory traversals</h3>

<p>If you use bind mounts, you need to take care of applications that traverse the filesystem tree recursively, such as backups and indexing (e.g. to build a <a href="http://en.wikipedia.org/wiki/Locate_(Unix)" rel="noreferrer">locate</a> database).</p>

<p>Usually, bind mounts should be excluded from recursive directory traversals, so that each directory tree is only traversed once, at the original location. With bindfs and nullfs, configure the traversal tool to ignore these filesystem types, if possible. Linux bind mounts cannot be recognized as such: the new location is equivalent to the original. With Linux bind mounts, or with tools that can only exclude paths and not filesystem types, you need to exclude the mount points for the bind mounts.</p>

<p>Traversals that stop at filesystem boundaries (e.g. <code>find -xdev</code>, <code>rsync -x</code>, <code>du -x</code>, …) will automatically stop when they encounter a bindfs or nullfs mount point, because that mount point is a different filesystem. With Linux bind mounts, the situation is a bit more complicated: there is a filesystem boundary only if the bind mount is grafting a different filesystem, not if it is grafting another part of the same filesystem.</p>

<h2>Going beyond bind mounts</h2>

<p>Bind mounts provide a view of a directory tree at a different location. They expose the same files, possibly with different mount options and (with bindfs) different ownership and permissions. Filesystems that present an altered view of a directory tree are called <em>overlay filesystems</em> or <em>stackable filesystems</em>. There are many other overlay filesystems that perform more advanced transformations. Here are a few common ones. If your desired use case is not covered here, check the <a href="http://sourceforge.net/p/fuse/wiki/FileSystems/" rel="noreferrer">repository of FUSE filesystems</a>.</p>

<ul>
<li><a href="http://sourceforge.net/projects/loggedfs/" rel="noreferrer">loggedfs</a> — log all filesystem access for debugging or monitoring purposes (<a href="https://unix.stackexchange.com/questions/13794/loggedfs-configuration-file-syntax/13797#13797">configuration file syntax</a>, <a href="https://unix.stackexchange.com/questions/6068/is-it-possible-to-find-out-what-program-or-script-created-a-given-file/6080#6080">Is it possible to find out what program or script created a given file?</a>, <a href="https://unix.stackexchange.com/questions/18844/list-the-files-accessed-by-a-program/18872#18872">List the files accessed by a program</a>)</li>
</ul>

<h3>Filter visible files</h3>

<ul>
<li><a href="http://clamfs.sourceforge.net/" rel="noreferrer">clamfs</a> — run files through a virus scanner when they are read</li>
<li><a href="http://filterfs.sourceforge.net/" rel="noreferrer">filterfs</a> — hide parts of a filesystem</li>
<li><a href="https://github.com/cognusion/fuse-rofs" rel="noreferrer">rofs</a> — a read-only view. Similar to <code>bindfs -r</code>, just a little more lightweight.</li>
<li><p><a href="http://en.wikipedia.org/wiki/Union_mount" rel="noreferrer">Union mounts</a> — present multiple filesystems (called <em>branches</em>) under a single directory: if <code>tree1</code> contains <code>foo</code> and <code>tree2</code> contains <code>bar</code> then their union view contains both <code>foo</code> and <code>bar</code>. New files are written to a specific branch, or to a branch chosen according to more complex rules. There are several implementations of this concept, including:</p>

<ul>
<li><a href="http://aufs.sourceforge.net/" rel="noreferrer">aufs</a> — Linux kernel implementation, but <a href="https://en.wikipedia.org/wiki/Aufs" rel="noreferrer">rejected upstream many times</a></li>
<li><a href="http://funionfs.apiou.org/?lng=en" rel="noreferrer">funionfs</a> — FUSE implementation</li>
<li><a href="http://svn.uvw.ru/mhddfs/trunk/README" rel="noreferrer">mhddfs</a> — FUSE, write files to a branch based on free space</li>
<li><a href="https://git.kernel.org/cgit/linux/kernel/git/torvalds/linux.git/tree/Documentation/filesystems/overlayfs.txt" rel="noreferrer">overlay</a> — Linux kernel implementation, merged upstream in Linux v3.18</li>
<li><a href="https://github.com/rpodgorny/unionfs-fuse" rel="noreferrer">unionfs-fuse</a> — FUSE, with caching and copy-on-write features</li>
</ul></li>
</ul>

<h3>Modify file names and metadata</h3>

<ul>
<li><a href="http://brain-dump.org/projects/ciopfs/" rel="noreferrer">ciopfs</a> — case-insensitive filenames (can be useful to mount Windows filesystems)</li>
<li><a href="http://fuse-convmvfs.sourceforge.net/" rel="noreferrer">convmvfs</a> — convert filenames between character sets (<a href="https://unix.stackexchange.com/questions/67232/same-file-different-filename-due-to-encoding-problem/67273#67273">example</a>)</li>
<li><a href="http://sourceforge.net/projects/posixovl/" rel="noreferrer">posixovl</a> — store Unix filenames and other metadata (permissions, ownership, …) on more restricted filesystems such as VFAT (<a href="https://unix.stackexchange.com/questions/108890/what-is-the-best-way-to-synchronize-files-to-a-vfat-partition/108937#108937">example</a>)</li>
</ul>

<h3>View altered file contents</h3>

<ul>
<li><a href="http://avf.sourceforge.net/" rel="noreferrer">avfs</a> — for each archive file, present a directory with the content of the archive (<a href="https://unix.stackexchange.com/questions/13749/how-do-i-recursively-grep-through-compressed-archives/13798#13798">example</a>, <a href="https://unix.stackexchange.com/search?tab=votes&amp;q=avfs%20is%3aanswer">more examples</a>). There are also many <a href="http://sourceforge.net/p/fuse/wiki/ArchiveFileSystems/" rel="noreferrer">FUSE filesystems that expose specific archives as directories</a>.</li>
<li><a href="http://users.softlab.ntua.gr/~thkala/projects/fuseflt/" rel="noreferrer">fuseflt</a> — run files through a pipeline when reading them, e.g. to recode text files or media files (<a href="https://unix.stackexchange.com/questions/33574/how-to-use-grep-ack-with-files-in-arbitrary-encoding/33580#33580">example</a>)</li>
<li><a href="https://github.com/vasi/lzopfs" rel="noreferrer">lzopfs</a> — transparent decompression of compressed files</li>
<li><a href="http://khenriks.github.io/mp3fs/" rel="noreferrer">mp3fs</a> — transcode FLAC files to MP3 when they are read (<a href="https://unix.stackexchange.com/questions/37701/how-to-encode-huge-flac-files-into-mp3-and-other-files-like-aac/115695#115695">example</a>)</li>
<li><a href="https://code.google.com/p/scriptfs/" rel="noreferrer">scriptfs</a> — execute scripts to serve content (a sort of local CGI) (<a href="https://unix.stackexchange.com/questions/181673/using-process-substitution-to-trick-programs-expecting-files-with-specific-exte/181680#181680">example</a>)</li>
</ul>

<h3>Modify the way content is stored</h3>

<ul>
<li><a href="https://code.google.com/p/chironfs/" rel="noreferrer">chironfs</a> — replicate files onto multiple underlying storage (<a href="https://unix.stackexchange.com/questions/14544/raid-1-lvm-at-the-level-of-directories-aka-mknodding-a-directory">RAID-1 at the directory tree level</a>)</li>
<li><a href="http://n0x.org/copyfs" rel="noreferrer">copyfs</a> — keep copies of all versions of the files</li>
<li><a href="http://www.arg0.net/encfs" rel="noreferrer">encfs</a> — encrypt files</li>
<li><a href="https://code.google.com/p/pcachefs/" rel="noreferrer">pcachefs</a> — on-disk cache layer for slow remote filesystems</li>
<li><a href="https://github.com/vi/simplecowfs" rel="noreferrer">simplecowfs</a> — store changes via the provided view in memory, leaving the original files intact</li>
<li><a href="http://wayback.sourceforge.net/" rel="noreferrer">wayback</a> — keep copies of all versions of the files</li>
</ul>
    </div>

