use super::config::JailConf;
use super::error::Result;
use super::utils::{get_folder_from_pid, mkdir, write_buf_to_file};

use std::ffi::{CStr, CString};
use sys_util::sched::setns;
use sys_util::unistd::mkdir_ignore_eexist_cstr;
use sys_util::SyscallReturnCode; // OsStr

use std::path::{Path, PathBuf};
use std::str::FromStr;
use utils::filepath::path_to_bytes;

pub fn create_cgroup<T: AsRef<Path>>(path: &T) -> Result<()> {
    // pid: libc::pid_t
    match mkdir(path, 0o700 as libc::mode_t) {
        // checker si le cgroup est bien créé avec le mode 0700
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(format!("could not create_cgroup {:?}: {}", path.as_ref(), e).into()),
        },
    }
}

pub fn create_cgroup_cstr(path: &CStr) -> Result<()> {
    match mkdir_ignore_eexist_cstr(path, 0o700 as libc::mode_t) {
        // checker si le cgroup est bien créé avec le mode 0700
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not create_cgroup {:?}: {}", path.as_ref(), e).into()),
    }
}

pub fn write_to_cgroup<T: AsRef<Path>>(path: &T, value: &[u8]) -> Result<()> {
    write_buf_to_file(path, libc::O_WRONLY | libc::O_CLOEXEC, value)?;
    Ok(())
}

// for an explanation of the difference between the tasks and the .procs file
// see cgroup.md in this folder
pub fn add_pid_to_task_list(cgroup_path: &mut PathBuf, pid: &str) -> Result<()> {
    cgroup_path.push("tasks");
    write_to_cgroup(cgroup_path, pid.as_bytes())?;
    cgroup_path.pop();
    Ok(())
}

impl<'a> JailConf<'a> {
    pub fn get_mem_cgroup_v1_path(&self, pid: &str) -> PathBuf {
        let mut mem_cgroup_path = PathBuf::from_str(self.cgroup_mem_mount).unwrap();
        mem_cgroup_path.push(self.cgroup_mem_parent);
        mem_cgroup_path.push(get_folder_from_pid(pid));
        mem_cgroup_path
    }

    pub fn get_pids_cgroup_v1_path(&self, pid: &str) -> PathBuf {
        let mut pids_cgroup_path = PathBuf::from_str(self.cgroup_pids_mount).unwrap();
        pids_cgroup_path.push(self.cgroup_pids_parent);
        pids_cgroup_path.push(get_folder_from_pid(pid));
        pids_cgroup_path
    }

    pub fn get_net_cls_cgroup_v1_path(&self, pid: &str) -> PathBuf {
        let mut net_cls_cgroup_path = PathBuf::from_str(self.cgroup_net_cls_mount).unwrap();
        net_cls_cgroup_path.push(self.cgroup_net_cls_parent);
        net_cls_cgroup_path.push(get_folder_from_pid(pid));
        net_cls_cgroup_path
    }

    pub fn get_cpu_cgroup_v1_path(&self, pid: &str) -> PathBuf {
        let mut cpu_cgroup_path = PathBuf::from_str(self.cgroup_cpu_mount).unwrap();
        cpu_cgroup_path.push(self.cgroup_cpu_parent);
        cpu_cgroup_path.push(get_folder_from_pid(pid));
        cpu_cgroup_path
    }
}

pub fn init_ns_from_parent_mem(jconf: &JailConf, pid: &str) -> Result<()> {
    if jconf.cgroup_mem_max == 0 {
        return Ok(());
    }

    let mut mem_cgroup_path = jconf.get_mem_cgroup_v1_path(pid);

    create_cgroup(&mem_cgroup_path)?;

    mem_cgroup_path.push("memory.limit_in_bytes");
    write_to_cgroup(
        &mem_cgroup_path,
        jconf.cgroup_mem_max.to_string().as_bytes(),
    )?;
    mem_cgroup_path.pop();

    /*
     * Use OOM-killer instead of making processes hang/sleep
     */
    mem_cgroup_path.push("memory.oom_control");
    write_to_cgroup(&mem_cgroup_path, "0".as_bytes())?;
    mem_cgroup_path.pop();

    add_pid_to_task_list(&mut mem_cgroup_path, pid)
}

pub fn create_ns_from_parent_mem(jconf: &JailConf, id: &str) -> Result<()> {
    if jconf.cgroup_mem_max == 0 {
        return Ok(());
    }

    let mut mem_cgroup_path = jconf.get_mem_cgroup_v1_path(id);

    create_cgroup(&mem_cgroup_path)?;

    mem_cgroup_path.push("memory.limit_in_bytes");
    write_to_cgroup(
        &mem_cgroup_path,
        jconf.cgroup_mem_max.to_string().as_bytes(),
    )?;
    mem_cgroup_path.pop();

    /*
     * Use OOM-killer instead of making processes hang/sleep
     */
    mem_cgroup_path.push("memory.oom_control");
    write_to_cgroup(&mem_cgroup_path, "0".as_bytes())?;
    mem_cgroup_path.pop();

    Ok(())
}

