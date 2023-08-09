// extern crate sys_util;
// extern crate libc; https://doc.rust-lang.org/stable/edition-guide/rust-2018/module-system/path-clarity.html

use std::ffi::{CStr, CString};
use std::fs;
use std::mem::MaybeUninit;
use std::path::Path;
use std::{fs::File, io::Write, os::unix::io::FromRawFd};
use sys_util::{
    errno, errno::Errno, syscall_temp_failure_retry, syscall_temp_failure_retry_raw,
    SyscallReturnCode,
};
use utils::filepath::path_to_bytes;

pub fn read_fd_wrapper(fd: libc::c_int, expected: char) -> super::error::Result<()> {
    let mut buf = [0; 1];
    if read_from_fd_ignore_err(fd, &mut buf)? != 1 {
        return Err("could not read from fd".into());
    }
    if buf[0] as char != expected {
        return Err("read invalid char from fd".into());
    }

    Ok(())
}

pub fn write_fd_wrapper(fd: libc::c_int, carac: char) -> super::error::Result<()> {
    let bsl = vec![carac as u8];
    if !write_to_fd(fd, bsl.as_slice()) {
        return Err("Couldn't write to fd".into());
    }

    Ok(())
}

// does this copy or move the string created inside the function ?
// does move mean in rust copied with trait copy ?
/* From rust compiler
note: move occurs because `val` has type `std::string::String`, which does
not implement the `Copy` trait
*/
pub fn get_folder_from_pid(pid: &str) -> String {
    let mut subfolder = "TOASTAINER.".to_string();
    subfolder.push_str(pid);
    subfolder
}

pub fn mkdir<T: AsRef<Path>>(path: &T, mode: libc::mode_t) -> Result<(), std::io::Error> {
    // let dir = CStr::from_bytes_with_nul(path).unwrap();

    // https://stackoverflow.com/questions/38948669/whats-the-most-direct-way-to-convert-a-path-to-a-c-char
    let dir = CString::new(path_to_bytes(path.as_ref())).unwrap(); // we use CString::new to create a new string terminated by a null byte \0

    // The call is safe because we provide valid arguments.
    // mode libc::S_IRUSR | libc::S_IWUSR was the mode for temp pivot root directory from firecracker, check that
    SyscallReturnCode(unsafe { libc::mkdir(dir.as_ptr(), mode) }).into_empty_result()?;
    // .map_err(|e| format!("Failed to create dir: {}", e))

    Ok(())
}

pub fn is_dir<T: AsRef<Path>>(path: &T) -> bool {
    /*
     * In nsjail, If the source dir is NULL, it assumes it's a dir (for /proc and tmpfs) -> not implemented here but check that it is ok
     * nsjail uses stat(path, &st) with struct stat st and then (S_ISDIR(st.st_mode)) -> check if rust metadata gives the same results
     */

    // this would be the Rust idiomatic way
    // match fs::metadata(path) {
    //     Ok(m) => m.is_dir(),
    //     Err(_) => false,
    // }

    let dir = CString::new(path_to_bytes(path.as_ref())).unwrap();

    // https://doc.rust-lang.org/std/mem/union.MaybeUninit.html
    // This new feature may soon be stable, see if it should replace maybeunit: https://doc.rust-lang.org/std/boxed/struct.Box.html#method.new_uninit
    // Create an explicitly uninitialized struct. The compiler knows that data inside
    // a `MaybeUninit<T>` may be invalid, and hence this is not UB (undefined behavior):
    let mut st = MaybeUninit::<libc::stat>::uninit();

    // Set it to a valid value.
    match SyscallReturnCode(unsafe { libc::stat(dir.as_ptr(), st.as_mut_ptr()) })
        .into_empty_result()
    {
        Err(_) => return false,
        _ => (),
    }

    // Extract the initialized data -- this is only allowed *after* properly
    // initializing `x`!
    // shadowing
    let st = unsafe { st.assume_init() };

    // see http://man7.org/linux/man-pages/man7/inode.7.html
    /*
    * S_IFMT     0170000   bit mask for the file type bit field

    * S_IFSOCK   0140000   socket
    * S_IFLNK    0120000   symbolic link
    * S_IFREG    0100000   regular file
    * S_IFBLK    0060000   block device
    * S_IFDIR    0040000   directory
    * S_IFCHR    0020000   character device
    * S_IFIFO    0010000   FIFO
    */
    if (st.st_mode & libc::S_IFMT) == libc::S_IFDIR {
        return true;
    }

    false
}

