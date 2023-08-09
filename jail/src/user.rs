use super::config::JailConf;
use super::error::Result;
use super::subproc;
use super::utils::write_buf_to_file;
use std::ffi::{CStr, CString};
use sys_util::errno::Errno;
use sys_util::sched::setns;
use sys_util::bindings::{__NR_setresgid, __NR_setresuid};
// use cmd::exec;

// glib securebits.h ; nsjail user.cc

/* When set, a process can retain its capabilities even after
transitioning to a non-root user (the set-uid fixup suppressed by
bit 2). Bit-4 is cleared when a process calls exec(); setting both
bit 4 and 5 will create a barrier through exec that no exec()'d
child can use this feature again. */
const SECURE_KEEP_CAPS: libc::c_int = 4;
// const SECURE_KEEP_CAPS_LOCKED: libc::c_int = 5; /* make bit-4 immutable */
/* Each securesetting is implemented using two bits. One bit specifies
whether the setting is on or off. The other bit specify whether the
setting is locked or not. A setting which is locked cannot be
changed from user-level. */
// #define issecure_mask(X)	(1 << (X))

/*
#define SECBIT_KEEP_CAPS	(issecure_mask(SECURE_KEEP_CAPS))
#define SECBIT_KEEP_CAPS_LOCKED (issecure_mask(SECURE_KEEP_CAPS_LOCKED))
*/

const SECBIT_KEEP_CAPS: libc::c_int = 1 << SECURE_KEEP_CAPS;
// const SECBIT_KEEP_CAPS_LOCKED: libc::c_int = 1 << SECURE_KEEP_CAPS_LOCKED;

/* When set, setuid to/from uid 0 does not trigger capability-"fixup".
When unset, to provide compatiblility with old programs relying on
set*uid to gain/lose privilege, transitions to/from uid 0 cause
capabilities to be gained/lost. */
const SECURE_NO_SETUID_FIXUP: libc::c_int = 2;
// const SECURE_NO_SETUID_FIXUP_LOCKED: libc::c_int = 3; /* make bit-2 immutable */
const SECBIT_NO_SETUID_FIXUP: libc::c_int = 1 << SECURE_NO_SETUID_FIXUP;
// const SECBIT_NO_SETUID_FIXUP_LOCKED: libc::c_int = 1 << SECURE_NO_SETUID_FIXUP_LOCKED;

pub fn set_res_gid(gid: libc::gid_t) -> Result<()> {
    // let sysno = cpp_bindings::get_set_res_gid_syscall_number();

    if unsafe {
        libc::syscall(
            __NR_setresgid as i64,
            gid as libc::c_long,
            gid as libc::c_long,
            gid as libc::c_long,
        )
    } == -1
    {
        return Err(format!("set_res_gid: {}", Errno::last()).into());
    }

    Ok(())
}

pub fn set_res_uid(uid: libc::uid_t) -> Result<()> {
    // let sysno = cpp_bindings::get_set_res_uid_syscall_number();

    if unsafe { libc::syscall(__NR_setresuid as i64, uid, uid, uid) } == -1 {
        return Err(format!("set_res_uid: {}", Errno::last()).into());
    }

    Ok(())
}

// allow setgroups when using exclusively newgid: nsjail commit Nov 1 : https://github.com/google/nsjail/commit/1111bb135a8a13231c8754cf0b45b58e4c0e9cb6
pub fn has_gid_map_self(jcon: &JailConf) -> bool {
    for gid in jcon.gids.iter() {
        if !gid.is_newidmap {
            return true;
        }
    }
    false
}

