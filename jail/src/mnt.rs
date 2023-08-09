use std::ffi::CStr;

use super::config::{JailConf, MountT, PID, PIVOT_FOLDER};
use super::error::Result;
use sys_util::errno::Errno;
use sys_util::sched::setns;
use sys_util::stat::{umask, umask_mode};
use sys_util::{self, unistd};

pub static CONST_C_STR_ROOT: &'static [u8] = b"/\0";
pub static CONST_C_STR_TMPS: &'static [u8] = b"tmpfs\0";
pub static CONST_C_STR_TMPS_SIZE: &'static [u8] = b"size=16777216\0";
pub static CONST_C_STR_NONE: &'static [u8] = b"none\0";

// you must call this function when this program starts from parent
// it is called in lib.rs/init_package
pub fn init_mount_folder(non_root_owner: libc::uid_t, non_root_group: libc::gid_t) {
    let prev = umask_mode(0o022);
    let uid = unsafe { libc::getuid() };
    unistd::mkdir_ignore_eexist(&format!("/run/user/{}", uid), 0o755).unwrap();
    unistd::mkdir_ignore_eexist(&format!("/run/user/{}/toastainer", uid), 0o755).unwrap();
    unistd::mkdir_ignore_eexist(
        &format!("/run/user/{}/toastainer/{}", uid, unsafe { PID }),
        0o755,
    )
    .unwrap();
    unistd::mkdir_ignore_eexist(
        &format!("/run/user/{}/toastainer/{}/root", uid, unsafe { PID }),
        0o755,
    )
    .unwrap();

    umask(prev);
}

pub fn pre_init_ns_pool(jconf: &mut JailConf) -> Result<()> {
    if !jconf.clone_newns {
        init_no_clone_ns(jconf)?;
        unistd::chdir(&jconf.cwd).map_err(|e| format!("unistd::chdir {}: {}", jconf.cwd, e))?;
        return Ok(());
    }

    init_clone_ns_pre(jconf)
}

pub fn post_init_ns_pool(jconf: &mut JailConf) -> Result<()> {
    if !jconf.clone_newns {
        return Ok(());
    }

    init_clone_ns_post(jconf)?;
    unistd::chdir(&jconf.cwd).map_err(|e| format!("unistd::chdir {}: {}", jconf.cwd, e))?;
    Ok(())
}

/*
 * With mode MODE_STANDALONE_EXECVE it's required to mount /proc inside a new process,
 * as the current process is still in the original PID namespace (man pid_namespaces)
 */
pub fn init_ns(jconf: &mut JailConf) -> Result<()> {
    init_ns_internal(jconf)

    // todo: nsjail MODE_STANDALONE_EXECVE
    // if used unshare before subprocNewproc instead of clone with the flag for a new pid namespace,
    // the current process is still in the old pid namespace, so it's required to mount /proc inside a new process
    // (man pid_namespaces)
}

pub fn init_ns_internal(jconf: &mut JailConf) -> Result<()> {
    if jconf.clone_newns {
        init_clone_ns(jconf)?;
    } else {
        init_no_clone_ns(jconf)?;
    }

    unistd::chdir(&jconf.cwd).map_err(|e| format!("unistd::chdir {}: {}", jconf.cwd, e))?;

    Ok(())
}

pub fn init_clone_ns(jconf: &mut JailConf) -> Result<()> {
    init_clone_ns_pre(jconf)?;
    init_clone_ns_post(jconf)
}