// It's called writeln_special because we have to use this rather convoluted way of writing
// to special cgroup files, to avoid getting errors. It would be nice to know why that happens :-s
// firecracker function
pub fn writeln_special<T, V>(file_path: &T, value: V) -> Result<(), String>
where
    T: AsRef<Path>,
    V: ::std::fmt::Display,
{
    fs::write(file_path, format!("{}\n", value)).map_err(|e| format!("failed to write; {}", e))
}

pub fn readln_special<T: AsRef<Path>>(file_path: &T) -> Result<String, std::io::Error> {
    let mut line = fs::read_to_string(file_path)?; // ? return err from this function if there is one, otherwise put Ok val into line

    // Remove the newline character at the end (if any).
    line.pop();

    Ok(line)
}

// le writeBufToFile de nsjail gère mieux les perms que le writeln_special de firecracker ; tester les deux voire ce que ca donne

// nsjail transcripted function
pub fn write_buf_to_file<T: AsRef<Path>>(
    path: &T,
    open_flags: libc::c_int,
    buf: &[u8],
) -> Result<(), std::io::Error> {
    // let filename = CStr::from_bytes_with_nul(path).unwrap();
    let filename = CString::new(path_to_bytes(path.as_ref())).unwrap(); // we use CString::new to create a new string terminated by a null byte \0

    let err: std::io::Error; // does not need to be mutable, why ? because assigned only later ?

    {
        // let fd = SyscallReturnCode(unsafe { libc::open(filename.as_ptr(), open_flags, 0644) }) // check that 0644 is correctly applied
        //     .into_result()?;

        let fd = match syscall_temp_failure_retry!(unsafe {
            libc::open(filename.as_ptr(), open_flags, 0644)
        }) {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        };

        let mut f = unsafe { File::from_raw_fd(fd) };

        match f.write_all(buf) {
            Ok(_) => return Ok(()),
            Err(e) => {
                err = e;
                // drop(f);
            }
        }
    } // used scope not to need to drop(f); before deleting it in case of an error

    // f is closed here.
    // Is fd also closed by f destructor or do we need to do it manually
    // since it is called inside an unsafe Block ??? Test it !!!

    if open_flags & libc::O_CREAT != 0 {
        unsafe { libc::unlink(filename.as_ptr()) };
    }

    Err(err.into())
}

pub fn write_to_fd(fd: libc::c_int, buf: &[u8]) -> bool {
    let mut written_sz: usize = 0;
    while written_sz < buf.len() {
        let sz = match syscall_temp_failure_retry!(unsafe {
            libc::write(
                fd,
                buf[written_sz..].as_ptr() as *const libc::c_void,
                buf.len() - written_sz as libc::size_t,
            )
        }) {
            Ok(v) => v,
            Err(_) => return false,
        };
        if sz < 0 {
            return false;
        }
        written_sz += sz as usize;
    }

    true
}

