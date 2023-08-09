use jail::config::{IDMapT, JailConf};
use jail::error::Result;
use jail::subproc;
use std::ffi::CStr;
use sys_util::errno::Errno;

// uncomment and use when stable in rust
// static ubuntu_env: &'static CStr = unsafe{CStr::from_bytes_with_nul_unchecked(b"PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\0")};

pub fn create_uts_namespace(hostname: &str) -> Result<JailConf> {
    let mut jconf = JailConf::default();
    jconf.clone_newuts().clone_newpid(); // newpid allows us to be sure that when first child terminates all other child in the jail terminates too ; see pid namespace for more information
    jconf.hostname = hostname;
    jconf.save_uts_nm = true;

    let child_pid = subproc::run_child(&mut jconf, vec![], vec![], None)?;
    if unsafe { libc::waitpid(child_pid, 0 as *mut libc::c_int, 0) } != child_pid {
        return Err(format!(
            "Error waiting the child process to finish: {}",
            Errno::last()
        )
        .into());
    }

    subproc::clean_after_child(&mut jconf, child_pid)?;

    // reset
    jconf.save_uts_nm = false;
    jconf.clone_newuts = false;
    jconf.clone_newpid = false;

    Ok(jconf)
}

/// you can provide in jconf only: num_cpus, max_cpus, disable_rl, rl_as, rl_core, rl_cpu, rl_fsize, rl_nofile, rl_nproc, rl_stack
pub fn create_cgroup(jconf: &mut JailConf) -> Result<()> {
    jconf.clone_newcgroup().clone_newpid();
    jconf.save_cgroup_nm = true;

    let child_pid = subproc::run_child(jconf, vec![], vec![], None)?;
    if unsafe { libc::waitpid(child_pid, 0 as *mut libc::c_int, 0) } != child_pid {
        return Err(format!(
            "Error waiting the child process to finish: {}",
            Errno::last()
        )
        .into());
    }

    subproc::clean_after_child(jconf, child_pid)?;

    // reset
    jconf.save_cgroup_nm = false;
    jconf.clone_newcgroup = false;
    jconf.clone_newpid = false;

    Ok(())
}

/// you can provide in jconf: iface_lo, iface_vs, iface_vs_ip, iface_vs_nm, iface_vs_gw, iface_vs_ma, ifaces
pub fn create_net(jconf: &mut JailConf) -> Result<()> {
    jconf.clone_newnet().clone_newpid();
    jconf.save_net_nm = true;

    let child_pid = subproc::run_child(jconf, vec![], vec![], None)?;
    if unsafe { libc::waitpid(child_pid, 0 as *mut libc::c_int, 0) } != child_pid {
        return Err(format!(
            "Error waiting the child process to finish: {}",
            Errno::last()
        )
        .into());
    }

    subproc::clean_after_child(jconf, child_pid)?;

    // reset
    jconf.save_net_nm = false;
    jconf.clone_newnet = false;
    jconf.clone_newpid = false;

    Ok(())
}

/// you can provide in jconf: uids, gids
pub fn create_user(jconf: &mut JailConf) -> Result<()> {
    jconf.clone_newuser().clone_newpid();
    jconf.save_user_nm = true;

    let child_pid = subproc::run_child(jconf, vec![], vec![], None)?;
    if unsafe { libc::waitpid(child_pid, 0 as *mut libc::c_int, 0) } != child_pid {
        return Err(format!(
            "Error waiting the child process to finish: {}",
            Errno::last()
        )
        .into());
    }

    subproc::clean_after_child(jconf, child_pid)?;

    // reset
    jconf.save_user_nm = false;
    jconf.clone_newuser = false;
    jconf.clone_newpid = false;

    Ok(())
}

pub fn create_ipc<'a>() -> Result<JailConf<'a>> {
    let mut jconf = JailConf::default();
    jconf.clone_newipc().clone_newpid();
    jconf.save_ipc_nm = true;

    let child_pid = subproc::run_child(&mut jconf, vec![], vec![], None)?;
    if unsafe { libc::waitpid(child_pid, 0 as *mut libc::c_int, 0) } != child_pid {
        return Err(format!(
            "Error waiting the child process to finish: {}",
            Errno::last()
        )
        .into());
    }

    subproc::clean_after_child(&mut jconf, child_pid)?;

    // reset
    jconf.save_ipc_nm = false;
    jconf.clone_newipc = false;
    jconf.clone_newpid = false;

    Ok(jconf)
}

pub fn create_mnt_namespace(root_dir: &str) -> Result<JailConf> {
    let mut jconf = JailConf::new();
    jconf.clone_newns().clone_newpid();
    jconf.save_mnt_nm = true;
    jconf
        .with_chroot(root_dir)
        .chroot_is_rw()
        .with_default_mounts();

    let child_pid = subproc::run_child(&mut jconf, vec![], vec![], None)?;
    if unsafe { libc::waitpid(child_pid, 0 as *mut libc::c_int, 0) } != child_pid {
        return Err(format!(
            "Error waiting the child process to finish: {}",
            Errno::last()
        )
        .into());
    }

    subproc::clean_after_child(&mut jconf, child_pid)?;

    // reset
    jconf.save_mnt_nm = false;
    jconf.clone_newns = false;
    jconf.clone_newpid = false;

    Ok(jconf)
}
