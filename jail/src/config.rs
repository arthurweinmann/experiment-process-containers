use std::ffi::{CStr, CString};
use std::time::SystemTime;

use seccomp::BpfProgram;

use super::error::Result;
use super::rlimit;

pub const NSSIGS: [libc::c_int; 8] = [
    libc::SIGINT,
    libc::SIGQUIT,
    libc::SIGUSR1,
    libc::SIGALRM,
    libc::SIGCHLD,
    libc::SIGTERM,
    libc::SIGTTIN,
    libc::SIGTTOU,
];

pub static mut PID: u32 = 0;

// TODO: check that using the same pivot folder for multiple jail is ok
pub static mut PIVOT_FOLDER_STRING_NUL_TERMINATED: String = String::new();
pub static mut PIVOT_FOLDER: &[u8] = b"\0";

static SLASH: &'static [u8] = b"/";

pub unsafe fn init_statics() {
    PID = std::process::id();
    PIVOT_FOLDER_STRING_NUL_TERMINATED =
        format!("/run/user/{}/toastainer/{}/root\0", libc::getuid(), PID);
    PIVOT_FOLDER = PIVOT_FOLDER_STRING_NUL_TERMINATED.as_bytes();
}

// uncomment and use in JailConf::new_from_root when stable
// pub static CONST_C_STR_PIVOT_ROOT_FOLDER: &'static CStr = unsafe{CStr::from_bytes_with_nul_unchecked(b"/run/user/0/toastainer/root\0")};

/**
 * // jcong.with_uid().with_gid.etc # use theses intermediary function to set your jail conf
 * jcong.instantiate
 */