pub fn set_group_deny(jconf: &JailConf, pid: &str) -> Result<()> {
    /*
     * No need to write 'deny' to /proc/pid/setgroups if our euid==0, as writing to
     * uid_map/gid_map will succeed anyway
     */
    if !jconf.clone_newuser || jconf.orig_euid == 0 || !has_gid_map_self(jconf) {
        return Ok(());
    }

    match write_buf_to_file(
        &format!("/proc/{}/setgroups", pid),
        libc::O_WRONLY | libc::O_CLOEXEC,
        "deny".as_bytes(),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn gid_map_self(jconf: &JailConf, pid: &str) -> Result<()> {
    let mut map = String::new();
    for gid in &jconf.gids {
        if gid.is_newidmap {
            continue;
        }

        map.push_str(&format!(
            "{} {} {}\n",
            gid.inside_id, gid.outside_id, gid.count
        ))
    }

    if map.is_empty() {
        return Ok(());
    }

    // println!("map: {} {}", pid, map);

    match write_buf_to_file(
        &format!("/proc/{}/gid_map", pid),
        libc::O_WRONLY | libc::O_CLOEXEC,
        map.as_bytes(),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error in gid_map_self:write_buf_to_file: {:?}", e).into()),
    }
}

pub fn uid_map_self(jconf: &JailConf, pid: &str) -> Result<()> {
    let mut map = String::new();
    for uid in &jconf.uids {
        if uid.is_newidmap {
            continue;
        }

        map.push_str(&format!(
            "{} {} {}\n",
            uid.inside_id, uid.outside_id, uid.count
        ))
    }

    if map.is_empty() {
        return Ok(());
    }

    match write_buf_to_file(
        &format!("/proc/{}/uid_map", pid),
        libc::O_WRONLY | libc::O_CLOEXEC,
        map.as_bytes(),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/* Use /usr/bin/newgidmap for writing the gid map: http://man7.org/linux/man-pages/man1/newgidmap.1.html*/
pub fn gid_map_external(jconf: &JailConf, pid: &str, env: Option<&[CString]>) -> Result<()> {
    let mut used = false;

    // let mut cmd = format!("/usr/bin/newgidmap {}", pid);

    let mut cmd = String::from("/usr/bin/newgidmap ");
    cmd.push_str(pid);

    for gid in &jconf.gids {
        if !gid.is_newidmap {
            continue;
        }
        used = true;

        // cmd.push_str(format!(" {} {} {}", gid.inside_id, gid.outside_id, gid.count));

        cmd.push_str(" ");
        cmd.push_str(&gid.inside_id.to_string());
        cmd.push_str(" ");
        cmd.push_str(&gid.outside_id.to_string());
        cmd.push_str(" ");
        cmd.push_str(&gid.count.to_string());
    }

    if !used {
        return Ok(());
    }

    // println!("gid command: {}", cmd);

    // println!("jconf orig_euid: {}", jconf.orig_euid);

    // println!("in /proc/pid/gid_map: {}", utils::readln_special(&format!("/proc/{}/gid_map", pid)).unwrap());

    // exec::BashCommand::new_sh(&cmd).run_utf8()?;

    let exe_res = subproc::system_exe_wrapper(&cmd, env)?;

    if exe_res != 0 {
        return Err(format!(
            "failed to system_exe_wrapper in gid_map_external: {}",
            exe_res
        )
        .into());
    }

    Ok(())
}

/* Use /usr/bin/newuidmap for writing the uid map: http://man7.org/linux/man-pages/man1/newuidmap.1.html */
pub fn uid_map_external(jconf: &JailConf, pid: &str, env: Option<&[CString]>) -> Result<()> {
    let mut used = false;

    let mut cmd = String::from("/usr/bin/newuidmap ");
    cmd.push_str(pid);

    for uid in &jconf.uids {
        if !uid.is_newidmap {
            continue;
        }
        used = true;

        cmd.push_str(" ");
        cmd.push_str(&uid.inside_id.to_string());
        cmd.push_str(" ");
        cmd.push_str(&uid.outside_id.to_string());
        cmd.push_str(" ");
        cmd.push_str(&uid.count.to_string());
    }

    if !used {
        return Ok(());
    }

    if subproc::system_exe_wrapper(&cmd, env)? != 0 {
        return Err("failed to system_exe_wrapper in uid_map_external".into());
    }

    Ok(())
}

pub fn uid_gid_map(jconf: &JailConf, pid: &str, env: Option<&[CString]>) -> Result<()> {
    gid_map_self(jconf, pid)?;
    gid_map_external(jconf, pid, env)?;
    uid_map_self(jconf, pid)?;
    uid_map_external(jconf, pid, env)
}

// env is equivalent of C++ environ variable defined as a global variable in the Glibc source file posix/environ.c
// maybe we can get the same with https://doc.rust-lang.org/std/env/index.html ?
pub fn init_ns_from_parent(jconf: &JailConf, pid: &str, env: Option<&[CString]>) -> Result<()> {
    set_group_deny(jconf, pid)?;

    if !jconf.clone_newuser {
        return Ok(());
    }

    uid_gid_map(jconf, pid, env)
}

pub fn init_ns_from_child(jconf: &JailConf) -> Result<()> {
    if !jconf.clone_newuser && jconf.orig_euid != 0 {
        return Ok(());
    }

    /*
     * Make sure all capabilities are retained after the subsequent setuid/setgid, as they will
     * be needed for privileged operations: mounts, uts change etc.
     */
    if unsafe {
        libc::prctl(
            libc::PR_SET_SECUREBITS,
            SECBIT_KEEP_CAPS | SECBIT_NO_SETUID_FIXUP,
            0,
            0,
            0,
        )
    } == -1
    {
        return Err(format!("init_ns_from_child: prctl(PR_SET_SECUREBITS, SECBIT_KEEP_CAPS | SECBIT_NO_SETUID_FIXUP): {}", Errno::last()).into());
    }

    /*
     * Best effort because of /proc/self/setgroups. We deny
     * setgroups(2) calls only if user namespaces are in use.
     */
    let mut groups: Vec<libc::gid_t> = vec![];
    let mut group_string = String::from("[");

    if !jconf.clone_newuser && jconf.gids.len() > 1 {
        let mut first = true;
        for it in &jconf.gids {
            if !first {
                group_string.push_str(", ");
            }
            first = false;
            groups.push(it.inside_id);
            group_string.push_str(&it.inside_id.to_string());
        }
    }
    group_string.push_str("]");

    if jconf.user_inside_gid > -1 {
        set_res_gid(jconf.user_inside_gid as u32)?;
    } else {
        set_res_gid(jconf.gids[0].inside_id)?;
    }

    // as size_t is only for linux, otherwise use c_int (for openbsd for exemple, see nix crate unistd.rs for more information)
    if unsafe {
        libc::setgroups(
            groups.len() as libc::size_t,
            groups.as_ptr() as *const libc::gid_t,
        )
    } == -1
    {
        /* Indicate error if specific groups were requested */
        if groups.len() > 0 {
            return Err(format!("init_ns_from_child: setgroups: {}", Errno::last()).into());
        }
    }

    if jconf.user_inside_uid > -1 {
        set_res_uid(jconf.user_inside_uid as u32)?;
    } else {
        set_res_uid(jconf.uids[0].inside_id)?;
    }

    /*
     * Disable securebits again to avoid spawned programs
     * unexpectedly retaining capabilities after a UID/GID
     * change.
     */
    if unsafe { libc::prctl(libc::PR_SET_SECUREBITS, 0, 0, 0, 0) } == -1 {
        return Err(format!("prctl(libc::PR_SET_SECUREBITS, 0): {}", Errno::last()).into());
    }

    Ok(())
}

pub fn join_ns(fd: i32) -> Result<()> {
    setns(fd, libc::CLONE_NEWUSER)?;
    Ok(())
}
