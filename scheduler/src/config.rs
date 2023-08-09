use jail::config::{IDMapT, JailConf, MountT};

use seccomp::{allow_syscall, BpfProgram, SeccompAction, SeccompFilter};

use std::convert::TryInto;
use std::ffi::{CStr, CString};

const TOAST1: &[u8] = "toast1\0".as_bytes();
const NMCIDR16: &[u8] = "255.255.0.0\0".as_bytes();
const EMPTY: &[u8] = "\0".as_bytes();

pub fn create_toaster_pool_jconf<'a>(
    local_cloud_provider: &str,
    root_dir: CString,
    num_cpus: i64,
    ip: CString,
    gw: &'a CStr,
    cwd: String,
    mount_slave: bool,
    admin: bool,
) -> JailConf<'a> {
    let mut jconf = JailConf::new_from_root(
        root_dir,
        unsafe { CStr::from_bytes_with_nul_unchecked(TOAST1) },
        ip,
        unsafe { CStr::from_bytes_with_nul_unchecked(NMCIDR16) },
        gw,
        unsafe { CStr::from_bytes_with_nul_unchecked(EMPTY) },
    );

    jconf.debug = false;

    jconf.mnt_ms_slave = mount_slave;

    jconf.clone_newcgroup = true;
    jconf.clone_newipc = true;
    jconf.clone_newnet = true;
    jconf.clone_newns = true;
    jconf.clone_newpid = true;
    jconf.clone_newuser = true;
    jconf.clone_newuts = true;

    jconf.create_pooled_thread = true;

    jconf.hostname = "toastate";

    jconf.is_root_rw = true;

    jconf.disable_rl = true; // with params below, causes problems with non admin image
    jconf.rl_as = 100 * 1024 * 1024;
    jconf.rl_core = 100 * 1024 * 1024;
    jconf.rl_cpu = 2;
    jconf.rl_fsize = 100 * 1024 * 1024;
    jconf.rl_nofile = 1500;
    jconf.rl_nproc = 150;
    jconf.rl_stack = 100 * 1024 * 1024;

    jconf.cwd = cwd;
    jconf.num_cpus = num_cpus;

    jconf.iface_lo = true;

    // jconf.seccomp_filter = Some(toastate_default_filter());
    jconf.uids.push(IDMapT {
        inside_id: 1,
        outside_id: 1,
        count: 999,
        is_newidmap: false,
    });
    jconf.gids.push(IDMapT {
        inside_id: 1,
        outside_id: 1,
        count: 999,
        is_newidmap: false,
    });

    jconf.uids.push(IDMapT {
        inside_id: 0,
        outside_id: 1000,
        count: 1,
        is_newidmap: false,
    });
    jconf.gids.push(IDMapT {
        inside_id: 0,
        outside_id: 1000,
        count: 1,
        is_newidmap: false,
    });

    jconf.uids.push(IDMapT {
        inside_id: 1001,
        outside_id: 1001,
        count: 80000,
        is_newidmap: false,
    });
    jconf.gids.push(IDMapT {
        inside_id: 1001,
        outside_id: 1001,
        count: 80000,
        is_newidmap: false,
    });

    jconf.user_inside_uid = 0;
    jconf.user_inside_gid = 0;

    jconf.with_default_mounts();

    // Needed for pseudo-terminals opened using ptmx device to work, if it takes too much time maybe put it as an option in toaster settings
    // secure because https://lwn.net/Articles/689539/
    jconf.mountpts.push(MountT {
        src: None,
        dst: MountT::transform_dst(&jconf, "dev/pts"), // empty string means /, see mnt.rs
        dst_in_pivot: CString::new("/dev/pts").unwrap(),
        fs_type: Some(CString::new("devpts").unwrap()),
        options: Some(CString::new("gid=4,mode=620").unwrap()),
        flags: 0,
        is_dir: true,
        is_mandatory: true,
        is_symlink: false,
        mounted: false,
    });

    jconf
}

pub fn set_jconf_as_join(jconf: &mut JailConf) {
    jconf.create_pooled_thread = false;
    jconf.join_sleeping_thread = true;
}

