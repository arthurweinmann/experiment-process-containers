use super::config::JailConf;
use super::error::Result;

use sys_util::bindings::CAP_LAST_CAP;

use sys_util::errno::Errno;

/**
 * For now, we make sure to explicitly drop all capabilities. If one day you wish to retain some, see nsjail caps.cc
 * Also, we may not need to do this according to https://lwn.net/Articles/532593/ which states that when a process
 * with non-zero user IDs performs an execve(), the process's capability sets are cleared.
 * See http://man7.org/linux/man-pages/man7/capabilities.7.html
 *
 * According to tests, the capabilities are only removed for the process launched with execv from this thread
 *
 * See cpp_bindings::drop_all_caps
*/
pub fn init_ns(jconf: &JailConf) -> Result<()> {
    if jconf.keep_caps {
        if !cpp_bindings::keep_caps() {
            return Err("cpp_bindings::keep_caps".into());
        }
        return Ok(());
    }

    /*
     * Remove all capabilities from the ambient set first. It works with newer kernel versions
     * only
     */
    if unsafe {
        libc::prctl(
            libc::PR_CAP_AMBIENT,
            libc::PR_CAP_AMBIENT_CLEAR_ALL,
            0,
            0,
            0,
        )
    } == -1
    {
        return Err(Errno::last().into());
    } // Not useful according to test in our case

    /*
     * A higher-level interface layered on top of this operation is provided in the libcap(3) library in the form of cap_drop_bound(3).
     * See http://man7.org/linux/man-pages/man3/cap_drop_bound.3.html
     * According to test, this operations takes ~21.49µs
     *
     * WARNING: depending on the linux kernel version, there may be less or more capabilities, you must be sure to compile toastainer for the right target
     * or it will pose a security risk. CAP_LAST_CAP is auto-generated from <linux/capability.h> bindings on the local system.
     */
    for i in 0..CAP_LAST_CAP + 1 {
        if unsafe { libc::prctl(libc::PR_CAPBSET_DROP, i, 0, 0, 0) } == -1 {
            return Err(Errno::last().into());
        }
    }

    Ok(())
}
