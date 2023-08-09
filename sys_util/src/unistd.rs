use super::errno::{self, Errno};
use super::NixPath;
use libc::{self, c_int, gid_t, pid_t, uid_t};
use std::ffi::CStr;
use std::{fmt, mem};

/// Computes the raw UID and GID values to pass to a `*chown` call.
fn chown_raw_ids(owner: Option<Uid>, group: Option<Gid>) -> (libc::uid_t, libc::gid_t) {
    // According to the POSIX specification, -1 is used to indicate that owner and group
    // are not to be changed.  Since uid_t and gid_t are unsigned types, we have to wrap
    // around to get -1.
    let uid = owner.map(Into::into)
        .unwrap_or_else(|| (0 as uid_t).wrapping_sub(1));
    let gid = group.map(Into::into)
        .unwrap_or_else(|| (0 as gid_t).wrapping_sub(1));
    (uid, gid)
}

/// Change the ownership of the file at `path` to be owned by the specified
/// `owner` (user) and `group` (see
/// [chown(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/chown.html)).
///
/// The owner/group for the provided path name will not be modified if `None` is
/// provided for that argument.  Ownership change will be attempted for the path
/// only if `Some` owner/group is provided.
#[inline]
pub fn chown<P: ?Sized + NixPath>(path: &P, owner: Option<Uid>, group: Option<Gid>) -> Result<(), errno::Errno> {
    let res = path.with_nix_path(|cstr| {
        let (uid, gid) = chown_raw_ids(owner, group);
        unsafe { libc::chown(cstr.as_ptr(), uid, gid) }
    })?;

    Errno::result(res).map(drop)
}

#[inline]
pub fn chown_cstr(path: &CStr, owner: uid_t, group: gid_t) -> Result<(), errno::Errno> {
    let res = unsafe { libc::chown(path.as_ptr(), owner, group) };

    Errno::result(res).map(drop)
}

/// User identifier
///
/// Newtype pattern around `uid_t` (which is just alias). It prevents bugs caused by accidentally
/// passing wrong value.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Uid(uid_t);

impl Uid {
    /// Creates `Uid` from raw `uid_t`.
    pub fn from_raw(uid: uid_t) -> Self {
        Uid(uid)
    }

    /// Returns Uid of calling process. This is practically a more Rusty alias for `getuid`.
    pub fn current() -> Self {
        getuid()
    }

    /// Returns effective Uid of calling process. This is practically a more Rusty alias for `geteuid`.
    pub fn effective() -> Self {
        geteuid()
    }

    /// Returns true if the `Uid` represents privileged user - root. (If it equals zero.)
    pub fn is_root(self) -> bool {
        self == ROOT
    }

    /// Get the raw `uid_t` wrapped by `self`.
    pub fn as_raw(self) -> uid_t {
        self.0
    }
}

impl From<Uid> for uid_t {
    fn from(uid: Uid) -> Self {
        uid.0
    }
}

impl fmt::Display for Uid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Constant for UID = 0
pub const ROOT: Uid = Uid(0);

/// Group identifier
///
/// Newtype pattern around `gid_t` (which is just alias). It prevents bugs caused by accidentally
/// passing wrong value.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Gid(gid_t);

impl Gid {
    /// Creates `Gid` from raw `gid_t`.
    pub fn from_raw(gid: gid_t) -> Self {
        Gid(gid)
    }

    /// Returns Gid of calling process. This is practically a more Rusty alias for `getgid`.
    pub fn current() -> Self {
        getgid()
    }

    /// Returns effective Gid of calling process. This is practically a more Rusty alias for `getegid`.
    pub fn effective() -> Self {
        getegid()
    }

    /// Get the raw `gid_t` wrapped by `self`.
    pub fn as_raw(self) -> gid_t {
        self.0
    }
}

impl From<Gid> for gid_t {
    fn from(gid: Gid) -> Self {
        gid.0
    }
}

impl fmt::Display for Gid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Get a real user ID
///
/// See also [getuid(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/getuid.html)
// POSIX requires that getuid is always successful, so no need to check return
// value or errno.
#[inline]
pub fn getuid() -> Uid {
    Uid(unsafe { libc::getuid() })
}

