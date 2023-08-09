use libc::c_char;

use std::ffi::{CStr, CString};
use std::ptr;

// traits
use std::ops::Deref;

use super::errno::Errno;

fn to_exec_array<T: AsRef<CStr>>(args: &[T]) -> Vec<*const c_char> {
    use std::iter::once;
    args.iter()
        .map(|s| s.as_ref().as_ptr())
        .chain(once(ptr::null()))
        .collect()
}

/// Replace the current process image with a new one (see
/// [exec(3)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/exec.html)).
///
/// See the `::nix::unistd::execve` system call for additional details.  `execv`
/// performs the same action but does not allow for customization of the
/// environment for the new process.
#[inline]
pub fn execv<T: Deref<Target = CStr>, U: AsRef<CStr>>(path: T, argv: &[U]) -> Errno {
    let args_p = to_exec_array(argv);

    unsafe { libc::execv(path.as_ptr(), args_p.as_ptr()) };

    Errno::last()
}

/// Replace the current process image with a new one (see
/// [execve(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/exec.html)).
///
/// The execve system call allows for another process to be "called" which will
/// replace the current process image.  That is, this process becomes the new
/// command that is run. On success, this function will not return. Instead,
/// the new program will run until it exits.
///
/// `::nix::unistd::execv` and `::nix::unistd::execve` take as arguments a slice
/// of `::std::ffi::CString`s for `args` and `env` (for `execve`). Each element
/// in the `args` list is an argument to the new process. Each element in the
/// `env` list should be a string in the form "key=value".
#[inline]
pub fn execve(path: &CStr, args: &[&CStr], env: &[&CStr]) -> Result<(), Errno> {
    let args_p = to_exec_array(args);
    let env_p = to_exec_array(env);

    unsafe { libc::execve(path.as_ptr(), args_p.as_ptr(), env_p.as_ptr()) };

    Err(Errno::last())
}

/// Replace the current process image with a new one and replicate shell `PATH`
/// searching behavior (see
/// [exec(3)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/exec.html)).
///
/// See `::nix::unistd::execve` for additional details.  `execvp` behaves the
/// same as execv except that it will examine the `PATH` environment variables
/// for file names not specified with a leading slash.  For example, `execv`
/// would not work if "bash" was specified for the path argument, but `execvp`
/// would assuming that a bash executable was on the system `PATH`.
#[inline]
pub fn execvp(filename: &CStr, args: &[&CStr]) -> Result<(), Errno> {
    let args_p = to_exec_array(args);

    unsafe { libc::execvp(filename.as_ptr(), args_p.as_ptr()) };

    Err(Errno::last())
}

/// Replace the current process image with a new one and replicate shell `PATH`
/// searching behavior (see
/// [`execvpe(3)`](http://man7.org/linux/man-pages/man3/exec.3.html)).
///
/// This functions like a combination of `execvp(2)` and `execve(2)` to pass an
/// environment and have a search path. See these two for additional
/// information.
#[cfg(any(target_os = "haiku", target_os = "linux", target_os = "openbsd"))]
pub fn execvpe(filename: &CStr, args: &[&CStr], env: &[&CStr]) -> Result<(), Errno> {
    let args_p = to_exec_array(args);
    let env_p = to_exec_array(env);

    unsafe { libc::execvpe(filename.as_ptr(), args_p.as_ptr(), env_p.as_ptr()) };

    Err(Errno::last())
}

/// Replace the current process image with a new one (see
/// [fexecve(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/fexecve.html)).
///
/// The `fexecve` function allows for another process to be "called" which will
/// replace the current process image.  That is, this process becomes the new
/// command that is run. On success, this function will not return. Instead,
/// the new program will run until it exits.
///
/// This function is similar to `execve`, except that the program to be executed
/// is referenced as a file descriptor instead of a path.
// Note for NetBSD and OpenBSD: although rust-lang/libc includes it (under
// unix/bsd/netbsdlike/) fexecve is not currently implemented on NetBSD nor on
// OpenBSD.
#[cfg(any(target_os = "android", target_os = "linux", target_os = "freebsd"))]
#[inline]
pub fn fexecve(fd: libc::c_int, args: &[&CStr], env: &[&CStr]) -> Result<(), Errno> {
    let args_p = to_exec_array(args);
    let env_p = to_exec_array(env);

    unsafe { libc::fexecve(fd, args_p.as_ptr(), env_p.as_ptr()) };

    Err(Errno::last())
}

/// Execute program relative to a directory file descriptor (see
/// [execveat(2)](http://man7.org/linux/man-pages/man2/execveat.2.html)).
///
/// The `execveat` function allows for another process to be "called" which will
/// replace the current process image.  That is, this process becomes the new
/// command that is run. On success, this function will not return. Instead,
/// the new program will run until it exits.
///
/// This function is similar to `execve`, except that the program to be executed
/// is referenced as a file descriptor to the base directory plus a path.
#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub fn execveat<T: Deref<Target = CStr>, U: AsRef<CStr>>(
    dirfd: libc::c_int,
    pathname: Option<T>,
    args: &[U],
    env: &[U],
    flags: libc::c_int,
) -> Errno {
    let args_p = to_exec_array(args);
    let env_p = to_exec_array(env);

    if let Some(pathname) = pathname {
        unsafe {
            libc::syscall(
                libc::SYS_execveat,
                dirfd,
                pathname.as_ptr(),
                args_p.as_ptr(),
                env_p.as_ptr(),
                flags,
            );
        };
    } else {
        unsafe {
            libc::syscall(
                libc::SYS_execveat,
                dirfd,
                CString::new("").unwrap().as_ptr(), // TODO: see a better and more performant way of passing "", this operation is ~ 29.263 ns according to benchmark
                args_p.as_ptr(),
                env_p.as_ptr(),
                flags,
            );
        };
    }

    Errno::last()
}
