use std::ffi::{OsStr, CStr};
use std::os::unix::ffi::OsStrExt;
use super::SyscallReturnCode;
use std::mem;
use std::fmt;
use std::str::from_utf8_unchecked;

// #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/*
* So while repr(C) happens to do the right thing with respect to memory layout, 
* it's not quite the right tool for newtypes in FFI. Instead of declaring a C struct, 
* we need to communicate to the Rust compiler that our newtype is just for type safety 
* on the Rust side. This is what repr(transparent) does.

* The attribute can be applied to a newtype-like structs that contains a single field. 
* It indicates that the newtype should be represented exactly like that field's type, i.e., 
* the newtype should be ignored for ABI purpopses: not only is it laid out the same in memory, 
* it is also passed identically in function calls.
*/
// #[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct UtsName(libc::utsname);

impl fmt::Debug for UtsName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UtsName {{ sysname: {}, nodename: {}, machine: {} }}", self.sysname(), self.nodename(), self.machine())
    }
}

impl UtsName {
    pub fn sysname(&self) -> &str {
        to_str(&(&self.0.sysname as *const libc::c_char ) as *const *const libc::c_char)
    }

    pub fn nodename(&self) -> &str {
        to_str(&(&self.0.nodename as *const libc::c_char ) as *const *const libc::c_char)
    }

    pub fn release(&self) -> &str {
        to_str(&(&self.0.release as *const libc::c_char ) as *const *const libc::c_char)
    }

    pub fn version(&self) -> &str {
        to_str(&(&self.0.version as *const libc::c_char ) as *const *const libc::c_char)
    }

    pub fn machine(&self) -> &str {
        to_str(&(&self.0.machine as *const libc::c_char ) as *const *const libc::c_char)
    }
}

#[inline]
fn to_str<'a>(s: *const *const libc::c_char) -> &'a str {
    unsafe {
        let res = CStr::from_ptr(*s).to_bytes();
        from_utf8_unchecked(res)
    }
}

pub fn uname() -> UtsName {
    unsafe {
        let mut ret = mem::MaybeUninit::uninit();
        libc::uname(ret.as_mut_ptr());
        UtsName(ret.assume_init())
    }
}

/// Set the system host name (see
/// [sethostname(2)](http://man7.org/linux/man-pages/man2/gethostname.2.html)).
///
/// Given a name, attempt to update the system host name to the given string.
/// On some systems, the host name is limited to as few as 64 bytes.  An error
/// will be return if the name is not valid or the current process does not have
/// permissions to update the host name.
pub fn sethostname<S: AsRef<OsStr>>(name: S) -> Result<(), std::io::Error> {
    // Handle some differences in type of the len arg across platforms.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
    ))]
    type SethostnameLenT = libc::c_int;

    #[cfg(not(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
    )))]
    type SethostnameLenT = libc::size_t;

    let ptr = name.as_ref().as_bytes().as_ptr() as *const libc::c_char;
    let len = name.as_ref().len() as SethostnameLenT;

    SyscallReturnCode(unsafe { libc::sethostname(ptr, len) }).into_empty_result()
}

#[cfg(test)]
mod test {
    #[cfg(target_os = "linux")]
    #[test]
    pub fn test_uname_linux() {
        assert_eq!(super::uname().sysname(), "Linux");
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[test]
    pub fn test_uname_darwin() {
        assert_eq!(super::uname().sysname(), "Darwin");
    }

    #[cfg(target_os = "freebsd")]
    #[test]
    pub fn test_uname_freebsd() {
        assert_eq!(super::uname().sysname(), "FreeBSD");
    }
}