pub fn add_child_from_parent_mem(jconf: &JailConf, cgroup_id: &str, pid: &str) -> Result<()> {
    if jconf.cgroup_mem_max == 0 {
        return Ok(());
    }

    let mut mem_cgroup_path = jconf.get_mem_cgroup_v1_path(cgroup_id);

    add_pid_to_task_list(&mut mem_cgroup_path, pid)
}

pub fn init_ns_from_parent_pids(jconf: &JailConf, pid: &str) -> Result<()> {
    if jconf.cgroup_pids_max == 0 {
        return Ok(());
    }

    let mut pids_cgroup_path = jconf.get_pids_cgroup_v1_path(pid);

    create_cgroup(&pids_cgroup_path)?;

    pids_cgroup_path.push("pids.max");
    write_to_cgroup(
        &pids_cgroup_path,
        jconf.cgroup_pids_max.to_string().as_bytes(),
    )?;
    pids_cgroup_path.pop();

    add_pid_to_task_list(&mut pids_cgroup_path, pid)
}

pub fn create_ns_from_parent_pids(jconf: &JailConf, id: &str) -> Result<()> {
    if jconf.cgroup_pids_max == 0 {
        return Ok(());
    }

    let mut pids_cgroup_path = jconf.get_pids_cgroup_v1_path(id);

    create_cgroup(&pids_cgroup_path)?;

    pids_cgroup_path.push("pids.max");
    write_to_cgroup(
        &pids_cgroup_path,
        jconf.cgroup_pids_max.to_string().as_bytes(),
    )?;
    pids_cgroup_path.pop();

    Ok(())
}

pub fn add_child_from_parent_pids(jconf: &JailConf, cgroup_id: &str, pid: &str) -> Result<()> {
    if jconf.cgroup_pids_max == 0 {
        return Ok(());
    }

    let mut pids_cgroup_path = jconf.get_pids_cgroup_v1_path(cgroup_id);

    add_pid_to_task_list(&mut pids_cgroup_path, pid)
}

pub fn init_ns_from_parent_net_cls(jconf: &JailConf, pid: &str) -> Result<()> {
    if jconf.cgroup_net_cls_classid == 0 {
        return Ok(());
    }

    let mut net_cls_cgroup_path = jconf.get_net_cls_cgroup_v1_path(pid);

    create_cgroup(&net_cls_cgroup_path)?;

    // see https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v1/net_cls.html on how to use network classifiers to rate limit toaster network bandwith
    let net_cls_classid_str = format!("0x{:x}", jconf.cgroup_net_cls_classid); // check if same as in nsjail C++ in cgroup.cc->initNsFromParentNetCls

    net_cls_cgroup_path.push("net_cls.classid");
    write_to_cgroup(&net_cls_cgroup_path, net_cls_classid_str.as_bytes())?;
    net_cls_cgroup_path.pop();

    add_pid_to_task_list(&mut net_cls_cgroup_path, pid)
}

pub fn create_ns_from_parent_net_cls(jconf: &JailConf, id: &str) -> Result<()> {
    if jconf.cgroup_net_cls_classid == 0 {
        return Ok(());
    }

    let mut net_cls_cgroup_path = jconf.get_net_cls_cgroup_v1_path(id);

    create_cgroup(&net_cls_cgroup_path)?;

    /*
     * In nsjail C++
     * ss << "0x" << std::hex << nsjconf->cgroup_net_cls_classid;
     *	net_cls_classid_str = ss.str();
     * See https://en.cppreference.com/w/cpp/io/manip/hex
     */

    let net_cls_classid_str = format!("0x{:x}", jconf.cgroup_net_cls_classid); // check if same as in nsjail C++ in cgroup.cc->initNsFromParentNetCls

    net_cls_cgroup_path.push("net_cls.classid");
    write_to_cgroup(&net_cls_cgroup_path, net_cls_classid_str.as_bytes())?;
    net_cls_cgroup_path.pop();

    Ok(())
}

pub fn add_child_from_parent_net_cls(jconf: &JailConf, cgroup_id: &str, pid: &str) -> Result<()> {
    if jconf.cgroup_net_cls_classid == 0 {
        return Ok(());
    }

    let mut net_cls_cgroup_path = jconf.get_net_cls_cgroup_v1_path(cgroup_id);

    add_pid_to_task_list(&mut net_cls_cgroup_path, pid)
}

pub fn init_ns_from_parent_cpu(jconf: &JailConf, pid: &str) -> Result<()> {
    if jconf.cgroup_cpu_ms_per_sec == 0 {
        return Ok(());
    }

    let mut cpu_cgroup_path = jconf.get_cpu_cgroup_v1_path(pid);

    create_cgroup(&cpu_cgroup_path)?;

    let cpu_us_per_sec_str = jconf.cgroup_cpu_ms_per_sec * 1000; // with *1000 ? (*1000U in NSjail C++)

    cpu_cgroup_path.push("cpu.cfs_quota_us");
    write_to_cgroup(&cpu_cgroup_path, cpu_us_per_sec_str.to_string().as_bytes())?;
    cpu_cgroup_path.pop();

    cpu_cgroup_path.push("cpu.cfs_period_us");
    write_to_cgroup(
        &cpu_cgroup_path,
        "1000000".as_bytes(), // why this value ? -Jemy: 1000*1000us = 1s, we allow cgroup_cpu_ms_per_sec / 1 second or else it would have been cpu per us / ms instead
    )?;
    cpu_cgroup_path.pop();

    add_pid_to_task_list(&mut cpu_cgroup_path, pid)
}