fn init_clone_ns_pre(jconf: &mut JailConf) -> Result<()> {
    unistd::chdir("/").map_err(|e| format!("init_clone_ns: could not chdir in /: {}", e))?;

    /*
     * Make changes to / (recursively) private, to avoid changing the global mount ns
     * The source, filesystemtype, and data arguments are ignored when changing the propagation type of an existing mount
     * MS_PRIVATE
              Make this mount point private.  Mount and unmount events do
              not propagate into or out of this mount point.
     *
     * If one day we use unshare for some reason, in this particular case there will be no need to do this,
     * since when creating a new mount namespace, unshare assumes that the user wants a fully isolated namespace,
     * and makes all mount points private by performing the equivalent of the following command
    */
    if !jconf.mnt_ms_slave {
        let res = unsafe {
            libc::mount(
                CONST_C_STR_ROOT.as_ptr() as *const libc::c_char,
                CONST_C_STR_ROOT.as_ptr() as *const libc::c_char,
                std::ptr::null(),
                libc::MS_REC | libc::MS_PRIVATE,
                std::ptr::null() as *const libc::c_void,
            )
        };
        if res == -1 {
            return Err(("Could not make mounts private:", Errno::last()).into());
        }
    } else {
        // set mounts recursively to libc::MS_SLAVE to be able for example to mount overlay a folder in the jail folder from outside
        // the jail and for it to propagate inside
        let res = unsafe {
            libc::mount(
                CONST_C_STR_ROOT.as_ptr() as *const libc::c_char,
                CONST_C_STR_ROOT.as_ptr() as *const libc::c_char,
                std::ptr::null(),
                libc::MS_REC | libc::MS_SLAVE,
                std::ptr::null() as *const libc::c_void,
            )
        };
        if res == -1 {
            return Err(("Could not make mounts private:", Errno::last()).into());
        }
    }

    // println!("{}", cmd::exec::bash_cmd_stdout("cat /proc/self/mountinfo | sed 's/ - .*//'"));

    let res = unsafe {
        libc::mount(
            std::ptr::null(),
            CStr::from_bytes_with_nul_unchecked(PIVOT_FOLDER).as_ptr(),
            CONST_C_STR_TMPS.as_ptr() as *const libc::c_char,
            0,
            CONST_C_STR_TMPS_SIZE.as_ptr() as *const libc::c_void,
        )
    };
    if res == -1 {
        return Err(("Could not mount tmpfs:", Errno::last()).into());
    }

    Ok(())
}

