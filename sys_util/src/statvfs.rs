use libc::{self, c_ulong};
use super::errno::{Errno};
use super::NixPath;
use std::mem;

/// Wrapper around the POSIX `statvfs` struct
///
/// For more information see the [`statvfs(3)` man pages](http://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_statvfs.h.html).
#[repr(transparent)]
// #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Statvfs(libc::statvfs);

impl Statvfs {
    /// get the file system block size
    pub fn block_size(&self) -> c_ulong {
        self.0.f_bsize
    }

    /// Get the fundamental file system block size
    pub fn fragment_size(&self) -> c_ulong {
        self.0.f_frsize
    }

    /// Get the number of blocks.
    ///
    /// Units are in units of `fragment_size()`
    pub fn blocks(&self) -> libc::fsblkcnt_t {
        self.0.f_blocks
    }

    /// Get the number of free blocks in the file system
    pub fn blocks_free(&self) -> libc::fsblkcnt_t {
        self.0.f_bfree
    }

    /// Get the number of free blocks for unprivileged users
    pub fn blocks_available(&self) -> libc::fsblkcnt_t {
        self.0.f_bavail
    }

    /// Get the total number of file inodes
    pub fn files(&self) -> libc::fsfilcnt_t {
        self.0.f_files
    }

    /// Get the number of free file inodes
    pub fn files_free(&self) -> libc::fsfilcnt_t {
        self.0.f_ffree
    }

    /// Get the number of free file inodes for unprivileged users
    pub fn files_available(&self) -> libc::fsfilcnt_t {
        self.0.f_favail
    }

    /// Get the file system id
    pub fn filesystem_id(&self) -> c_ulong {
        self.0.f_fsid
    }

    /// Get the mount flags
    pub fn fflags(&self) -> c_ulong {
        self.0.f_flag
    }

    /// Get the maximum filename length
    pub fn name_max(&self) -> c_ulong {
        self.0.f_namemax
    }

}

/// Return a `Statvfs` object with information about the `path`
pub fn statvfs<P: ?Sized + NixPath>(path: &P) -> Result<Statvfs, Errno> {
    unsafe {
        Errno::clear();
        let mut stat = mem::MaybeUninit::<libc::statvfs>::uninit();
        let res = path.with_nix_path(|path|
            libc::statvfs(path.as_ptr(), stat.as_mut_ptr())
        )?;

        Errno::result(res).map(|_| Statvfs(stat.assume_init()))
    }
}

/// Return a `Statvfs` object with information about `fd`
pub fn fstatvfs(fd: libc::c_int) -> Result<Statvfs, Errno> {
    unsafe {
        Errno::clear();
        let mut stat = mem::MaybeUninit::<libc::statvfs>::uninit();
        Errno::result(libc::fstatvfs(fd, stat.as_mut_ptr()))
            .map(|_| Statvfs(stat.assume_init()))
    }
}