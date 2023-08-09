// wrapper arround seccomp package
use super::config::JailConf;
use super::utils::read_from_fd_ignore_err;
use seccomp::SeccompFilter;

use sys_util::fcntl;

use std::ffi::CString;
use std::mem::MaybeUninit; // This new feature may soon be stable, see if it should replace maybeunit: https://doc.rust-lang.org/std/boxed/struct.Box.html#method.new_uninit
use std::str::from_utf8;

pub fn apply_policy(jconf: &mut JailConf) -> bool {
    let filter = jconf.seccomp_filter.take(); // we consume/move the seccomp filter out of jconf, so after this you can no longer use it but you should not need to

    if let Some(filter) = filter {
        match SeccompFilter::apply(filter) {
            Ok(_) => {}
            Err(_) => return false,
        }
    }

    true
}

pub fn which_seccomp_violation(pid: libc::pid_t, si: &libc::siginfo_t) -> String {
    let pid_syscall_fd = match fcntl::open_no_mode(
        &format!("/proc/{}/syscall", pid),
        libc::O_RDONLY | libc::O_CLOEXEC,
    ) {
        Ok(v) => v,
        Err(_) => {
            return format!("Seccomp Violation: pid={} SiCode: {}, SiErrno: {}, SiSigno: {} <> could not open pid_syscall_fd", pid, si.si_code, si.si_errno, si.si_signo);
        }
    };

    let mut buf = [0; 4096];
    let rdsize = read_from_fd_ignore_err(pid_syscall_fd, &mut buf[..4095]).expect("");

    if rdsize < 1 {
        unsafe { libc::close(pid_syscall_fd) };
        return format!("Seccomp Violation: pid={} SiCode: {}, SiErrno: {}, SiSigno: {} <> could not open pid_syscall_fd", pid, si.si_code, si.si_errno, si.si_signo);
    }

    buf[4095] = '\0' as u8;

    let mut arg1 = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut arg2 = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut arg3 = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut arg4 = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut arg5 = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut arg6 = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut sp = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut pc = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut sc = MaybeUninit::<libc::ptrdiff_t>::uninit();

    let ret = unsafe {
        libc::sscanf(
            buf.as_ptr() as *const libc::c_char,
            CString::new("%td %tx %tx %tx %tx %tx %tx %tx %tx")
                .unwrap()
                .as_ptr() as *const libc::c_char,
            sc.as_mut_ptr(),
            arg1.as_mut_ptr(),
            arg2.as_mut_ptr(),
            arg3.as_mut_ptr(),
            arg4.as_mut_ptr(),
            arg5.as_mut_ptr(),
            arg6.as_mut_ptr(),
            sp.as_mut_ptr(),
            pc.as_mut_ptr(),
        )
    };

    unsafe { libc::close(pid_syscall_fd) };

    if ret == 9 {
        let arg1 = unsafe { arg1.assume_init() };
        let arg2 = unsafe { arg2.assume_init() };
        let arg3 = unsafe { arg3.assume_init() };
        let arg4 = unsafe { arg4.assume_init() };
        let arg5 = unsafe { arg5.assume_init() };
        let arg6 = unsafe { arg6.assume_init() };
        let sp = unsafe { sp.assume_init() };
        let pc = unsafe { pc.assume_init() };
        let sc = unsafe { sc.assume_init() };

        return format!("Seccomp Violation: pid={} SiCode: {}, SiErrno: {}, SiSigno: {} <> Syscall number: {}, Arguments: [{}, {}, {}, {}, {}, {}], SP: {}, PC: {},", pid, si.si_code, si.si_errno, si.si_signo, sc, arg1, arg2, arg3, arg4, arg5, arg6, sp, pc);
    } else if ret == 3 {
        let arg1 = unsafe { arg1.assume_init() };
        let arg2 = unsafe { arg2.assume_init() };

        return format!(
            "Seccomp Violation: pid={} SiCode: {}, SiErrno: {}, SiSigno: {} <> SP: {}, PC: {},",
            pid, si.si_code, si.si_errno, si.si_signo, arg1, arg2
        );
    }

    let buf_str = from_utf8(&buf).ok();
    if let Some(buf_str) = buf_str {
        return format!(
            "Seccomp Violation: pid={} SiCode: {}, SiErrno: {}, SiSigno: {} <> Syscall string: {}",
            pid, si.si_code, si.si_errno, si.si_signo, buf_str
        );
    } else {
        return format!("Seccomp Violation: pid={} SiCode: {}, SiErrno: {}, SiSigno: {} <> Syscall string: could not decode buf from /proc/pid/syscall as utf-8", pid, si.si_code, si.si_errno, si.si_signo);
    }
}