#[derive(Clone, Debug)]
pub struct JailConf<'a> {
    pub debug: bool,
    /*
     * https://unix.stackexchange.com/questions/162900/what-is-this-folder-run-user-1000
     * /run/user/$uid is created by pam_systemd and used for storing files used by running processes for that user.
     * This directory is local to the system and only accessible by the target user.
     * It also keeps things nice and organized. When a user logs out, and no active sessions remain,
     * pam_systemd will wipe the /run/user/$uid directory out.
     *
     *
     *
     * if does not work, try one of: /run/user/toastainer.{}.root ; /tmp/toastainer.{}.root ; {getenv("TMPDIR")}/toastainer.{}.root ;
     * /dev/shm/toastainer.{}.root ; /tmp/toastainer.{}.root.{random number}
     *
     * Pivot dir must be created at start with init_mount_folder
     */
    pub exec_fd: i32, // i32 instead of u32 so we can set default value to -1 and there will be no risk of collision or misinterpretation
    pub hostname: &'a str,
    pub cwd: String, // Directory in the namespace the process will run (default: '/')
    pub chroot: CString, // Directory containing / of the jail (default: none)
    pub port: u32,
    pub bind_host: &'a str,
    pub daemonize: bool,
    pub tlimit: u64,
    pub max_cpus: u16,
    pub keep_env: bool,
    pub keep_caps: bool,
    pub rl_as: u64,
    pub rl_core: u64,
    pub rl_cpu: u64,
    pub rl_fsize: u64,
    pub rl_nofile: u64,
    pub rl_nproc: u64,
    pub rl_stack: u64,
    pub disable_rl: bool,
    pub personality: u64,

    pub clone_newnet: bool,
    pub clone_newuser: bool,
    pub clone_newns: bool,
    pub clone_newpid: bool,
    pub clone_newipc: bool,
    pub clone_newuts: bool,
    pub clone_newcgroup: bool,

    pub create_pooled_thread: bool,
    pub join_sleeping_thread: bool, // if true, restart sleeping thread with passed_admin_parent_fd

    pub mnt_ms_slave: bool,

    pub prepare_env_in_child: bool,
    pub handle_fds_in_child: bool,
    pub use_exec_caveat: bool,

    pub is_root_rw: bool, // Mount chroot dir (/) R/W (default: R/O)
    pub is_silent: bool,
    pub stderr_to_null: bool,
    pub skip_setsid: bool, // Don't call setsid(), allows for terminal signal handling in the sandboxed process. Dangerous
    pub max_conns_per_ip: u32,
    pub proc_path: String,
    pub is_proc_rw: bool,

    pub disable_no_new_privs: bool, // setting this to true is dangerous

    pub iface_lo: bool,
    pub ifaces: Vec<&'a CStr>, // existing network interfaces you want to move inside the new NET namespace
    pub iface_vs: &'a CStr, // Interface which will be cloned (MACVLAN) and put inside the subprocess' namespace as 'vs'
    pub iface_vs_ip: CString, // IP of the 'vs' interface (e.g. \"192.168.0.1\")
    pub iface_vs_nm: &'a CStr, // Netmask of the 'vs' interface (e.g. \"255.255.255.0\")
    pub iface_vs_gw: &'a CStr, // Default GW for the 'vs' interface (e.g. \"192.168.0.1\")
    pub iface_vs_ma: &'a CStr, // MAC-address of the 'vs' interface (e.g. \"ba:ad:ba:be:45:00\")

    pub multi_net: Option<MultiNetConfig>, // if defined, previous iface properties are ignored

    pub cgroup_mem_mount: &'a str,
    pub cgroup_mem_parent: &'a str,
    pub cgroup_mem_max: u64,
    pub cgroup_pids_mount: &'a str,
    pub cgroup_pids_parent: &'a str,
    pub cgroup_pids_max: u64,
    pub cgroup_net_cls_mount: &'a str,
    pub cgroup_net_cls_parent: &'a str,
    pub cgroup_net_cls_classid: u64,
    pub cgroup_cpu_mount: &'a str,
    pub cgroup_cpu_parent: &'a str,
    pub cgroup_cpu_ms_per_sec: u64,
    pub cgroupv2_mount: &'a str,
    pub use_cgroupv2: bool,

    pub seccomp_log: bool,
    pub nice_level: i64,
    pub num_cpus: i64,
    pub orig_uid: libc::uid_t,
    pub orig_euid: libc::uid_t,
    pub mountpts: Vec<MountT>,
    pub pids: Vec<PIDT<'a>>,

    pub uids: Vec<IDMapT>,
    pub gids: Vec<IDMapT>,
    pub user_inside_uid: i32,
    pub user_inside_gid: i32,

    pub openfds: Vec<i32>,
    pub caps: Vec<i64>,

    pub seccomp_filter: Option<BpfProgram>,

    pub fd_in: libc::c_int,
    pub fd_out: libc::c_int,
    pub fd_err: libc::c_int,

    pub env: Option<Vec<CString>>,
    pub exec_file: Option<CString>,
    pub argv: Option<Vec<CString>>,

    pub passed_admin_child_fd: libc::c_int, // used for comm between parent and child, if set, do not forget to close after use
    pub passed_admin_parent_fd: libc::c_int, // so the child is able to close its copy of parent_fd, does he really need to do that ? investigate

    pub child_pid: Option<libc::c_int>,
    pub child_pidfd: Option<libc::c_int>,
}