pub fn create_toaster_jconf<'a>(
    local_cloud_provider: &str,
    root_dir: CString,
    num_cpus: i64,
    ip: CString,
    gw: &'a CStr,
    cwd: String,
    mount_slave: bool,
    admin: bool,
) -> JailConf<'a> {
    let mut jconf = JailConf::new_from_root(
        root_dir,
        unsafe { CStr::from_bytes_with_nul_unchecked(TOAST1) },
        ip,
        unsafe { CStr::from_bytes_with_nul_unchecked(NMCIDR16) },
        gw,
        unsafe { CStr::from_bytes_with_nul_unchecked(EMPTY) },
    );

    jconf.debug = false;

    jconf.mnt_ms_slave = mount_slave;

    jconf.clone_newcgroup = true;
    jconf.clone_newipc = true;
    jconf.clone_newnet = true;
    jconf.clone_newns = true;
    jconf.clone_newpid = true;
    jconf.clone_newuser = true;
    jconf.clone_newuts = true;
    jconf.hostname = "toastate";

    jconf.is_root_rw = true;

    jconf.disable_rl = true; // Need to work cgroupv2 before, make it work and implement network traffic controller net_cls
    jconf.rl_as = 100 * 1024 * 1024;
    jconf.rl_core = 100 * 1024 * 1024;
    jconf.rl_cpu = 2;
    jconf.rl_fsize = 100 * 1024 * 1024;
    jconf.rl_nofile = 1500;
    jconf.rl_nproc = 150;
    jconf.rl_stack = 100 * 1024 * 1024;

    // jconf.cgroup_cpu_ms_per_sec = 600;

    jconf.cwd = cwd;
    jconf.num_cpus = num_cpus;

    jconf.iface_lo = true;

    // jconf.seccomp_filter = Some(toastate_default_filter());
    jconf.uids.push(IDMapT {
        inside_id: 1,
        outside_id: 1,
        count: 999,
        is_newidmap: false,
    });
    jconf.gids.push(IDMapT {
        inside_id: 1,
        outside_id: 1,
        count: 999,
        is_newidmap: false,
    });

    jconf.uids.push(IDMapT {
        inside_id: 0,
        outside_id: 1000,
        count: 1,
        is_newidmap: false,
    });
    jconf.gids.push(IDMapT {
        inside_id: 0,
        outside_id: 1000,
        count: 1,
        is_newidmap: false,
    });

    jconf.uids.push(IDMapT {
        inside_id: 1001,
        outside_id: 1001,
        count: 80000,
        is_newidmap: false,
    });
    jconf.gids.push(IDMapT {
        inside_id: 1001,
        outside_id: 1001,
        count: 80000,
        is_newidmap: false,
    });

    jconf.user_inside_uid = 0;
    jconf.user_inside_gid = 0;

    jconf.with_default_mounts();

    // Needed for pseudo-terminals opened using ptmx device to work, if it takes too much time maybe put it as an option in toaster settings
    // secure because https://lwn.net/Articles/689539/
    jconf.mountpts.push(MountT {
        src: None,
        dst: MountT::transform_dst(&jconf, "dev/pts"), // empty string means /, see mnt.rs
        dst_in_pivot: CString::new("/dev/pts").unwrap(),
        fs_type: Some(CString::new("devpts").unwrap()),
        options: Some(CString::new("gid=4,mode=620").unwrap()),
        flags: 0,
        is_dir: true,
        is_mandatory: true,
        is_symlink: false,
        mounted: false,
    });

    jconf
}

// pub fn toastate_default_filter() -> BpfProgram {
//     SeccompFilter::new(
//         vec![
//             allow_syscall(libc::SYS_rt_sigprocmask),
//             allow_syscall(libc::SYS_rt_sigaction),
//             allow_syscall(libc::SYS_execve),
//             #[cfg(target_arch = "x86_64")]
//             allow_syscall(libc::SYS_mmap),
//             #[cfg(target_arch = "aarch64")]
//             // See this issue for why we are doing it this way on arch64:
//             // https://github.com/rust-lang/libc/issues/1348.
//             allow_syscall(SYS_mmap),
//             #[cfg(target_arch = "x86_64")]
//             allow_syscall(libc::SYS_arch_prctl),
//             allow_syscall(libc::SYS_set_tid_address),
//             #[cfg(target_arch = "x86_64")]
//             allow_syscall(libc::SYS_readlink),
//             #[cfg(target_arch = "x86_64")]
//             allow_syscall(libc::SYS_open),
//             allow_syscall(libc::SYS_read),
//             allow_syscall(libc::SYS_close),
//             allow_syscall(libc::SYS_brk),
//             allow_syscall(libc::SYS_sched_getaffinity),
//         ]
//         .into_iter()
//         .collect(),
//         SeccompAction::Trap,
//     )
//     .unwrap()
//     .try_into()
//     .unwrap()
// }

