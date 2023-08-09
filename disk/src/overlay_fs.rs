use super::btrfs;
use jail::error::Result;

use sys_util::errno::Errno;
use sys_util::unistd::{chown_cstr, mkdir_cstr, mkdir_ignore_eexist_cstr};

use std::ffi::{CStr, CString};

static overlay_cstr: &'static [u8] = b"overlay\0";
static slash: &'static [u8] = b"/";
static null: &'static [u8] = b"\0"; // b"\000"
static workdir: &'static [u8] = b"workdir";
static volume: &'static [u8] = b"volume";
static lowerdir_data: &'static [u8] = b"lowerdir=";
static upperdir_data: &'static [u8] = b",upperdir=";
static workdir_data: &'static [u8] = b",workdir=";

// https://jvns.ca/blog/2019/11/18/how-containers-work--overlayfs/

pub struct OverlayDir {
    btrfs_file_system: CString,
    pub upperdir: CString,
    pub workdir: CString,
    subvolume_name: CString,
    pub uid: CString,
    pub mount_point: CString,
    pub mounted: bool,
}

impl OverlayDir {
    /// if is_for_pooling is true, we need to do an extra chown non root user of the mount point
    pub fn new(
        btrfs_file_system: CString,
        overlay_dir: &[u8],
        uid: CString,
        // quota_b: u64,
        non_root_owner: libc::uid_t,
        non_root_group: libc::gid_t,
        is_for_pooling: bool,
    ) -> Result<Self> {
        let subvolume_name = [btrfs_file_system.to_bytes(), slash, uid.to_bytes()].concat();
        let subvolume_workdir = unsafe {
            CString::from_vec_unchecked([subvolume_name.as_slice(), slash, workdir].concat())
        };
        let subvolume_upperdir = unsafe {
            CString::from_vec_unchecked([subvolume_name.as_slice(), slash, volume].concat())
        };
        let subvolume_name = unsafe { CString::from_vec_unchecked(subvolume_name) };

        // Done in Gtvs btrfs pool
        // btrfs::new_subvolume_cstr(&btrfs_file_system, &uid, &subvolume_name, quota_b).map_err(
        //     |e| {
        //         format!(
        //             "could not btrfs::new_subvolume_cstr({:?}, {:?}, {:?}, {:?}): {}",
        //             &btrfs_file_system, &uid, &subvolume_name, quota_b, e
        //         )
        //     },
        // )?;

        let mount_point =
            unsafe { CString::from_vec_unchecked([overlay_dir, slash, uid.to_bytes()].concat()) };

        mkdir_cstr(&mount_point, 0o755)
            .map_err(|e| format!("could not mkdir {:?}: {}", mount_point, e))?;
        mkdir_cstr(&subvolume_workdir, 0o755)
            .map_err(|e| format!("could not mkdir {:?}: {}", subvolume_workdir, e))?;
        mkdir_cstr(&subvolume_upperdir, 0o755)
            .map_err(|e| format!("could not mkdir {:?}: {}", subvolume_upperdir, e))?;

        chown_cstr(
            subvolume_upperdir.as_c_str(),
            non_root_owner,
            non_root_group,
        )?; // if all parent dir are not owned by ubuntu, mount overlay shifts files and folders owner to root

        if is_for_pooling {
            // is needed in the case of a pool toaster because the mount overlay is done after the nm init
            chown_cstr(mount_point.as_c_str(), non_root_owner, non_root_group)
                .expect("could not chown overlay mount dir");
        }

        Ok(OverlayDir {
            btrfs_file_system: btrfs_file_system,
            upperdir: subvolume_upperdir,
            workdir: subvolume_workdir,
            subvolume_name: subvolume_name,
            uid: uid,
            mount_point: mount_point,
            mounted: false,
        })
    }

    /// mount_point must already exists
    pub fn mount(&mut self, lower_dirs: &[u8]) -> Result<()> {
        let data = [
            lowerdir_data,
            lower_dirs,
            upperdir_data,
            self.upperdir.to_bytes(),
            workdir_data,
            self.workdir.to_bytes(),
            null,
        ]
        .concat();

        let res = unsafe {
            libc::mount(
                overlay_cstr.as_ptr() as *const libc::c_char,
                self.mount_point.as_ptr(),
                overlay_cstr.as_ptr() as *const libc::c_char,
                libc::MS_MGC_VAL,
                data.as_ptr() as *const libc::c_void,
            )
        };
        if res == -1 {
            return Err((format!(
                "Could not mount overlayfs with data: {:?} {:?}: {}",
                unsafe { String::from_utf8_unchecked(data.clone()) },
                data,
                Errno::last()
            ))
            .into());
        }
        self.mounted = true;
        Ok(())
    }

    pub fn kill(self) -> Result<()> {
        let res = unsafe { libc::umount2(self.mount_point.as_ptr(), libc::MNT_DETACH) }; // MNT_DETACH instead of MNT_FORCE because of the unix listener in gtvs
        if res == -1 {
            return Err(("Could not umount overlayfs:", Errno::last()).into());
        }
        // btrfs::delete_subvolume_cstr(&self.btrfs_file_system, self.uid.to_bytes())?;
        // btrfs subvolume and overlay mount remaining dir must be deleted in go
        Ok(())
    }

    pub fn resize(&self, size: u64) -> Result<()> {
        btrfs::set_quota_cstr(&self.subvolume_name, size)?;
        Ok(())
    }

    pub fn generate_overlay_data(&self, lower_dirs: &[u8]) -> Vec<u8> {
        [
            lowerdir_data,
            lower_dirs,
            upperdir_data,
            self.upperdir.to_bytes(),
            workdir_data,
            self.workdir.to_bytes(),
            null,
        ]
        .concat()
    }
}

pub fn mount_overlay(data: &[u8], mount_point: &CStr) -> Result<()> {
    let res = unsafe {
        libc::mount(
            overlay_cstr.as_ptr() as *const libc::c_char,
            mount_point.as_ptr(),
            overlay_cstr.as_ptr() as *const libc::c_char,
            libc::MS_MGC_VAL,
            data.as_ptr() as *const libc::c_void,
        )
    };
    if res == -1 {
        return Err((format!(
            "Could not mount_overlay with data: {:?}: {}",
            unsafe { String::from_utf8_unchecked(data.to_vec()) },
            Errno::last()
        ))
        .into());
    }
    Ok(())
}