pub fn which_seccomp_violation_pid_only(pid: libc::pid_t) -> String {
    let pid_syscall_fd = match fcntl::open_no_mode(
        &format!("/proc/{}/syscall", pid),
        libc::O_RDONLY | libc::O_CLOEXEC,
    ) {
        Ok(v) => v,
        Err(_) => {
            return format!(
                "Seccomp Violation: pid={} <> could not open pid_syscall_fd",
                pid
            );
        }
    };

    let mut buf = [0; 4096];
    let rdsize = read_from_fd_ignore_err(pid_syscall_fd, &mut buf[..4095]).unwrap();

    if rdsize < 1 {
        unsafe { libc::close(pid_syscall_fd) };
        return format!(
            "Seccomp Violation: pid={}  <> could not open pid_syscall_fd",
            pid
        );
    }

    buf[4095] = '\0' as u8;

    let mut arg1 = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut arg2 = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut arg3 = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut arg4 = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut arg5 = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut arg6 = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut sp = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut pc = MaybeUninit::<libc::uintptr_t>::uninit();
    let mut sc = MaybeUninit::<libc::ptrdiff_t>::uninit();

    let ret = unsafe {
        libc::sscanf(
            buf.as_ptr() as *const libc::c_char,
            CString::new("%td %tx %tx %tx %tx %tx %tx %tx %tx")
                .unwrap()
                .as_ptr() as *const libc::c_char,
            sc.as_mut_ptr(),
            arg1.as_mut_ptr(),
            arg2.as_mut_ptr(),
            arg3.as_mut_ptr(),
            arg4.as_mut_ptr(),
            arg5.as_mut_ptr(),
            arg6.as_mut_ptr(),
            sp.as_mut_ptr(),
            pc.as_mut_ptr(),
        )
    };

    unsafe { libc::close(pid_syscall_fd) };

    if ret == 9 {
        let arg1 = unsafe { arg1.assume_init() };
        let arg2 = unsafe { arg2.assume_init() };
        let arg3 = unsafe { arg3.assume_init() };
        let arg4 = unsafe { arg4.assume_init() };
        let arg5 = unsafe { arg5.assume_init() };
        let arg6 = unsafe { arg6.assume_init() };
        let sp = unsafe { sp.assume_init() };
        let pc = unsafe { pc.assume_init() };
        let sc = unsafe { sc.assume_init() };

        return format!("Seccomp Violation: pid={}  <> Syscall number: {}, Arguments: [{}, {}, {}, {}, {}, {}], SP: {}, PC: {},", pid,  sc, arg1, arg2, arg3, arg4, arg5, arg6, sp, pc);
    } else if ret == 3 {
        let arg1 = unsafe { arg1.assume_init() };
        let arg2 = unsafe { arg2.assume_init() };

        return format!(
            "Seccomp Violation: pid={}  <> SP: {}, PC: {},",
            pid, arg1, arg2
        );
    }

    let buf_str = from_utf8(&buf).ok();
    if let Some(buf_str) = buf_str {
        return format!(
            "Seccomp Violation: pid={} <> Syscall string: {}",
            pid, buf_str
        );
    } else {
        return format!("Seccomp Violation: pid={}  <> Syscall string: could not decode buf from /proc/pid/syscall as utf-8", pid);
    }
}