/// Get the effective user ID
///
/// See also [geteuid(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/geteuid.html)
// POSIX requires that geteuid is always successful, so no need to check return
// value or errno.
#[inline]
pub fn geteuid() -> Uid {
    Uid(unsafe { libc::geteuid() })
}

/// Get the real group ID
///
/// See also [getgid(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/getgid.html)
// POSIX requires that getgid is always successful, so no need to check return
// value or errno.
#[inline]
pub fn getgid() -> Gid {
    Gid(unsafe { libc::getgid() })
}

/// Get the effective group ID
///
/// See also [getegid(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/getegid.html)
// POSIX requires that getegid is always successful, so no need to check return
// value or errno.
#[inline]
pub fn getegid() -> Gid {
    Gid(unsafe { libc::getegid() })
}

#[inline]
pub fn chroot<P: ?Sized + NixPath>(path: &P) -> Result<(), errno::Errno> {
    let res = path.with_nix_path(|cstr| unsafe { libc::chroot(cstr.as_ptr()) })?;

    Errno::result(res).map(drop)
}

pub fn chroot_cstr(path: &CStr) -> Result<(), errno::Errno> {
    Errno::result(unsafe { libc::chroot(path.as_ptr()) }).map(drop)
}

// from nix crates, branch master
// syscall doc at http://man7.org/linux/man-pages/man2/pipe.2.html
pub fn pipe2(flags: c_int) -> Result<(c_int, c_int), errno::Errno> {
    let mut fds = mem::MaybeUninit::<[c_int; 2]>::uninit();

    let res = unsafe { libc::pipe2(fds.as_mut_ptr() as *mut c_int, flags) };

    errno::Errno::result(res)?;

    unsafe { Ok((fds.assume_init()[0], fds.assume_init()[1])) }
}

/// Process identifier
///
/// Newtype pattern around `pid_t` (which is just alias). It prevents bugs caused by accidentally
/// passing wrong value.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Pid(pid_t);

impl Pid {
    /// Creates `Pid` from raw `pid_t`.
    pub fn from_raw(pid: pid_t) -> Self {
        Pid(pid)
    }

    /// Returns PID of calling process
    pub fn this() -> Self {
        getpid()
    }

    /// Returns PID of parent of calling process
    pub fn parent() -> Self {
        getppid()
    }

    /// Get the raw `pid_t` wrapped by `self`.
    pub fn as_raw(self) -> pid_t {
        self.0
    }
}

impl From<Pid> for pid_t {
    fn from(pid: Pid) -> Self {
        pid.0
    }
}

impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Represents the successful result of calling `fork`
///
/// When `fork` is called, the process continues execution in the parent process
/// and in the new child.  This return type can be examined to determine whether
/// you are now executing in the parent process or in the child.
#[derive(Clone, Copy, Debug)]
pub enum ForkResult {
    Parent { child: Pid },
    Child,
}

impl ForkResult {
    /// Return `true` if this is the child process of the `fork()`
    #[inline]
    pub fn is_child(self) -> bool {
        match self {
            ForkResult::Child => true,
            _ => false,
        }
    }

    /// Returns `true` if this is the parent process of the `fork()`
    #[inline]
    pub fn is_parent(self) -> bool {
        !self.is_child()
    }
}

