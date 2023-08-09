# Ideas for enabling mount namespace pool

- Put all toaster overlay images in a folder bind mounted so shared with all mount namespaces that will be used for toaster execution. Chown each toaster images with a different non root user. You can chown toaster with the same creator and user id with the same linux user. Then when launching toasters in a user namespace, map the inside root user to the outside user corresponding to the toaster code. A toastainer should only be able to mess with the images that has its outside linux user as owner. Use shiftfs to change the owner view of the image to the inside root. This way we can have a pool of premounted and pre pivot rooted mount namespace which all have the same folder of toaster images bindmounted and then before execution just set the pre created user namespace to the right outside user id.
See shiftfs for more information.
Better: User namespace in the pool can have any outside user id as the inside root, just not one already attributed to a toaster image, and then just before execution, from outside the jail(so the scheduler) chow' the toaster image to the chooser user namespace outside user id. Should be less expensive than not being able to create a mount namespace before execution to feed a pool.
    - Maximum number of user on linux is the maximum uint32, e.g. 2^32-1 = 4 294 967 295 : https://www.quora.com/How-many-maximum-users-can-be-created-on-Linux

# TODO

- checkout eBPF possibilities for toastainer:
   - https://docs.google.com/presentation/d/1Wi9K1oF7pfnq7URgM3gQ4AFJqQ0MFfhujbr5EGdO2NY/edit#slide=id.g35f391192_00
   - https://github.com/cilium/ebpf
   - https://github.com/cilium/cilium
   - https://cilium.io/blog/tags/ebpf/
   - https://falco.org/


- checkout https://virtio-fs.gitlab.io/

- Make sure that a toaster cannot maintain its namespaces open indefinetely according to:
```
   Namespace lifetime
       Absent any other factors, a namespace is automatically torn down when
       the last process in the namespace terminates or leaves the namespace.
       However, there are a number of other factors that may pin a namespace
       into existence even though it has no member processes.  These factors
       include the following:

       *  An open file descriptor or a bind mount exists for the correspond‐
          ing /proc/[pid]/ns/* file.

       *  The namespace is hierarchical (i.e., a PID or user namespace), and
          has a child namespace.

       *  It is a user namespace that owns one or more nonuser namespaces.

       *  It is a PID namespace, and there is a process that refers to the
          namespace via a /proc/[pid]/ns/pid_for_children symbolic link.

       *  It is an IPC namespace, and a corresponding mount of an mqueue
          filesystem (see mq_overview(7)) refers to this namespace.

       *  It is a PID namespace, and a corresponding mount of a proc(5)
          filesystem refers to this namespace.
```

- deactivate eBPF for normal linux users: https://utcc.utoronto.ca/~cks/space/blog/linux/DisablingUserEBPF ; https://www.openwall.com/lists/oss-security/2020/03/30/3 ; https://people.canonical.com/~ubuntu-security/cve/2020/CVE-2020-8835.html -> set sysctl kernel.unprivileged_bpf_disabled to 1.

- harden tvs linux in addition of namespaces and overlayfs and btrfs which are common ingredients (more often for overlayfs and user namespaces in particular) in exploitable kernel security issues. 
   - https://utcc.utoronto.ca/~cks/space/blog/linux/OverlayfsNoMore
   - https://utcc.utoronto.ca/~cks/space/blog/linux/UserNamespacesWhySecurityProblems
   - https://people.canonical.com/~ubuntu-security/cve/2016/CVE-2016-1576.html
   - https://people.canonical.com/~ubuntu-security/cve/2016/CVE-2016-1575.html
   - https://seclists.org/oss-sec/2015/q2/717
   - https://www.openwall.com/lists/oss-security/2016/12/06/1
   - http://www.halfdog.net/Security/2015/UserNamespaceOverlayfsSetuidWriteExec/
   - https://www.openwall.com/lists/oss-security/2015/12/31/5

- net namespace: sniff toaster traffic with tuntap for example, to check pattern for ddos security and also to proxy and distribute dns requests across the Toastate network not to hit rate limits quite slow on AWS for example. See also https://molo.ch/.

- Check NSJAIL newest commits regularly to see if we need to change something too

- change code to reflect newest version of Rust (for example in https://blog.rust-lang.org/2020/03/12/Rust-1.42.html no more need to declare extern crate to import their macros)

- Check out new (and integrate) Linux new capability: https://patchwork.kernel.org/patch/11353569/ ; not in our caps module and either in jail package: caps::TOTAL_CAPS = 37;

- New syscalls are coming in linux:  watch_mount(), watch_sb(), and fsinfo()

- Add an option for a firecracker like toastainer for long running task and diversification.