fn init_clone_ns_post(jconf: &mut JailConf) -> Result<()> {
    // we do not need tmpdir contrary to nsjail since we do not mount only contents, like the src_contents field in nsjail mount struct
    // would it be more performant to mount code as content ? check out again nsjail

    for mpt in jconf.mountpts.iter_mut() {
        match mount_pt(mpt) {
            Ok(_) => {}
            Err(e) => {
                if mpt.is_mandatory {
                    return Err(e);
                }
            }
        }
    }

    /*
     * This requires some explanation: It's actually possible to pivot_root('/', '/').
     * After this operation has been completed, the old root is mounted over the new
     * root, and it's OK to simply umount('/') now, and to have new_root as '/'. This
     * allows us not care about providing any special directory for old_root, which is
     * sometimes not easy, given that e.g. /tmp might not always be present inside
     * new_root
     */
    let res = unsafe {
        libc::syscall(
            libc::SYS_pivot_root,
            CStr::from_bytes_with_nul_unchecked(PIVOT_FOLDER).as_ptr(),
            CStr::from_bytes_with_nul_unchecked(PIVOT_FOLDER).as_ptr(),
        )
    };
    if res == -1 {
        return Err(("Could not pivot:", Errno::last()).into());
    }

    let res = unsafe {
        libc::umount2(
            CONST_C_STR_ROOT.as_ptr() as *const libc::c_char,
            libc::MNT_DETACH,
        )
    };
    if res == -1 {
        return Err(("Could not umount /:", Errno::last()).into());
    }

    for mpt in jconf.mountpts.iter_mut() {
        match remount_pt(mpt) {
            Ok(_) => {}
            Err(e) => {
                if mpt.is_mandatory {
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

pub fn mount_pt(mpt: &mut MountT) -> Result<()> {
    let srcpath: *const libc::c_char;
    if let Some(ref k) = mpt.src {
        srcpath = k.as_ptr();
    } else {
        srcpath = CONST_C_STR_NONE.as_ptr() as *const libc::c_char;
    }

    if mpt.is_symlink {
        let res = unsafe { libc::symlink(srcpath, mpt.dst.as_ptr() as *const libc::c_char) };
        if res == -1 && mpt.is_mandatory {
            return Err(("Could not symlink:", Errno::last()).into());
        }
    }

    if mpt.is_dir {
        // TODO: mkdirall in case the depth is more than 1 to avoid a no such file or directory error

        let res = unsafe { libc::mkdir(mpt.dst.as_ptr() as *const libc::c_char, 0o711) };
        if res == -1 {
            let e = Errno::last();
            if e != Errno::EEXIST {
                return Err(format!("Could not mkdir in mount_pt {:?} : {}", mpt.dst, e).into());
            }
        }
    } else {
        let fd = unsafe {
            libc::open(
                mpt.dst.as_ptr() as *const libc::c_char,
                libc::O_CREAT | libc::O_RDONLY | libc::O_CLOEXEC,
                0644,
            )
        };
        if fd >= 0 {
            unsafe { libc::close(fd) };
        } else {
            return Err(format!("could not open/create {:?}", mpt.dst).into());
        }
    }

    let fs_type: *const libc::c_char;
    if let Some(ref k) = mpt.fs_type {
        fs_type = k.as_ptr();
    } else {
        fs_type = std::ptr::null() as *const libc::c_char;
    }

    let options: *const libc::c_void;
    if let Some(ref k) = mpt.options {
        options = k.as_ptr() as *const libc::c_void;
    } else {
        options = std::ptr::null();
    }

    /*
     * Initially mount it as RW, it will be remounted later on if needed
     */

    // The ! operator is implemented for many primitive types and it's equivalent to the ~ operator in C
    // See https://stackoverflow.com/questions/38896155/what-is-the-bitwise-not-operator-in-rust
    let res = unsafe {
        libc::mount(
            srcpath,
            mpt.dst.as_ptr() as *const libc::c_char,
            fs_type,
            mpt.flags & !(libc::MS_RDONLY),
            options,
        )
    };
    if res == -1 {
        return Err(format!(
            "Could not mount {:?} {:?} {:?} {:?}: {}",
            mpt.src,
            mpt.dst,
            mpt.fs_type,
            mpt.options,
            Errno::last()
        )
        .into());
    }

    mpt.mounted = true;

    Ok(())
}

/*
    Equivalent of C++ struct:

    struct {
        const unsigned long mount_flag;
        const unsigned long vfs_flag;
    } static const mountPairs[] = {
        {MS_NOSUID, ST_NOSUID},
        {MS_NODEV, ST_NODEV},
        {MS_NOEXEC, ST_NOEXEC},
        {MS_SYNCHRONOUS, ST_SYNCHRONOUS},
        {MS_MANDLOCK, ST_MANDLOCK},
        {MS_NOATIME, ST_NOATIME},
        {MS_NODIRATIME, ST_NODIRATIME},
        {MS_RELATIME, ST_RELATIME},
    };
*/
pub const MOUNT_PAIRS: [(libc::c_ulong, libc::c_ulong); 8] = [
    (libc::MS_NOSUID, libc::ST_NOSUID),
    (libc::MS_NODEV, libc::ST_NODEV),
    (libc::MS_NOEXEC, libc::ST_NOEXEC),
    (libc::MS_SYNCHRONOUS, libc::ST_SYNCHRONOUS),
    (libc::MS_MANDLOCK, libc::ST_MANDLOCK),
    (libc::MS_NOATIME, libc::ST_NOATIME),
    (libc::MS_NODIRATIME, libc::ST_NODIRATIME),
    (libc::MS_RELATIME, libc::ST_RELATIME),
];

pub const MS_LAZYTIME: libc::c_ulong = 1 << 25; /* Update the on-disk [acm]times lazily. Not defined in libc  */

pub const PER_MOUNTPOINT_FLAGS: libc::c_ulong = MS_LAZYTIME
    | libc::MS_MANDLOCK
    | libc::MS_NOATIME
    | libc::MS_NODEV
    | libc::MS_NODIRATIME
    | libc::MS_NOEXEC
    | libc::MS_NOSUID
    | libc::MS_RELATIME
    | libc::MS_RDONLY
    | libc::MS_SYNCHRONOUS;

pub fn remount_pt(mpt: &MountT) -> Result<()> {
    if !mpt.mounted {
        return Ok(());
    }
    if mpt.is_symlink {
        return Ok(());
    }

    match sys_util::statvfs::statvfs(mpt.dst_in_pivot.as_c_str()) {
        Err(e) => return Err(format!("Could not statvfs {:?}: {}", mpt.dst_in_pivot, e).into()),
        Ok(vfs) => {
            let mut new_flags =
                libc::MS_REMOUNT | libc::MS_BIND | (mpt.flags as u64 & PER_MOUNTPOINT_FLAGS);
            for i in MOUNT_PAIRS.iter() {
                if (vfs.fflags() & i.1) != 0 {
                    // C++: if (vfs.f_flag & i.vfs_flag)
                    new_flags |= i.0;
                }
            }
            let res = unsafe {
                libc::mount(
                    mpt.dst_in_pivot.as_ptr(),
                    mpt.dst_in_pivot.as_ptr(),
                    std::ptr::null(),
                    new_flags,
                    std::ptr::null() as *const libc::c_void,
                )
            };
            if res == -1 {
                return Err(("Could not remount_pt:", Errno::last()).into());
            }
            Ok(())
        }
    }
}

pub fn init_no_clone_ns(jconf: &JailConf) -> Result<()> {
    /*
     * If CLONE_NEWNS is not used, we would be changing the global mount namespace, so simply
     * use --chroot in this case
     */
    if jconf.chroot.as_bytes().len() == 0 {
        return Ok(());
    }

    unistd::chroot_cstr(&jconf.chroot)?;
    unistd::chdir("/")?;

    Ok(())
}

pub fn join_ns(fd: i32) -> Result<()> {
    setns(fd, libc::CLONE_NEWNS)?;
    Ok(())
}