/// Create a new child process duplicating the parent process ([see
/// fork(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/fork.html)).
///
/// After calling the fork system call (successfully) two processes will
/// be created that are identical with the exception of their pid and the
/// return value of this function.  As an example:
///
/// ```no_run
/// use nix::unistd::{fork, ForkResult};
///
/// match fork() {
///    Ok(ForkResult::Parent { child, .. }) => {
///        println!("Continuing execution in parent process, new child has pid: {}", child);
///    }
///    Ok(ForkResult::Child) => println!("I'm a new child process"),
///    Err(_) => println!("Fork failed"),
/// }
/// ```
///
/// This will print something like the following (order indeterministic).  The
/// thing to note is that you end up with two processes continuing execution
/// immediately after the fork call but with different match arms.
///
/// ```text
/// Continuing execution in parent process, new child has pid: 1234
/// I'm a new child process
/// ```
///
/// # Safety
///
/// In a multithreaded program, only [async-signal-safe] functions like `pause`
/// and `_exit` may be called by the child (the parent isn't restricted). Note
/// that memory allocation may **not** be async-signal-safe and thus must be
/// prevented.
///
/// Those functions are only a small subset of your operating system's API, so
/// special care must be taken to only invoke code you can control and audit.
///
/// [async-signal-safe]: http://man7.org/linux/man-pages/man7/signal-safety.7.html
#[inline]
pub fn fork() -> Result<ForkResult, errno::Errno> {
    use self::ForkResult::*;
    let res = unsafe { libc::fork() };

    Errno::result(res).map(|res| match res {
        0 => Child,
        res => Parent { child: Pid(res) },
    })
}

/// Get the pid of this process (see
/// [getpid(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/getpid.html)).
///
/// Since you are running code, there is always a pid to return, so there
/// is no error case that needs to be handled.
#[inline]
pub fn getpid() -> Pid {
    Pid(unsafe { libc::getpid() })
}

/// Get the pid of this processes' parent (see
/// [getpid(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/getppid.html)).
///
/// There is always a parent pid to return, so there is no error case that needs
/// to be handled.
#[inline]
pub fn getppid() -> Pid {
    Pid(unsafe { libc::getppid() }) // no error handling, according to man page: "These functions are always successful."
}

/// Change the current working directory of the calling process (see
/// [chdir(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/chdir.html)).
///
/// This function may fail in a number of different scenarios.  See the man
/// pages for additional details on possible failure cases.
#[inline]
pub fn chdir<P: ?Sized + NixPath>(path: &P) -> Result<(), errno::Errno> {
    let res = path.with_nix_path(|cstr| unsafe { libc::chdir(cstr.as_ptr()) })?;

    Errno::result(res).map(drop)
}

/// Creates new directory `path` with access rights `mode`.  (see [mkdir(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/mkdir.html))
///
/// # Errors
///
/// There are several situations where mkdir might fail:
///
/// - current user has insufficient rights in the parent directory
/// - the path already exists
/// - the path name is too long (longer than `PATH_MAX`, usually 4096 on linux, 1024 on OS X)
///
/// # Example
///
/// ```rust
/// extern crate tempfile;
/// extern crate nix;
///
/// use nix::unistd;
/// use nix::sys::stat;
/// use tempfile::tempdir;
///
/// fn main() {
///     let tmp_dir1 = tempdir().unwrap();
///     let tmp_dir2 = tmp_dir1.path().join("new_dir");
///
///     // create new directory and give read, write and execute rights to the owner
///     match unistd::mkdir(&tmp_dir2, stat::Mode::S_IRWXU) {
///        Ok(_) => println!("created {:?}", tmp_dir2),
///        Err(err) => println!("Error creating directory: {}", err),
///     }
/// }
/// ```
#[inline]
pub fn mkdir<P: ?Sized + NixPath>(path: &P, mode: libc::mode_t) -> Result<(), errno::Errno> {
    let res = path.with_nix_path(|cstr| unsafe { libc::mkdir(cstr.as_ptr(), mode) })?;

    Errno::result(res).map(drop)
}

#[inline]
pub fn mkdir_ignore_eexist<P: ?Sized + NixPath>(
    path: &P,
    mode: libc::mode_t,
) -> Result<(), errno::Errno> {
    let res = path.with_nix_path(|cstr| unsafe { libc::mkdir(cstr.as_ptr(), mode) })?;

    match Errno::result(res).map(drop) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e == Errno::EEXIST {
                return Ok(());
            }
            Err(e)
        }
    }
}

#[inline]
pub fn mkdir_ignore_eexist_cstr(
    path: &CStr,
    mode: libc::mode_t,
) -> Result<(), errno::Errno> {
    let res = unsafe { libc::mkdir(path.as_ptr(), mode) };

    match Errno::result(res).map(drop) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e == Errno::EEXIST {
                return Ok(());
            }
            Err(e)
        }
    }
}

