# Mappings

## Mapping 1: Toaster binary execution with overlay fs

### checkout

 --uid_mapping|-U VALUE
	Add a custom uid mapping of the form inside_uid:outside_uid:count. Setting this requires newuidmap (set-uid) to be present
 --gid_mapping|-G VALUE
	Add a custom gid mapping of the form inside_gid:outside_gid:count. Setting this requires newgidmap (set-uid) to be present

   --user|-u VALUE
	Username/uid of processess inside the jail (default: your current uid). You can also use inside_ns_uid:outside_ns_uid:count convention here. Can be specified multiple times
  --group|-g VALUE
	Groupname/gid of processess inside the jail (default: your current gid). You can also use inside_ns_gid:global_ns_gid:count convention here. Can be specified multiple times

### NSJail used configuration

> Bash Command:

```
/usr/bin/nsjail 

**** Generic params, all exe runtime

    --rlimit_fsize hard 
    --rlimit_nofile 128 
    --rlimit_as {X} 
    --rlimit_stack {X} 
    --rlimit_nproc 132
    --rlimit_cpu hard

    --macvlan_iface | -I  tveth1 (Interface which will be cloned (MACVLAN) and put inside the subprocess' namespace as 'vs')
    --macvlan_vs_ip 10.166.Y.Y (IP of the 'vs' interface (e.g. "192.168.0.1"))
    --macvlan_vs_nm  255.255.0.0 (Netmask of the 'vs' interface (e.g. "255.255.255.0"))
    --macvlan_vs_gw  10.166.0.1 (Default GW for the 'vs' interface (e.g. "192.168.0.1"))

    --seccomp_string {kafelPolicy(see below)} (String with kafel seccomp-bpf policy (see kafel/)) -> used in preparePolicy in sandbox.cc to create the bpf filter policy

**** Params specific to toaster binary exe (golang run)

    --cwd | -D /root (Directory in the namespace the process will run (default: '/'))

    --chroot|-c {overlaydir.MountPoint} (Directory containing / of the jail (default: none))

    --rw (Mount chroot dir (/) R/W (default: R/O))

    --env|-E PATH=/bin:/usr/bin:/usr/sbin (Additional environment variable (can be used multiple times))

    -- /root/toaster
```

> Kafel Policy used:

```GO
const kafelPolicy = `
POLICY toasters {
  KILL {
    acct,
    add_key,
    adjtimex,
    bpf,
    clock_adjtime,
    clock_settime,
    create_module,
    delete_module,
    finit_module,
    get_kernel_syms,
    get_mempolicy,
    init_module,
    io_cancel,
    io_destroy,
    io_getevents,
    io_setup,
    io_submit,
    ioperm,
    iopl,
    kcmp,
    keyctl,
    kexec_file_load,
    kexec_load,
    lookup_dcookie,
    mbind,
    migrate_pages,
    modify_ldt,
    mount,
    move_pages,
    name_to_handle_at,
    nfsservctl,
    open_by_handle_at,
    perf_event_open,
    personality,
    pivot_root,
    query_module,
    process_vm_readv,
    process_vm_writev,
    ptrace,
    quotactl,
    reboot,
    remap_file_pages,
    request_key,
    seccomp,
    set_mempolicy,
    set_thread_area,
    setns,
    settimeofday,
    syslog,
    swapon,
    swapoff,
    sysfs,
    umount,
    unshare,
    uselib,
    userfaultfd,
    vmsplice
  }
}
USE toasters DEFAULT ALLOW`
```

> Resulting nsconf:


```yaml
NSCONF:
  use_execveat = false
  exec_fd = -1
  hostname = "NSJAIL"
  cwd = "/"
  port = 0
  bindhost = "::"
  daemonize = false
  tlimit = 0
  max_cpus = 0
  keep_env = false
  keep_caps = false
  disable_no_new_privs = false
  rl_as = {X}
  rl_core = 0ULL
  rl_cpu = {hard limit}
  rl_fsize = {hard limit}
  rl_nofile = 128
  rl_nproc = 132
  rl_stack = {X}
  disable_rl = false
  personality = 0
  clone_newnet = true
  clone_newuser = true
  clone_newns = true
  clone_newpid = true
  clone_newipc = true
  clone_newuts = true
  clone_newcgroup = true
  mode = MODE_STANDALONE_ONCE
  is_root_rw = true
  is_silent = false
  stderr_to_null = false
  skip_setsid = false
  max_conns_per_ip = 0
  proc_path = "/proc"
  is_proc_rw = false
  cgroup_mem_mount = "/sys/fs/cgroup/memory"
  cgroup_mem_parent = "NSJAIL"
  cgroup_mem_max = (size_t)0
  cgroup_pids_mount = "/sys/fs/cgroup/pids"
  cgroup_pids_parent = "NSJAIL"
  cgroup_pids_max = 0U
  cgroup_net_cls_mount = "/sys/fs/cgroup/net_cls"
  cgroup_net_cls_parent = "NSJAIL"
  cgroup_net_cls_classid = 0U
  cgroup_cpu_mount = "/sys/fs/cgroup/cpu"
  cgroup_cpu_parent = "NSJAIL"
  cgroup_cpu_ms_per_sec = 0U
  cgroupv2_mount = "/sys/fs/cgroup"
  use_cgroupv2 = false
  iface_vs = "tveth1"
  iface_lo = true
  iface_vs_ip = "10.166.Y.Y"
  iface_vs_nm = "255.255.0.0"
  iface_vs_gw = "10.166.0.1"
  iface_vs_ma = ""
  orig_uid = getuid()
  orig_euid = geteuid()
  num_cpus = sysconf(_SC_NPROCESSORS_ONLN)
  seccomp_fprog.filter = NULL
  seccomp_fprog.len = 0
  seccomp_log = false
  nice_level = 19

  uids = [{inside_id: getuid(/* Get the real user ID of the calling process.  */), outside_id: getuid(), count: 1, is_newidmap: false}] // default value when not set at the command line
  gids = [{inside_id: getgid(/* Get the real group ID of the calling process.  */), outside_id: getgid(), count: 1, is_newidmap: false}] // default value when not set at the command line

  chroot = "{overlaydir.MountPoint}"
  cwd = "/root"

  openfds = std::vector<int> [STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO]

  envs = std::vector<std::string> ["PATH=/bin:/usr/bin:/usr/sbin"]

  argv = std::vector<std::string> ["/root/toaster"]

  exec_file = "/root/toaster" (argv[0] if not provided and argv not empty)

    mountpts:
      src = "{overlaydir.MountPoint}"
      dst = "/"
      fs_type = ""
      options = ""
      flags = (MS_BIND | MS_REC | MS_PRIVATE)
      is_symlink = false
      is_mandatory = true
      mounted = false
      src_content = ""
      is_dir = true

      src = ""
      dst = "/proc"
      fs_type = "proc"
      options = ""
      flags = MS_RDONLY
      is_symlink = false
      is_mandatory = true
      mounted = false
      src_content = ""
      is_dir = true

```

### Fil d'ariane

- main

  - setSigHandlers
  - setTimer
  - sandbox.preparePolicy(nsconf)
    - kafel_set_input_string(nsjconf->kafel_string)
    - kafel_compile(nsjconf->seccomp_fprog)
  - nsjail.standaloneMode(nsjconf)
    - For loop: F1
      - subproc.runChild(nsjconf, STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO):

        ```C++
        flags = 0UL | CLONE_NEWNET | CLONE_NEWUSER | CLONE_NEWNS | CLONE_NEWPID | CLONE_NEWIPC | CLONE_NEWUTS | CLONE_NEWCGROUP
        ```

        ```
        if nsjcon->mode had been equal to MODE_STANDALONE_EXECVE, we would have call unshare(flags) then subprocNewproc before continuing, check out the behaviour later and see https://stackoverflow.com/questions/4856255/the-difference-between-fork-vfork-exec-and-clone
        ```

        ```C++
        flags |= SIGCHLD
        ```
        ```
        Creation d'un stream socket unix entre deux fd, child_fd et parent_fd avec le syscall socketpair
        ```

        - pid = cloneProc(flags): (see clone.md for explanations) ; pid = 0 if child, child process pid in parent pid namespace if parent.

        - `if pid == 0 { /* in child */`

          close(parent_fd)

          - subprocNewProc(nsjconf, STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO, child_fd) (see subprocNewProc.md):

          ```C++
          /* subprocNewProc should replace the current process image and so never return. Thus, if it succeeds, these two lines should never executes */
          util::writeToFd(child_fd, &kSubprocErrorChar, sizeof(kSubprocErrorChar));
          LOG_F("Launching child process failed");
          ```

        - `} /* end in child */`

        - `addProc(nsjconf, pid, STDIN_FILENO)`:

          push `{pid: pid, start: time(NULL), remote_txt: net::connToText(sock, /* remote= */ true, &p.remote_addr), pid_syscall_fd: "/proc/{pid}/syscall"}` into nsjconf->pids

        - `initParent(nsjconf, pid, parent_fd)`:

          - `net::initNsFromParent(nsjconf, pid)`

          ```C++
          if (nsjconf->use_cgroupv2) {
            if (!cgroup2::initNsFromParent(nsjconf, pid)) {
              LOG_E("Couldn't initialize cgroup 2 user namespace for pid=%d", pid);
              exit(0xff);
            }
          } else if (!cgroup::initNsFromParent(nsjconf, pid)) {
            LOG_E("Couldn't initialize cgroup user namespace for pid=%d", pid);
            exit(0xff);
          }
          ```

          - `user::initNsFromParent(nsjconf, pid)`

          - `util::writeToFd(pipefd, &kSubprocDoneChar, sizeof(kSubprocDoneChar))`





  - sandbox::closePolicy(nsjconf)