// We need to implement Default trait for struct JailConf because rust won't allow empty struct or fields. In rust we cannot init a struct without giving values for any of the fields
// to init an empty (default) jailconf struct, do let p1 = JailConf::default();
impl<'a> Default for JailConf<'a> {
    fn default() -> JailConf<'a> {
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };

        let mut jconf = JailConf {
            debug: false,
            hostname: "toastate",

            exec_fd: -1,

            use_exec_caveat: false,

            cwd: String::from("/"),
            chroot: CString::default(),
            port: 0,
            bind_host: "::",
            daemonize: false,
            tlimit: 0,
            personality: 0,

            max_cpus: 0,
            num_cpus: unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) },

            disable_rl: true,
            rl_as: 0,
            rl_core: 0,
            rl_cpu: 0,
            rl_fsize: 0,
            rl_nofile: 0,
            rl_nproc: 0,
            rl_stack: 0,

            clone_newnet: false,
            clone_newuser: false,
            clone_newns: false,
            clone_newpid: false,
            clone_newipc: false,
            clone_newuts: false,
            clone_newcgroup: false,
            prepare_env_in_child: false,
            handle_fds_in_child: false,

            mnt_ms_slave: false,

            keep_env: false,
            keep_caps: false,

            disable_no_new_privs: false,

            create_pooled_thread: false,
            join_sleeping_thread: false,

            is_root_rw: false,
            is_silent: false,
            stderr_to_null: false,
            skip_setsid: false,
            max_conns_per_ip: 0,
            proc_path: String::from("proc"), // no leading / means /proc, see mnt.rs
            is_proc_rw: false,

            iface_lo: true,
            iface_vs: CStr::from_bytes_with_nul("\0".as_bytes()).unwrap(),
            iface_vs_ip: CString::new("0.0.0.0").unwrap(),
            iface_vs_nm: CStr::from_bytes_with_nul("255.255.255.0\0".as_bytes()).unwrap(),
            iface_vs_gw: CStr::from_bytes_with_nul("0.0.0.0\0".as_bytes()).unwrap(),
            iface_vs_ma: CStr::from_bytes_with_nul("\0".as_bytes()).unwrap(),
            ifaces: vec![],
            multi_net: None,

            cgroup_mem_mount: "/sys/fs/cgroup/memory",
            cgroup_mem_parent: "TOASTAINER",
            cgroup_mem_max: 0,
            cgroup_pids_mount: "/sys/fs/cgroup/pids",
            cgroup_pids_parent: "TOASTAINER",
            cgroup_pids_max: 0,
            cgroup_net_cls_mount: "/sys/fs/cgroup/net_cls",
            cgroup_net_cls_parent: "TOASTAINER",
            cgroup_net_cls_classid: 0,
            cgroup_cpu_mount: "/sys/fs/cgroup/cpu",
            cgroup_cpu_parent: "TOASTAINER",
            cgroup_cpu_ms_per_sec: 0,
            cgroupv2_mount: "/sys/fs/cgroup",
            use_cgroupv2: false,

            seccomp_log: false,
            nice_level: 19,
            orig_uid: uid,
            orig_euid: unsafe { libc::geteuid() },
            mountpts: vec![],
            pids: vec![],
            uids: vec![],
            gids: vec![],
            user_inside_gid: -1,
            user_inside_uid: -1,
            // envs: vec![],
            openfds: vec![libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO],
            caps: vec![],

            env: None,
            exec_file: None,
            argv: None,

            seccomp_filter: None,
            fd_in: libc::STDIN_FILENO,
            fd_out: libc::STDOUT_FILENO,
            fd_err: libc::STDERR_FILENO,

            passed_admin_child_fd: -1,
            passed_admin_parent_fd: -1,

            child_pid: None,
            child_pidfd: None,
        };
        jconf
            .with_uid(uid, uid, 1, false)
            .with_gid(gid, gid, 1, false)
            .with_default_mounts();

        jconf.security_checks();

        jconf
    }
}

impl<'a> JailConf<'a> {
    pub fn instantiate(&mut self) {
        self.orig_euid = unsafe { libc::geteuid() }; // user::init_ns_from_child is triggered if orig_euid == 0, so root user is not the same as the one outside the namespace
        self.num_cpus = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };

        if self.uids.is_empty() {
            self.uids.push(IDMapT {
                inside_id: self.orig_uid,
                outside_id: self.orig_uid,
                count: 1,
                is_newidmap: false,
            });
        }
        if self.gids.is_empty() {
            let gid = unsafe { libc::getgid() };
            self.gids.push(IDMapT {
                inside_id: gid,
                outside_id: gid,
                count: 1,
                is_newidmap: false,
            });
        }
        self.security_checks();
    }

    pub fn borrow_env(&'a self) -> Option<&'a [CString]> {
        match self.env {
            Some(ref v) => Some(v),
            None => None,
        }
    }

    pub fn borrow_exec_file(&'a self) -> Option<&'a CString> {
        match self.exec_file {
            Some(ref v) => Some(v),
            None => None,
        }
    }

    pub fn borrow_argv(&'a self) -> Option<&'a [CString]> {
        match self.argv {
            Some(ref v) => Some(v),
            None => None,
        }
    }

    pub fn disable_proc(&mut self) -> &mut Self {
        self.proc_path = String::new();
        self
    }

    pub fn with_chroot(&mut self, c: &str) -> &mut Self {
        self.chroot = unsafe { CString::from_vec_unchecked(c.as_bytes().to_vec()) };
        self
    }

    pub fn chroot_is_rw(&mut self) -> &mut Self {
        self.is_root_rw = true;
        self
    }

    pub fn with_hostname(&mut self, name: &'a str) -> &mut Self {
        self.hostname = name;
        self
    }

    pub fn with_cwd(&mut self, dir: String) -> &mut Self {
        self.cwd = dir;
        self
    }

    pub fn with_env(&mut self, env: Vec<CString>) -> &mut Self {
        self.env = Some(env);
        self
    }

    pub fn with_mnt(&mut self, mnt: MountT) -> &mut Self {
        self.mountpts.push(mnt);
        self
    }

    pub fn with_exevc(&mut self, exec_file: CString, argv: Vec<CString>) -> &mut Self {
        self.exec_file = Some(exec_file);
        self.argv = Some(argv);
        self
    }

    pub fn with_net_config(
        &mut self,
        iface_lo: bool,
        iface_vs: &'a CStr,
        iface_vs_ip: CString,
        iface_vs_nm: &'a CStr,
        iface_vs_gw: &'a CStr,
        iface_vs_ma: &'a CStr,
    ) -> &mut Self {
        self.iface_lo = iface_lo;
        self.iface_vs = iface_vs;
        self.iface_vs_ip = iface_vs_ip;
        self.iface_vs_nm = iface_vs_nm;
        self.iface_vs_gw = iface_vs_gw;
        self.iface_vs_ma = iface_vs_ma;
        self
    }

    pub fn clone_newnet(&mut self) -> &mut Self {
        self.clone_newnet = true;
        self
    }

    pub fn clone_newuser(&mut self) -> &mut Self {
        self.clone_newuser = true;
        self
    }

    pub fn clone_newns(&mut self) -> &mut Self {
        self.clone_newns = true;
        self
    }

    pub fn clone_newpid(&mut self) -> &mut Self {
        self.clone_newpid = true;
        self
    }

    pub fn clone_newipc(&mut self) -> &mut Self {
        self.clone_newipc = true;
        self
    }

    pub fn clone_newuts(&mut self) -> &mut Self {
        self.clone_newuts = true;
        self
    }

    pub fn clone_newcgroup(&mut self) -> &mut Self {
        self.clone_newcgroup = true;
        self
    }

    pub fn prepare_env_in_child(&mut self) -> &mut Self {
        self.prepare_env_in_child = true;
        self
    }

    pub fn handle_fds_in_child(&mut self) -> &mut Self {
        self.handle_fds_in_child = true;
        self
    }

    pub fn keep_env(&mut self) -> &mut Self {
        self.keep_env = true;
        self
    }

    pub fn keep_caps(&mut self) -> &mut Self {
        self.keep_caps = true;
        self
    }

    pub fn with_rlimit_as(
        &mut self,
        hard: bool,
        soft: bool,
        infinity: bool,
        val_mb: u64,
    ) -> Result<&mut Self> {
        if infinity {
            self.rl_as = rlimit::RLIM64_INFINITY;
            return Ok(self);
        }
        let cur = rlimit::get_rlimit64(libc::RLIMIT_AS)?;
        if soft {
            self.rl_as = cur.rlim_cur;
        } else if hard {
            self.rl_as = cur.rlim_max;
        } else {
            self.rl_as = val_mb * (1024 * 1024);
        }
        Ok(self)
    }

    pub fn with_rlimit_core(
        &mut self,
        hard: bool,
        soft: bool,
        infinity: bool,
        val_mb: u64,
    ) -> Result<&mut Self> {
        if infinity {
            self.rl_core = rlimit::RLIM64_INFINITY;
            return Ok(self);
        }
        let cur = rlimit::get_rlimit64(libc::RLIMIT_CORE)?;
        if soft {
            self.rl_core = cur.rlim_cur;
        } else if hard {
            self.rl_core = cur.rlim_max;
        } else {
            self.rl_core = val_mb * (1024 * 1024);
        }
        Ok(self)
    }

    pub fn with_rlimit_cpu(
        &mut self,
        hard: bool,
        soft: bool,
        infinity: bool,
        val: u64,
    ) -> Result<&mut Self> {
        if infinity {
            self.rl_cpu = rlimit::RLIM64_INFINITY;
            return Ok(self);
        }
        let cur = rlimit::get_rlimit64(libc::RLIMIT_CPU)?;
        if soft {
            self.rl_cpu = cur.rlim_cur;
        } else if hard {
            self.rl_cpu = cur.rlim_max;
        } else {
            self.rl_cpu = val;
        }
        Ok(self)
    }

    pub fn with_rlimit_fsize(
        &mut self,
        hard: bool,
        soft: bool,
        infinity: bool,
        val_mb: u64,
    ) -> Result<&mut Self> {
        if infinity {
            self.rl_fsize = rlimit::RLIM64_INFINITY;
            return Ok(self);
        }
        let cur = rlimit::get_rlimit64(libc::RLIMIT_FSIZE)?;
        if soft {
            self.rl_fsize = cur.rlim_cur;
        } else if hard {
            self.rl_fsize = cur.rlim_max;
        } else {
            self.rl_fsize = val_mb * (1024 * 1024);
        }
        Ok(self)
    }

    pub fn with_rlimit_nofile(
        &mut self,
        hard: bool,
        soft: bool,
        infinity: bool,
        val: u64,
    ) -> Result<&mut Self> {
        if infinity {
            self.rl_nofile = rlimit::RLIM64_INFINITY;
            return Ok(self);
        }
        let cur = rlimit::get_rlimit64(libc::RLIMIT_NOFILE)?;
        if soft {
            self.rl_nofile = cur.rlim_cur;
        } else if hard {
            self.rl_nofile = cur.rlim_max;
        } else {
            self.rl_nofile = val;
        }
        Ok(self)
    }

    pub fn with_rlimit_nproc(
        &mut self,
        hard: bool,
        soft: bool,
        infinity: bool,
        val: u64,
    ) -> Result<&mut Self> {
        if infinity {
            self.rl_nproc = rlimit::RLIM64_INFINITY;
            return Ok(self);
        }
        let cur = rlimit::get_rlimit64(libc::RLIMIT_NPROC)?;
        if soft {
            self.rl_nproc = cur.rlim_cur;
        } else if hard {
            self.rl_nproc = cur.rlim_max;
        } else {
            self.rl_nproc = val;
        }
        Ok(self)
    }

    pub fn with_rlimit_stack(
        &mut self,
        hard: bool,
        soft: bool,
        infinity: bool,
        val_mb: u64,
    ) -> Result<&mut Self> {
        if infinity {
            self.rl_stack = rlimit::RLIM64_INFINITY;
            return Ok(self);
        }
        let cur = rlimit::get_rlimit64(libc::RLIMIT_STACK)?;
        if soft {
            self.rl_stack = cur.rlim_cur;
        } else if hard {
            self.rl_stack = cur.rlim_max;
        } else {
            self.rl_stack = val_mb * (1024 * 1024);
        }
        Ok(self)
    }

    /// do this at the end of all other jconf settings, including after pushing other mount points
    /// See pid.md for the explanation why you can mount /proc in a new pid namespace without risks
    pub fn with_default_mounts(&mut self) -> &mut Self {
        if self.chroot.as_bytes().len() > 0 {
            if self.is_root_rw {
                self.mountpts.insert(
                    0,
                    MountT {
                        src: Some(self.chroot.clone()),
                        dst: MountT::transform_dst(self, ""), // empty string means /, see mnt.rs
                        dst_in_pivot: CString::new("/").unwrap(),
                        fs_type: None,
                        options: None,
                        flags: libc::MS_BIND | libc::MS_REC | libc::MS_PRIVATE, // the chroot is a bind mount, e.g. the same contents are accessible in two places, see mnt.md
                        is_dir: true,
                        is_mandatory: true,
                        is_symlink: false,
                        mounted: false,
                    },
                );
            } else {
                self.mountpts.insert(
                    0,
                    MountT {
                        src: Some(self.chroot.clone()),
                        dst: MountT::transform_dst(self, ""), // empty string means /, see mnt.rs
                        dst_in_pivot: CString::new("/").unwrap(),
                        fs_type: None,
                        options: None,
                        flags: libc::MS_BIND | libc::MS_REC | libc::MS_PRIVATE | libc::MS_RDONLY,
                        is_dir: true,
                        is_mandatory: true,
                        is_symlink: false,
                        mounted: false,
                    },
                );
            }
        } else {
            if self.is_root_rw {
                self.mountpts.insert(
                    0,
                    MountT {
                        src: None,
                        dst: MountT::transform_dst(self, ""), // empty string means /, see mnt.rs
                        dst_in_pivot: CString::new("/").unwrap(),
                        fs_type: Some(CString::new("tmpfs").unwrap()),
                        options: None,
                        flags: 0,
                        is_dir: true,
                        is_mandatory: true,
                        is_symlink: false,
                        mounted: false,
                    },
                );
            } else {
                self.mountpts.insert(
                    0,
                    MountT {
                        src: None,
                        dst: MountT::transform_dst(self, ""), // empty string means /, see mnt.rs
                        dst_in_pivot: CString::new("/").unwrap(),
                        fs_type: Some(CString::new("tmpfs").unwrap()),
                        options: None,
                        flags: libc::MS_RDONLY,
                        is_dir: true,
                        is_mandatory: true,
                        is_symlink: false,
                        mounted: false,
                    },
                );
            }
        }

        if !self.proc_path.is_empty() {
            if self.is_root_rw {
                self.mountpts.push(MountT {
                    src: None,
                    dst: MountT::transform_dst(self, self.proc_path.as_str()),
                    dst_in_pivot: CString::new("/proc").unwrap(),
                    fs_type: Some(CString::new("proc").unwrap()),
                    options: None,
                    flags: 0,
                    is_dir: true,
                    is_mandatory: true,
                    is_symlink: false,
                    mounted: false,
                });
            } else {
                self.mountpts.push(MountT {
                    src: None,
                    dst: MountT::transform_dst(self, self.proc_path.as_str()),
                    dst_in_pivot: CString::new("/proc").unwrap(),
                    fs_type: Some(CString::new("proc").unwrap()),
                    options: None,
                    flags: libc::MS_RDONLY,
                    is_dir: true,
                    is_mandatory: true,
                    is_symlink: false,
                    mounted: false,
                });
            }
        }

        self
    }

    // in nsjail examples at https://github.com/google/nsjail, outside: current process one, inside: 99999,count: 1
    pub fn with_uid(
        &mut self,
        inside_uid: libc::uid_t,
        outside_uid: libc::uid_t,
        count: u64,
        is_newidmap: bool,
    ) -> &mut Self {
        self.uids.push(IDMapT {
            inside_id: inside_uid,
            outside_id: outside_uid,
            count: count,
            is_newidmap: is_newidmap,
        });
        self
    }

    // in nsjail examples at https://github.com/google/nsjail, outside: current process one, inside: 99999,count: 1
    pub fn with_gid(
        &mut self,
        inside_uid: libc::uid_t,
        outside_uid: libc::uid_t,
        count: u64,
        is_newidmap: bool,
    ) -> &mut Self {
        self.gids.push(IDMapT {
            inside_id: inside_uid,
            outside_id: outside_uid,
            count: count,
            is_newidmap: is_newidmap,
        });
        self
    }

    fn security_checks(&self) {
        for uid in self.uids.iter() {
            if uid.outside_id == 0 && self.clone_newuser {
                println!("WARNING: Process will be UID/EUID=0 in the global user namespace, and will have user root-level access to files")
            }
        }
        for gid in self.gids.iter() {
            if gid.outside_id == 0 && self.clone_newuser {
                println!("WARNING: Process will be GID/EGID=0 in the global user namespace, and will have group root-level access to files")
            }
        }
    }

    pub fn new() -> Self {
        let uid = unsafe { libc::getuid() };
        JailConf {
            debug: false,
            hostname: "toastate",

            exec_file: None,
            exec_fd: -1,

            use_exec_caveat: false,

            argv: None,
            cwd: String::from("/"),
            chroot: CString::default(),
            port: 0,
            bind_host: "::",
            daemonize: false,
            tlimit: 0,
            max_cpus: 0,
            personality: 0,
            disable_rl: false,

            rl_as: 0,
            rl_core: 0,
            rl_cpu: 0,
            rl_fsize: 0,
            rl_nofile: 0,
            rl_nproc: 0,
            rl_stack: 0,

            clone_newnet: false,
            clone_newuser: false,
            clone_newns: false,
            clone_newpid: false,
            clone_newipc: false,
            clone_newuts: false,
            clone_newcgroup: false,
            prepare_env_in_child: false,
            handle_fds_in_child: false,

            disable_no_new_privs: false,

            mnt_ms_slave: false,

            keep_env: false,
            keep_caps: false,

            create_pooled_thread: false,
            join_sleeping_thread: false,

            is_root_rw: false,
            is_silent: false,
            stderr_to_null: false,
            skip_setsid: false,
            max_conns_per_ip: 0,
            proc_path: String::from("proc"), // no leading / means /proc, see mnt.rs
            is_proc_rw: false,

            iface_lo: true,
            iface_vs: CStr::from_bytes_with_nul("\0".as_bytes()).unwrap(),
            iface_vs_ip: CString::new("0.0.0.0").unwrap(),
            iface_vs_nm: CStr::from_bytes_with_nul("255.255.255.0\0".as_bytes()).unwrap(),
            iface_vs_gw: CStr::from_bytes_with_nul("0.0.0.0\0".as_bytes()).unwrap(),
            iface_vs_ma: CStr::from_bytes_with_nul("\0".as_bytes()).unwrap(),
            ifaces: vec![],
            multi_net: None,

            cgroup_mem_mount: "/sys/fs/cgroup/memory",
            cgroup_mem_parent: "TOASTAINER",
            cgroup_mem_max: 0,
            cgroup_pids_mount: "/sys/fs/cgroup/pids",
            cgroup_pids_parent: "TOASTAINER",
            cgroup_pids_max: 0,
            cgroup_net_cls_mount: "/sys/fs/cgroup/net_cls",
            cgroup_net_cls_parent: "TOASTAINER",
            cgroup_net_cls_classid: 0,
            cgroup_cpu_mount: "/sys/fs/cgroup/cpu",
            cgroup_cpu_parent: "TOASTAINER",
            cgroup_cpu_ms_per_sec: 0,
            cgroupv2_mount: "/sys/fs/cgroup",
            use_cgroupv2: false,

            seccomp_log: false,
            nice_level: 19,
            num_cpus: 0,
            orig_uid: uid,
            orig_euid: 0,
            mountpts: vec![],
            pids: vec![],
            uids: vec![],
            gids: vec![],
            user_inside_gid: -1,
            user_inside_uid: -1,
            // envs: vec![],
            openfds: vec![libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO],
            caps: vec![],

            env: None,
            seccomp_filter: None,
            fd_in: libc::STDIN_FILENO,
            fd_out: libc::STDOUT_FILENO,
            fd_err: libc::STDERR_FILENO,

            passed_admin_child_fd: -1,
            passed_admin_parent_fd: -1,

            child_pid: None,
            child_pidfd: None,
        }
    }

    pub fn new_from_root(
        chroot: CString,
        iface_vs: &'a CStr,
        iface_vs_ip: CString,
        iface_vs_nm: &'a CStr,
        iface_vs_gw: &'a CStr,
        iface_vs_ma: &'a CStr,
    ) -> Self {
        JailConf {
            debug: false,
            hostname: "toastate",

            exec_file: None,
            exec_fd: -1,

            use_exec_caveat: false,

            argv: None,
            cwd: String::from("/"),
            chroot: chroot,
            port: 0,
            bind_host: "::",
            daemonize: false,
            tlimit: 0,
            max_cpus: 0,
            personality: 0,
            disable_rl: false,

            rl_as: 0,
            rl_core: 0,
            rl_cpu: 0,
            rl_fsize: 0,
            rl_nofile: 0,
            rl_nproc: 0,
            rl_stack: 0,

            disable_no_new_privs: false,

            clone_newnet: false,
            clone_newuser: false,
            clone_newns: false,
            clone_newpid: false,
            clone_newipc: false,
            clone_newuts: false,
            clone_newcgroup: false,
            prepare_env_in_child: false,
            handle_fds_in_child: false,

            mnt_ms_slave: false,

            keep_env: false,
            keep_caps: false,

            create_pooled_thread: false,
            join_sleeping_thread: false,

            is_root_rw: false,
            is_silent: false,
            stderr_to_null: false,
            skip_setsid: false,
            max_conns_per_ip: 0,
            proc_path: String::from("proc"), // no leading / means /proc, see mnt.rs
            is_proc_rw: false,

            iface_lo: true,
            iface_vs: iface_vs,
            iface_vs_ip: iface_vs_ip,
            iface_vs_nm: iface_vs_nm,
            iface_vs_gw: iface_vs_gw,
            iface_vs_ma: iface_vs_ma,
            multi_net: None,
            ifaces: vec![],

            cgroup_mem_mount: "/sys/fs/cgroup/memory",
            cgroup_mem_parent: "TOASTAINER",
            cgroup_mem_max: 0,
            cgroup_pids_mount: "/sys/fs/cgroup/pids",
            cgroup_pids_parent: "TOASTAINER",
            cgroup_pids_max: 0,
            cgroup_net_cls_mount: "/sys/fs/cgroup/net_cls",
            cgroup_net_cls_parent: "TOASTAINER",
            cgroup_net_cls_classid: 0,
            cgroup_cpu_mount: "/sys/fs/cgroup/cpu",
            cgroup_cpu_parent: "TOASTAINER",
            cgroup_cpu_ms_per_sec: 0,
            cgroupv2_mount: "/sys/fs/cgroup",
            use_cgroupv2: false,

            seccomp_log: false,
            nice_level: 19,
            num_cpus: 0,
            orig_uid: 0,
            orig_euid: 0,
            mountpts: vec![],
            pids: vec![],
            uids: vec![],
            gids: vec![],
            user_inside_gid: -1,
            user_inside_uid: -1,
            // envs: vec![],
            openfds: vec![libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO],
            caps: vec![],

            env: None,
            seccomp_filter: None,
            fd_in: libc::STDIN_FILENO,
            fd_out: libc::STDOUT_FILENO,
            fd_err: libc::STDERR_FILENO,

            passed_admin_child_fd: -1,
            passed_admin_parent_fd: -1,

            child_pid: None,
            child_pidfd: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MountT {
    pub src: Option<CString>,
    pub dst: CString,
    pub dst_in_pivot: CString,
    pub fs_type: Option<CString>,
    pub options: Option<CString>,
    pub flags: u64,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub is_mandatory: bool,
    pub mounted: bool,
}

impl MountT {
    // dst must not have a leading /
    pub fn transform_dst(jconf: &JailConf, dst: &str) -> CString {
        unsafe {
            CString::from_vec_unchecked(
                // TODO: replace with from_vec_with_nul_unchecked when stable
                [
                    &PIVOT_FOLDER[..PIVOT_FOLDER.len() - 1],
                    SLASH,
                    dst.as_bytes(),
                ]
                .concat(),
            )
        }

        // CString::new(format!(
        //     "/run/user/{}/toastainer/root/{}",
        //     jconf.orig_uid, dst
        // ))
        // .unwrap()
    }

    pub fn transform_dst_bytes(jconf: &JailConf, dst: &[u8]) -> CString {
        unsafe {
            CString::from_vec_unchecked(
                // TODO: replace with from_vec_with_nul_unchecked when stable
                [
                    &PIVOT_FOLDER[..PIVOT_FOLDER.len() - 1],
                    SLASH,
                    dst,
                ]
                .concat(),
            )
        }
    }
}

impl Default for MountT {
    fn default() -> MountT {
        MountT {
            src: None,
            dst: CString::default(),
            dst_in_pivot: CString::default(),
            fs_type: None,
            options: None,
            flags: 0,
            is_dir: false,
            is_symlink: false,
            is_mandatory: false,
            mounted: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PIDT<'a> {
    pub pid: libc::pid_t,
    pub start: SystemTime,
    pub remote_txt: &'a str,
    // we'll not implement remote socket control for now
    // struct sockaddr_in6 remote_addr;
    pub pid_syscall_fd: i32,
}

impl<'a> Default for PIDT<'a> {
    fn default() -> PIDT<'a> {
        PIDT {
            remote_txt: "",
            pid: 0,
            start: SystemTime::now(),
            pid_syscall_fd: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct IDMapT {
    pub inside_id: libc::uid_t,
    pub outside_id: libc::uid_t,
    pub count: u64,
    pub is_newidmap: bool,
}

impl Default for IDMapT {
    fn default() -> IDMapT {
        IDMapT {
            inside_id: 0,
            outside_id: 0,
            count: 0,
            is_newidmap: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MultiNetConfig {
    pub iface_vs: Vec<CString>, // Interface which will be cloned (MACVLAN) and put inside the subprocess' namespace as 'vs'
    pub iface_vs_ip: Vec<CString>, // IP of the 'vs' interface (e.g. \"192.168.0.1\")
    pub iface_vs_nm: Vec<CString>, // Netmask of the 'vs' interface (e.g. \"255.255.255.0\")
    pub iface_vs_gw: Vec<CString>, // Default GW for the 'vs' interface (e.g. \"192.168.0.1\")
    pub iface_vs_ma: Vec<CString>, // MAC-address of the 'vs' interface (e.g. \"ba:ad:ba:be:45:00\")
}