#[inline]
pub fn mkdir_cstr(path: &CStr, mode: libc::mode_t) -> Result<(), errno::Errno> {
    Errno::result(unsafe { libc::mkdir(path.as_ptr(), mode) }).map(drop)
}

/// Creates a symbolic link at `path2` which points to `path1`.
///
/// If `dirfd` has a value, then `path2` is relative to directory associated
/// with the file descriptor.
///
/// If `dirfd` is `None`, then `path2` is relative to the current working
/// directory. This is identical to `libc::symlink(path1, path2)`.
///
/// See also [symlinkat(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/symlinkat.html).
pub fn symlinkat<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(
    path1: &P1,
    dirfd: Option<libc::c_int>,
    path2: &P2,
) -> Result<(), errno::Errno> {
    let res = path1.with_nix_path(|path1| {
        path2.with_nix_path(|path2| unsafe {
            libc::symlinkat(
                path1.as_ptr(),
                dirfd.unwrap_or(libc::AT_FDCWD),
                path2.as_ptr(),
            )
        })
    })??;
    Errno::result(res).map(drop)
}

pub fn symlink<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(
    path1: &P1,
    path2: &P2,
) -> Result<(), errno::Errno> {
    let res = path1.with_nix_path(|path1| {
        path2.with_nix_path(|path2| unsafe { libc::symlink(path1.as_ptr(), path2.as_ptr()) })
    })??;
    Errno::result(res).map(drop)
}

pub fn pivot_root<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(
    new_root: &P1,
    put_old: &P2,
) -> Result<(), Errno> {
    let res = new_root.with_nix_path(|new_root| {
        put_old.with_nix_path(|put_old| unsafe {
            // libc::SYS_pivot_root is the equivalent of C++ __NR_pivot_root used in nsjail syscall in mnt.cc
            libc::syscall(libc::SYS_pivot_root, new_root.as_ptr(), put_old.as_ptr())
        })
    })??;

    Errno::result(res).map(drop)
}

/// Close a raw file descriptor
///
/// Be aware that many Rust types implicitly close-on-drop, including
/// `std::fs::File`.  Explicitly closing them with this method too can result in
/// a double-close condition, which can cause confusing `EBADF` errors in
/// seemingly unrelated code.  Caveat programmer.  See also
/// [close(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/close.html).
///
/// # Examples
///
/// ```no_run
/// extern crate tempfile;
/// extern crate nix;
///
/// use std::os::unix::io::AsRawFd;
/// use nix::unistd::close;
///
/// fn main() {
///     let f = tempfile::tempfile().unwrap();
///     close(f.as_raw_fd()).unwrap();   // Bad!  f will also close on drop!
/// }
/// ```
///
/// ```rust
/// extern crate tempfile;
/// extern crate nix;
///
/// use std::os::unix::io::IntoRawFd;
/// use nix::unistd::close;
///
/// fn main() {
///     let f = tempfile::tempfile().unwrap();
///     close(f.into_raw_fd()).unwrap(); // Good.  into_raw_fd consumes f
/// }
/// ```
pub fn close(fd: libc::c_int) -> Result<(), errno::Errno> {
    let res = unsafe { libc::close(fd) };
    Errno::result(res).map(drop)
}

/// Read from a raw file descriptor.
///
/// See also [read(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/read.html)
pub fn read(fd: libc::c_int, buf: &mut [u8]) -> Result<usize, errno::Errno> {
    let res = unsafe {
        libc::read(
            fd,
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len() as libc::size_t,
        )
    };

    Errno::result(res).map(|r| r as usize)
}

/// Write to a raw file descriptor.
///
/// See also [write(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/write.html)
pub fn write(fd: libc::c_int, buf: &[u8]) -> Result<usize, errno::Errno> {
    let res = unsafe {
        libc::write(
            fd,
            buf.as_ptr() as *const libc::c_void,
            buf.len() as libc::size_t,
        )
    };

    Errno::result(res).map(|r| r as usize)
}
