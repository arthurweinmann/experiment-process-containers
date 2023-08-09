use jail::error::Result;

use std::ffi::CStr;
use std::os::raw::c_char;

use sys_util::bindings::{
    btrfs_ioctl_qgroup_limit_args, btrfs_ioctl_vol_args, btrfs_ioctl_vol_args_v2,
    btrfs_ioctl_vol_args_v2__bindgen_ty_1, btrfs_ioctl_vol_args_v2__bindgen_ty_1__bindgen_ty_1,
    btrfs_ioctl_vol_args_v2__bindgen_ty_2, btrfs_qgroup_limit, BTRFS_QGROUP_LIMIT_MAX_RFER,
    _BTRFS_IOC_CLONE, _BTRFS_IOC_QGROUP_LIMIT, _BTRFS_IOC_SNAP_CREATE_V2, _BTRFS_IOC_SNAP_DESTROY,
    _BTRFS_IOC_SUBVOL_CREATE,
};
use sys_util::fcntl::{openat_no_mode, openat_no_mode_cstr, OFlag};
use sys_util::ioctl::{ioctl_with_ref_with_i32_fd, ioctl_with_val_with_i32_fd};
use sys_util::statfs;
use sys_util::NixPath;

use std::path::Path;
use std::time::Instant;

use walkdir::WalkDir;

pub fn is_btrfs_mounted(folder: &str) -> bool {
    // bash alternatives: `mount | grep "` + folder + `" | grep "type btrfs" && echo done`   ;  `btrfs filesystem show ` + folder
    let stats = match statfs::statfs(folder) {
        Ok(v) => v,
        Err(e) => {
            println!("statfs err: {}", e);
            return false;
        }
    };

    // See man7.org/linux/man-pages/man2/statfs.2.html
    if stats.filesystem_type() == statfs::BTRFS_SUPER_MAGIC {
        return true;
    }

    false
}

