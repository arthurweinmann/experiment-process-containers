use std::mem;
use super::errno::Errno;


// TODO: see how to implement all calls from nsjail:cpu.cc which is not the case right now, (for example libc lacks CPU_ALLOC and CPU_ALLOC_SIZE)
// For now it is implemented through cpp_bindings package

pub fn unshare(flags: libc::c_int) -> Result<(), Errno> {
    let res = unsafe { libc::unshare(flags) };

    Errno::result(res).map(drop)
}

pub fn setns(fd: libc::c_int, nstype: libc::c_int) -> Result<(), Errno> {
    let res = unsafe { libc::setns(fd, nstype) };

    Errno::result(res).map(drop)
}


#[repr(C)]
// #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CpuSet {
    cpu_set: libc::cpu_set_t,
}

impl CpuSet {
    pub fn new() -> CpuSet {
        CpuSet {
            cpu_set: unsafe { mem::zeroed() },
        }
    }

    pub fn is_set(&self, field: usize) -> Result<bool, Errno> {
        if field >= 8 * mem::size_of::<libc::cpu_set_t>() {
            Err(Errno::EINVAL)
        } else {
            Ok(unsafe { libc::CPU_ISSET(field, &self.cpu_set) })
        }
    }

    pub fn set(&mut self, field: usize) -> Result<(), Errno> {
        if field >= 8 * mem::size_of::<libc::cpu_set_t>() {
            Err(Errno::EINVAL)
        } else {
            Ok(unsafe { libc::CPU_SET(field, &mut self.cpu_set) })
        }
    }

    pub fn unset(&mut self, field: usize) -> Result<(), Errno> {
        if field >= 8 * mem::size_of::<libc::cpu_set_t>() {
            Err(Errno::EINVAL)
        } else {
            Ok(unsafe { libc::CPU_CLR(field, &mut self.cpu_set) })
        }
    }
}

pub fn sched_setaffinity(pid: libc::pid_t, cpuset: &CpuSet) -> Result<(), Errno> {
    let res = unsafe {
        libc::sched_setaffinity(
            pid,
            mem::size_of::<CpuSet>() as libc::size_t,
            &cpuset.cpu_set,
        )
    };

    Errno::result(res).map(drop)
}