/*

pub fn create_image_jconf<'a>(
    local_cloud_provider: &str,
    root_dir: CString,
    uts_nm: i32,
    num_cpus: i64,
    admin: bool,
    ip: CString,
    gw: &'a CStr,
) -> Result<JailConf<'a>> {
    let mut jconf = JailConf::new_from_root(
        root_dir,
        unsafe { CStr::from_bytes_with_nul_unchecked(TOAST1) },
        ip,
        unsafe { CStr::from_bytes_with_nul_unchecked(NMCIDR16) },
        gw,
        unsafe { CStr::from_bytes_with_nul_unchecked(EMPTY) },
    );

    jconf.debug = true;

    jconf.clone_newcgroup = true;
    jconf.clone_newipc = true;
    jconf.clone_newnet = true;
    jconf.clone_newns = true;
    jconf.clone_newpid = true;
    jconf.clone_newuser = true;

    jconf.clone_newuts = true;
    jconf.hostname = "toastate";

    jconf.is_root_rw = true;
    jconf.execute = true;

    jconf.disable_rl = true; // with params below, causes problems with non admin image
    jconf.rl_as = 100 * 1024 * 1024;
    jconf.rl_core = 100 * 1024 * 1024;
    jconf.rl_cpu = 2;
    jconf.rl_fsize = 100 * 1024 * 1024;
    jconf.rl_nofile = 1500;
    jconf.rl_nproc = 150;
    jconf.rl_stack = 100 * 1024 * 1024;

    // does not work inside toastnet yet, see how to make cgroup work inside a toastnet emulated server
    // jconf.cgroup_cpu_ms_per_sec = 600;
    // jconf.use_cgroupv2 = false;

    jconf.cwd = String::from("/"); // todo: let gtvs specify that
    jconf.num_cpus = num_cpus;

    jconf.iface_lo = true;

    if admin {
        jconf.keep_caps = true;
        jconf.clone_newuser = false;

        jconf.with_default_mounts();
    } else {
        // jconf.seccomp_filter = Some(toastate_default_filter());
        jconf.uids.push(IDMapT {
            inside_id: 1,
            outside_id: 1,
            count: 999,
            is_newidmap: false,
        });
        jconf.gids.push(IDMapT {
            inside_id: 1,
            outside_id: 1,
            count: 999,
            is_newidmap: false,
        });

        jconf.uids.push(IDMapT {
            inside_id: 0,
            outside_id: 1000,
            count: 1,
            is_newidmap: false,
        });
        jconf.gids.push(IDMapT {
            inside_id: 0,
            outside_id: 1000,
            count: 1,
            is_newidmap: false,
        });

        jconf.uids.push(IDMapT {
            inside_id: 1001,
            outside_id: 1001,
            count: 80000,
            is_newidmap: false,
        });
        jconf.gids.push(IDMapT {
            inside_id: 1001,
            outside_id: 1001,
            count: 80000,
            is_newidmap: false,
        });

        jconf.user_inside_uid = 0;
        jconf.user_inside_gid = 0;

        jconf.with_default_mounts();

        // Needed for pseudo-terminals opened using ptmx device to work, if it takes too much time maybe put it as an option in toaster settings
        // secure because https://lwn.net/Articles/689539/
        jconf.mountpts.push(MountT {
            src: None,
            dst: MountT::transform_dst(&jconf, "dev/pts"), // empty string means /, see mnt.rs
            dst_in_pivot: CString::new("/dev/pts").unwrap(),
            fs_type: Some(CString::new("devpts").unwrap()),
            options: Some(CString::new("gid=4,mode=620").unwrap()),
            flags: 0,
            is_dir: true,
            is_mandatory: true,
            is_symlink: false,
            mounted: false,
        });
    }

    Ok(jconf)
}

*/