trait AsMutBytes {
    fn as_mut_bytes(self: &'_ mut Self) -> &'_ mut [u8];
}

impl AsMutBytes for [c_char] {
    fn as_mut_bytes(self: &'_ mut Self) -> &'_ mut [u8] {
        unsafe { ::core::mem::transmute::<&mut [c_char], &mut [u8]>(self) }
    }
}

/// new_subvolume does not check if mountpoint exists and is a valid directory, this is your responsability
/// you may use an absolute path or a relative path relative to your current directory as mount_point
/// mount_point must not end with a / ; name must not begin with one
pub fn new_subvolume<P: ?Sized + NixPath>(mount_point: &P, name: &P, size: u64) -> Result<()> {
    name.with_nix_path(|n| {
        let fd = openat_no_mode(
            libc::AT_FDCWD, // ignored if mount_point is absolute
            mount_point,
            OFlag::O_RDONLY | OFlag::O_NONBLOCK | OFlag::O_CLOEXEC | OFlag::O_DIRECTORY,
        )
        .map_err(|e| {
            format!(
                "could not open mount point {:?}: {}",
                mount_point.to_toast_path(),
                e
            )
        })?;
        let mut c_char_array: [c_char; 4088usize] = [0; 4088];
        c_char_array[..n.len()]
            .as_mut_bytes()
            .copy_from_slice(n.to_bytes());

        // println!("{}", unsafe { String::from_raw_parts(c_char_array.as_mut_ptr() as *mut u8, c_char_array.len(), c_char_array.len()) });
        let now = Instant::now();
        // let res = ioctl_with_mut_ref_with_i32_fd_no_error(
        //     fd,
        //     _BTRFS_IOC_SUBVOL_CREATE,
        //     &mut btrfs_ioctl_vol_args {
        //         fd: 0,
        //         name: c_char_array,
        //     },
        // );
        let res = unsafe {
            libc::ioctl(
                fd,
                // 1342215182,
                _BTRFS_IOC_SUBVOL_CREATE,
                &mut btrfs_ioctl_vol_args {
                    fd: 0,
                    name: c_char_array,
                } as *mut btrfs_ioctl_vol_args as *mut libc::c_void,
            )
        };
        let now2 = Instant::now();
        println!("probe {:?}", now2.checked_duration_since(now).unwrap());
        // println!("rest: {:?}", res);

        // println!("ls: {}", cmd::exec::bash_cmd_stdout("ls /home/ubuntu/toastate/build/tests/btrmnt"));
        unsafe { libc::close(fd) };
        // match res {
        //     Err(e) => return Err(format!("ioctl error: {}", e).into()),
        //     _ => {}
        // }
        if res != 0 {
            panic!(
                "res was {} and errno {}",
                res,
                sys_util::errno::Errno::last()
            );
        }
        if size > 0 {
            mount_point.with_nix_path(|m| {
                let mut arr: Vec<u8> = vec![0; m.len() + n.len() + 1];
                arr[..m.len()].copy_from_slice(m.to_bytes());
                arr[m.len()..m.len() + 1].copy_from_slice("/".as_bytes());
                arr[m.len() + 1..].copy_from_slice(n.to_bytes());
                set_quota(arr.as_slice(), size).map_err(|e| {
                    format!(
                        "could not set quota for {:?}: {}",
                        unsafe { String::from_raw_parts(arr.as_mut_ptr(), arr.len(), arr.len()) },
                        e
                    )
                })
            })??;
        }
        Ok(())
    })?
}

/// new_subvolume_cstr does not check if mountpoint exists and is a valid directory, this is your responsability
/// you may use an absolute path or a relative path relative to your current directory as mount_point
/// mount_point must not end with a / ; name must not begin with one
pub fn new_subvolume_cstr(
    mount_point: &CStr,
    name: &CStr,
    mount_point_name: &CStr,
    size: u64,
) -> Result<()> {
    let fd = openat_no_mode_cstr(
        libc::AT_FDCWD, // ignored if mount_point is absolute
        mount_point,
        libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_DIRECTORY,
    )
    .map_err(|e| {
        format!(
            "could not open mount point {:?}: {}",
            mount_point.to_toast_path(),
            e
        )
    })?;
    let mut c_char_array: [c_char; 4088usize] = [0; 4088];
    c_char_array[..name.len()]
        .as_mut_bytes()
        .copy_from_slice(name.to_bytes());

    let now1 = std::time::Instant::now();

    let res = unsafe {
        libc::ioctl(
            fd,
            _BTRFS_IOC_SUBVOL_CREATE,
            &mut btrfs_ioctl_vol_args {
                fd: 0,
                name: c_char_array,
            } as *mut btrfs_ioctl_vol_args as *mut libc::c_void,
        )
    };

    let now2 = std::time::Instant::now();
    println!("btrfs ioctl create subvolume took: {:?}", now2.checked_duration_since(now1).unwrap()); // aws ec2 m5d: first compil ~ from 2ms to 569ms. Usually has a cold start but chained creation are fast.

    unsafe { libc::close(fd) };

    if res != 0 {
        panic!(
            "res was {} and errno {}: {:?}",
            res,
            sys_util::errno::Errno::last(),
            mount_point_name,
        );
    }
    if size > 0 {
        set_quota_cstr(mount_point_name, size)
            .map_err(|e| format!("could not set quota for {:?}: {}", mount_point_name, e))?;
    }
    Ok(())
}

/// set_quota does not check if mount_point_name exist and are valid directories, this is your responsability
pub fn set_quota<P: ?Sized + NixPath>(mount_point_name: &P, size: u64) -> Result<()> {
    let fd = openat_no_mode(
        libc::AT_FDCWD, // ignored if mount_point is absolute
        mount_point_name,
        OFlag::O_RDONLY | OFlag::O_NONBLOCK | OFlag::O_CLOEXEC | OFlag::O_DIRECTORY,
    )?;
    let res = ioctl_with_ref_with_i32_fd(
        fd,
        _BTRFS_IOC_QGROUP_LIMIT,
        &btrfs_ioctl_qgroup_limit_args {
            qgroupid: 0, // automatically set -> See https://elixir.bootlin.com/linux/latest/source/fs/btrfs/ioctl.c#L4884
            lim: btrfs_qgroup_limit {
                flags: BTRFS_QGROUP_LIMIT_MAX_RFER as u64,
                max_rfer: size,
                max_excl: 0,
                rsv_rfer: 0,
                rsv_excl: 0,
            },
        },
    );
    unsafe { libc::close(fd) };
    match res {
        Err(e) => return Err(e.into()),
        _ => {}
    }

    Ok(())
}

/// set_quota_cstr does not check if mount_point_name exist and are valid directories, this is your responsability
pub fn set_quota_cstr(mount_point_name: &CStr, size: u64) -> Result<()> {
    let fd = openat_no_mode_cstr(
        libc::AT_FDCWD, // ignored if mount_point is absolute
        mount_point_name,
        libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_DIRECTORY,
    )?;
    let res = ioctl_with_ref_with_i32_fd(
        fd,
        _BTRFS_IOC_QGROUP_LIMIT,
        &btrfs_ioctl_qgroup_limit_args {
            qgroupid: 0, // automatically set -> See https://elixir.bootlin.com/linux/latest/source/fs/btrfs/ioctl.c#L4884
            lim: btrfs_qgroup_limit {
                flags: BTRFS_QGROUP_LIMIT_MAX_RFER as u64,
                max_rfer: size,
                max_excl: 0,
                rsv_rfer: 0,
                rsv_excl: 0,
            },
        },
    );
    unsafe { libc::close(fd) };
    match res {
        Err(e) => return Err(e.into()),
        _ => {}
    }

    Ok(())
}

/// dest can be either the btrfs filesystem mount point itself or the path to a subvolume
pub fn snapshot<P: ?Sized + NixPath>(src: &P, dest: &P, name: &P) -> Result<()> {
    let fd_src = openat_no_mode(
        libc::AT_FDCWD, // ignored if mount_point is absolute
        src,
        OFlag::O_RDONLY | OFlag::O_NONBLOCK | OFlag::O_CLOEXEC | OFlag::O_DIRECTORY,
    )?;

    let fd_dest = openat_no_mode(
        libc::AT_FDCWD, // ignored if mount_point is absolute
        dest,
        OFlag::O_RDONLY | OFlag::O_NONBLOCK | OFlag::O_CLOEXEC | OFlag::O_DIRECTORY,
    )?;

    let mut c_char_array: [c_char; 4040usize] = [0; 4040];
    c_char_array[..(name.len() + 1)]
        .as_mut_bytes()
        .copy_from_slice(name.to_toast_path()?.to_bytes_with_nul());

    let res = ioctl_with_ref_with_i32_fd(
        fd_dest,
        _BTRFS_IOC_SNAP_CREATE_V2,
        &btrfs_ioctl_vol_args_v2 {
            fd: fd_src as i64,
            transid: 0,
            flags: 0,
            __bindgen_anon_1: btrfs_ioctl_vol_args_v2__bindgen_ty_1 {
                __bindgen_anon_1: btrfs_ioctl_vol_args_v2__bindgen_ty_1__bindgen_ty_1 {
                    size: 0,
                    qgroup_inherit: std::ptr::null_mut(),
                },
            },
            __bindgen_anon_2: btrfs_ioctl_vol_args_v2__bindgen_ty_2 { name: c_char_array },
        },
    );
    unsafe { libc::close(fd_src) };
    unsafe { libc::close(fd_dest) };
    match res {
        Err(e) => return Err(e.into()),
        _ => {}
    }

    Ok(())
}

pub fn cow_cstr(src: &CStr, dest: &CStr) -> Result<()> {
    if let Some(dest_dir) =
        Path::new(unsafe { std::str::from_utf8_unchecked(dest.to_bytes()) }).parent()
    {
        std::fs::create_dir_all(dest_dir)?;
    }

    let fd_src = openat_no_mode_cstr(
        libc::AT_FDCWD, // ignored if mount_point is absolute
        src,
        libc::O_RDONLY | libc::O_NOFOLLOW,
    )
    .map_err(|e| format!("openfd src {:?}: {}", src, e))?;

    let fd_dest = openat_no_mode_cstr(
        libc::AT_FDCWD, // ignored if mount_point is absolute
        dest,
        libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
    )
    .map_err(|e| format!("openfd dest {:?}: {}", dest, e))?;

    ioctl_with_val_with_i32_fd(fd_dest, _BTRFS_IOC_CLONE, fd_src as u64)
        .map_err(|e| format!("ioctl clone: {}", e))?;

    Ok(())
}

pub fn cow_recursive(src: &str, dest: &str) -> Result<()> {
    let dest = Path::new(dest);
    for entry in WalkDir::new(src)
        .into_iter()
        .filter_map(walkdir::Result::ok)
        .filter(|e| e.file_type().is_dir())
    {
        let relative = entry.path().strip_prefix(src).expect("Not a prefix");
        let dest = dest.join(relative);

        // println!("mkdir {:?}", dest);

        // unistd::mkdir_ignore_eexist(&dest.join(relative), 0755)
        //     .map_err(|e| format!("error mkdir {:?}: {}", dest.join(relative), e))?;
        std::fs::create_dir_all(&dest)?
    }

    for entry in WalkDir::new(src)
        .into_iter()
        .filter_map(walkdir::Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let fd_src = openat_no_mode(
            libc::AT_FDCWD, // ignored if mount_point is absolute
            entry.path(),
            OFlag::O_RDONLY | OFlag::O_NOFOLLOW,
        )
        .map_err(|e| format!("openfd src {:?}: {}", entry.path(), e))?;

        let fd_dest = openat_no_mode(
            libc::AT_FDCWD, // ignored if mount_point is absolute
            &dest.join(entry.path().strip_prefix(src).expect("Not a prefix 2")),
            OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_EXCL,
        )
        .map_err(|e| {
            format!(
                "openfd dest {:?}: {}",
                dest.join(entry.path().strip_prefix(src).expect("Not a prefix 2")),
                e
            )
        })?;

        ioctl_with_val_with_i32_fd(fd_dest, _BTRFS_IOC_CLONE, fd_src as u64)
            .map_err(|e| format!("ioctl clone: {}", e))?;
    }

    Ok(())
}

pub fn delete_subvolume<P: ?Sized + NixPath>(mount_point: &P, name: &P) -> Result<()> {
    let fd = openat_no_mode(
        libc::AT_FDCWD, // ignored if mount_point is absolute
        mount_point,
        OFlag::O_RDONLY | OFlag::O_NONBLOCK | OFlag::O_CLOEXEC | OFlag::O_DIRECTORY,
    )?;

    let mut c_char_array: [c_char; 4088usize] = [0; 4088];
    c_char_array[..(name.len() + 1)]
        .as_mut_bytes()
        .copy_from_slice(name.to_toast_path()?.to_bytes_with_nul());

    let res = ioctl_with_ref_with_i32_fd(
        fd,
        _BTRFS_IOC_SNAP_DESTROY,
        &btrfs_ioctl_vol_args {
            fd: 0,
            name: c_char_array,
        },
    );
    unsafe { libc::close(fd) };
    match res {
        Err(e) => return Err(e.into()),
        _ => {}
    }

    Ok(())
}

pub fn delete_subvolume_cstr(mount_point: &CStr, name: &[u8]) -> Result<()> {
    let fd = openat_no_mode_cstr(
        libc::AT_FDCWD, // ignored if mount_point is absolute
        mount_point,
        libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_DIRECTORY,
    )?;

    let mut c_char_array: [c_char; 4088usize] = [0; 4088];
    c_char_array[..name.len()]
        .as_mut_bytes()
        .copy_from_slice(name);

    let res = ioctl_with_ref_with_i32_fd(
        fd,
        _BTRFS_IOC_SNAP_DESTROY,
        &btrfs_ioctl_vol_args {
            fd: 0,
            name: c_char_array,
        },
    );
    unsafe { libc::close(fd) };
    match res {
        Err(e) => return Err(e.into()),
        _ => {}
    }

    Ok(())
}