pub fn read_from_fd_ignore_err(
    fd: libc::c_int,
    buf: &mut [u8],
) -> Result<usize, super::error::Error> {
    let mut read_sz: usize = 0;
    while read_sz < buf.len() {
        let sz = unsafe {
            // syscall_temp_failure_retry_raw!(
            libc::read(
                fd,
                buf.as_ptr() as *mut libc::c_void,
                buf.len() as libc::size_t,
            )
        };

        // println!("read_from_fd_ignore_err: fd:{} {}:{}", fd, sz, buf.len());

        if sz <= 0 {
            if sz < 0 {
                let err = Errno::last();
                return Err(format!("read_from_fd_ignore_err: {}", err).into());
            }

            // EOF - happens when execve succeeds and close the fd for example
            break;
        }
        read_sz += sz as usize;
    }
    Ok(read_sz)
}

pub fn write_message_to_fd(fd: libc::c_int, buf: &[u8]) -> Result<(), super::error::Error> {
    let mut written_sz: usize = 0;
    while written_sz < buf.len() {
        let sz = unsafe {
            libc::write(
                fd,
                buf[written_sz..].as_ptr() as *const libc::c_void,
                buf.len() - written_sz as libc::size_t,
            )
        };

        if sz < 0 {
            return Err(format!("write_message_to_fd: {}", Errno::last()).into());
        }

        if sz == 0 {
            return Err(super::error::Error::EOF);
        }

        written_sz += sz as usize;
    }

    Ok(())
}

/// iniCap should be 1024
pub fn read_message_from_fd(
    fd: libc::c_int,
    ini_cap: usize,
) -> Result<Vec<u8>, super::error::Error> {
    let mut buf: Vec<u8> = Vec::with_capacity(ini_cap);
    unsafe { buf.set_len(2) };

    read_message_syscalls(fd, 2, &buf)?;

    let lmess = ((buf[1] as u16) | ((buf[0] as u16) << 8)) as usize;

    if lmess > 512 {
        buf.reserve_exact(lmess - 2);
    }
    unsafe { buf.set_len(lmess) };

    read_message_syscalls(fd, lmess, &buf)?;

    Ok(buf)
}

pub fn read_message_syscalls(
    fd: libc::c_int,
    lmess: usize,
    buf: &[u8],
) -> Result<(), super::error::Error> {
    let mut read_sz: usize = 0;
    while read_sz < lmess {
        let sz = unsafe {
            libc::read(
                fd,
                buf[read_sz..lmess].as_ptr() as *mut libc::c_void,
                buf.len() as libc::size_t,
            )
        };
        if sz <= 0 {
            if sz < 0 {
                let err = Errno::last();
                return Err(format!("read_from_fd_ignore_err: {}", err).into());
            }

            return Err(super::error::Error::EOF);
        }
        read_sz += sz as usize;
    }
    Ok(())
}

/// Read from a raw file descriptor.
///
/// See also [read(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/read.html)
pub fn read_from_fd(fd: libc::c_int, buf: &mut [u8]) -> Result<usize, sys_util::errno::Errno> {
    let res = unsafe {
        libc::read(
            fd,
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len() as libc::size_t,
        )
    };

    sys_util::errno::Errno::result(res).map(|r| r as usize)
}

pub fn to_exec_array(args: &[&CStr]) -> Vec<*const libc::c_char> {
    use std::iter::once;
    args.iter()
        .map(|s| s.as_ptr())
        .chain(once(std::ptr::null()))
        .collect()
}

pub fn to_exec_array_cstring(args: &[CString]) -> Vec<*const libc::c_char> {
    use std::iter::once;
    args.iter()
        .map(|s| s.as_c_str().as_ptr())
        .chain(once(std::ptr::null()))
        .collect()
}

// https://users.rust-lang.org/t/how-to-allocate-huge-byte-array-safely/18284/12
// https://github.com/rust-lang/rfcs/blob/master/text/2116-alloc-me-maybe.md
// fn allocate_byte_array(len: usize) -> Result<Vec<u8>, CollectionAllocErr> {
//     let mut v = Vec::new();
//     v.try_reserve(len)?;
// 	unsafe {
// 	    v.set_len(len);
// 	}
// 	Ok(v)
// }
