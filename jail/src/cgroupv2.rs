/*

Better than cgroupv1, but need to:
    - add net_cls
    - make it pool compatible

*/


use super::config::JailConf;
use super::utils::{get_folder_from_pid, mkdir, write_buf_to_file};
use utils::filepath::path_to_bytes;

use std::ffi::CString;
use sys_util::SyscallReturnCode;

use std::path::{Path, PathBuf};
use std::str::FromStr;

impl<'a> JailConf<'a> {
    pub fn get_cgroup_v2_path(&self, pid: &str) -> PathBuf {
        let mut cgroup_path = PathBuf::from_str(self.cgroupv2_mount).unwrap();
        cgroup_path.push(get_folder_from_pid(pid));
        cgroup_path
    }
}

pub fn create_cgroup<T: AsRef<Path>>(path: &T) -> Result<(), std::io::Error> {
    // pid: libc::pid_t
    match mkdir(path, 0o700 as libc::mode_t) {
        // checker si le cgroup est bien créé avec le mode 0700
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(e),
        },
    }
}

pub fn write_to_cgroup(
    cgroup_path: &mut PathBuf,
    resource: &str,
    value: &str,
) -> Result<(), std::io::Error> {
    cgroup_path.push(resource);
    write_buf_to_file(cgroup_path, libc::O_WRONLY, value.as_bytes())?;
    cgroup_path.pop();
    Ok(())
}

pub fn add_pid_to_proc_list(cgroup_path: &mut PathBuf, pid: &str) -> Result<(), std::io::Error> {
    cgroup_path.push("cgroup.procs");
    write_buf_to_file(cgroup_path, libc::O_WRONLY, pid.as_bytes())?;
    cgroup_path.pop();
    Ok(())
}

pub fn remove_cgroup<T: AsRef<Path>>(cgroup_path: &T) -> Result<(), std::io::Error> {
    // let folder_name = CStr::from_bytes_with_nul(path_to_bytes(cgroup_path.as_ref())).unwrap();

    let folder_name = CString::new(path_to_bytes(cgroup_path.as_ref())).unwrap(); // we use CString::new to create a new string terminated by a null byte \0
    SyscallReturnCode(unsafe { libc::rmdir(folder_name.as_ptr()) }).into_empty_result()
}

pub fn init_ns_from_parent_mem(jconf: &JailConf, pid: &str) -> Result<(), std::io::Error> {
    if jconf.cgroup_mem_max == 0 {
        return Ok(());
    }

    let mut cgroup_path = jconf.get_cgroup_v2_path(pid);

    create_cgroup(&cgroup_path)?;
    add_pid_to_proc_list(&mut cgroup_path, pid)?;
    write_to_cgroup(
        &mut cgroup_path,
        "memory.max",
        &jconf.cgroup_mem_max.to_string(),
    )
}

pub fn init_ns_from_parent_pids(jconf: &JailConf, pid: &str) -> Result<(), std::io::Error> {
    if jconf.cgroup_pids_max == 0 {
        return Ok(());
    }

    let mut cgroup_path = jconf.get_cgroup_v2_path(pid);

    create_cgroup(&cgroup_path)?;
    add_pid_to_proc_list(&mut cgroup_path, pid)?;
    write_to_cgroup(
        &mut cgroup_path,
        "pids.max",
        &jconf.cgroup_pids_max.to_string(),
    )
}

pub fn init_ns_from_parent_cpu(jconf: &JailConf, pid: &str) -> Result<(), std::io::Error> {
    if jconf.cgroup_cpu_ms_per_sec == 0 {
        return Ok(());
    }

    let mut cgroup_path = jconf.get_cgroup_v2_path(pid);

    create_cgroup(&cgroup_path)?;
    add_pid_to_proc_list(&mut cgroup_path, pid)?;

    // The maximum bandwidth limit in the format: `$MAX $PERIOD`.
    // This indicates that the group may consume up to $MAX in each $PERIOD
    // duration.
    let val = format!("{} 1000000", jconf.cgroup_cpu_ms_per_sec * 1000);

    write_to_cgroup(&mut cgroup_path, "cpu.max", &val)
}

pub fn init_ns_from_parent(jconf: &JailConf, pid: &str) -> Result<(), std::io::Error> {
    init_ns_from_parent_mem(jconf, pid)?;
    init_ns_from_parent_pids(jconf, pid)?;
    init_ns_from_parent_cpu(jconf, pid)
}

pub fn finish_from_parent(jconf: &JailConf, pid: &str) -> Result<(), std::io::Error> {
    if jconf.cgroup_cpu_ms_per_sec != 0 || jconf.cgroup_pids_max != 0 || jconf.cgroup_mem_max != 0 {
        remove_cgroup(&jconf.get_cgroup_v2_path(pid))?;
    }

    Ok(())
}