pub fn create_ns_from_parent_cpu(jconf: &JailConf, id: &str) -> Result<()> {
    if jconf.cgroup_cpu_ms_per_sec == 0 {
        return Ok(());
    }

    let mut cpu_cgroup_path = jconf.get_cpu_cgroup_v1_path(id);

    create_cgroup(&cpu_cgroup_path)?;

    let cpu_ms_per_sec_str = jconf.cgroup_cpu_ms_per_sec * 1000; // with *1000 ? (*1000U in NSjail C++)

    cpu_cgroup_path.push("cpu.cfs_quota_us");
    write_to_cgroup(&cpu_cgroup_path, cpu_ms_per_sec_str.to_string().as_bytes())?;
    cpu_cgroup_path.pop();

    cpu_cgroup_path.push("cpu.cfs_period_us");
    write_to_cgroup(
        &cpu_cgroup_path,
        "1000000".as_bytes(), // why this value ?
    )?;
    cpu_cgroup_path.pop();

    Ok(())
}

pub fn add_child_from_parent_cpu(jconf: &JailConf, cgroup_id: &str, pid: &str) -> Result<()> {
    if jconf.cgroup_cpu_ms_per_sec == 0 {
        return Ok(());
    }

    let mut cpu_cgroup_path = jconf.get_cpu_cgroup_v1_path(cgroup_id);

    add_pid_to_task_list(&mut cpu_cgroup_path, pid)
}

pub fn init_ns_from_parent(jconf: &JailConf, pid: &str) -> Result<()> {
    init_ns_from_parent_mem(jconf, pid)?;
    init_ns_from_parent_pids(jconf, pid)?;
    init_ns_from_parent_net_cls(jconf, pid)?;
    init_ns_from_parent_cpu(jconf, pid)
}

pub fn create_ns_from_parent(jconf: &JailConf, cgroup_id: &str) -> Result<()> {
    create_ns_from_parent_mem(jconf, cgroup_id)?;
    create_ns_from_parent_pids(jconf, cgroup_id)?;
    create_ns_from_parent_net_cls(jconf, cgroup_id)?;
    create_ns_from_parent_cpu(jconf, cgroup_id)
}

pub fn add_child_from_parent(jconf: &JailConf, cgroup_id: &str, pid: &str) -> Result<()> {
    add_child_from_parent_mem(jconf, cgroup_id, pid)?;
    add_child_from_parent_pids(jconf, cgroup_id, pid)?;
    add_child_from_parent_net_cls(jconf, cgroup_id, pid)?;
    add_child_from_parent_cpu(jconf, cgroup_id, pid)
}

pub fn join_ns(fd: i32) -> Result<()> {
    setns(fd, libc::CLONE_NEWCGROUP)?;
    Ok(())
}

pub fn remove_cgroup<T: AsRef<Path>>(cgroup_path: &T) -> Result<()> {
    /*
     * This function currently corresponds to the rmdir function on Unix- but it could change in the future, so we use libc instead
     * see https://doc.rust-lang.org/std/fs/fn.remove_dir.html
     */
    // fs::remove_dir(cgroup_path)

    // let folder_name = CStr::from_bytes_with_nul(path_to_bytes(cgroup_path.as_ref())).unwrap();

    let folder_name = CString::new(path_to_bytes(cgroup_path.as_ref())).unwrap(); // we use CString::new to create a new string terminated by a null byte \0
    SyscallReturnCode(unsafe { libc::rmdir(folder_name.as_ptr()) }).into_empty_result()?;
    Ok(())
}

/// finish_from_parent can be used with cgroup pools, just put the cgroup id into pid argument instead
pub fn finish_from_parent(jconf: &JailConf, pid: &str) -> Result<()> {
    if jconf.cgroup_mem_max != 0 {
        let pat = jconf.get_mem_cgroup_v1_path(pid);
        remove_cgroup(&pat)?;
    }

    if jconf.cgroup_pids_max != 0 {
        let pat = jconf.get_pids_cgroup_v1_path(pid);
        remove_cgroup(&pat)?;
    }

    if jconf.cgroup_net_cls_classid != 0 {
        let pat = jconf.get_net_cls_cgroup_v1_path(pid);
        remove_cgroup(&pat)?;
    }

    if jconf.cgroup_cpu_ms_per_sec != 0 {
        let pat = jconf.get_cpu_cgroup_v1_path(pid);
        remove_cgroup(&pat)?;
    }

    Ok(())
}

pub fn init_ns(_: &JailConf) -> Result<()> {
    Ok(())
}
