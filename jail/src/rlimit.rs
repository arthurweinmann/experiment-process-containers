use super::error::Result;
use std::mem::MaybeUninit;

use sys_util::errno::Errno;

pub const RLIM64_INFINITY: u64 = 0xffffffffffffffff;

/// res would for example be libc::RLIMIT_NOFILE
pub fn get_rlimit64(res: u32) -> Result<libc::rlimit64> {
    let mut rl = libc::rlimit64 {
        rlim_cur: 0,
        rlim_max: 0,
    };

    if unsafe { libc::getrlimit64(res, &mut rl as *mut libc::rlimit64) } < 0 {
        return Err(format!("get_rlimit64: {}", Errno::last()).into());
    }

    Ok(rl)
}
