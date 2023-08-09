use std::ffi::CStr;

extern "C" {
    // syscall number
    fn getResGIDSyscallNb() -> libc::c_long;
    fn setResUIDSyscallNb() -> libc::c_long;

    // libnl
    fn libnlRtmSetLink() -> libc::c_int;
    fn libnlNlmFCreate() -> libc::c_int;

    // nsjail net
    fn ifaceConfig(
        iface: *const libc::c_char,
        ip: *const libc::c_char,
        mask: *const libc::c_char,
        gw: *const libc::c_char,
    ) -> bool;

    fn ifaceUp(ifacename: *const libc::c_char) -> libc::c_int;

    fn initCpu(num_cpus: libc::c_int, max_cpus: libc::c_int) -> libc::c_int;

    fn reapProc(printSeccompViolation: libc::c_int) -> libc::c_int;

    fn handleCaps(keepCaps: libc::c_int) -> libc::c_int;
    fn keepCaps() -> libc::c_int;
}

pub fn keep_caps() -> bool {
    if unsafe{keepCaps()} == 1 {
        return true;
    }
    false
}

pub fn init_cap_ns(keep_caps: bool) -> bool {
    if keep_caps {
        if unsafe{handleCaps(1)} == 1 {
            return true;
        }
    } else {
        if unsafe{handleCaps(0)} == 1 {
            return true;
        }
    }
    
    false
}

pub fn get_set_res_gid_syscall_number() -> libc::c_long {
    unsafe { getResGIDSyscallNb() }
}

pub fn get_set_res_uid_syscall_number() -> libc::c_long {
    unsafe { setResUIDSyscallNb() }
}

pub fn get_libnl_rtm_setlink() -> libc::c_int {
    unsafe { libnlRtmSetLink() }
}

pub fn get_libnl_nlm_f_create() -> libc::c_int {
    unsafe { libnlNlmFCreate() }
}

pub fn iface_config(iface: &CStr, ip: &CStr, mask: &CStr, gw: &CStr) -> bool {
    unsafe { ifaceConfig(iface.as_ptr(), ip.as_ptr(), mask.as_ptr(), gw.as_ptr()) }
}

pub fn iface_up(ifacename: &CStr) -> bool {
    if unsafe { ifaceUp(ifacename.as_ptr()) } == 1 {
        return true;
    }
    false
}

pub fn init_cpu(num_cpus: libc::c_int, max_cpus: libc::c_int) -> bool {
    if unsafe{ initCpu(num_cpus, max_cpus) } == 1 {
        return true;
    }

    false
}

pub fn reap_proc(print_seccomp_violation: bool) -> libc::c_int {
    // for now, rust has not a stable way to pass booleans to C
    if print_seccomp_violation {
        unsafe{ reapProc(1) }
    } else {
        unsafe{ reapProc(0) }
    }
}

#[cfg(test)]
mod test {
    use std::ffi::CString;
    use super::*;

    extern "C" {
        fn testCCharToStdString(iface: *const libc::c_char) -> bool;
    }

    #[test]
    fn test_iface() {
        let iface23 = CString::new("iface23".as_bytes()).unwrap();
        println!("{}", unsafe {
            testCCharToStdString(
                iface23.as_ptr(),
            )
        });
    }
}
