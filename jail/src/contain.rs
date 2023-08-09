use std::ffi::CString;
use std::fs;

use super::caps;
use super::cgroupv1;
use super::config::JailConf;
use super::cpu;
use super::error::Result;
use super::mnt;
use super::net;
use super::pid;
use super::user;
use super::uts;

use sys_util::errno;
use sys_util::{syscall_temp_failure_retry, syscall_temp_failure_retry_raw};

pub static CONST_DEV_NULL: &'static [u8] = b"/dev/null\0";

pub fn contain_user_ns(jconf: &JailConf) -> Result<()> {
    user::init_ns_from_child(jconf)
}

pub fn contain_init_pid_ns(jconf: &mut JailConf) -> Result<()> {
    pid::init_ns(jconf)
}

pub fn contain_init_mount_ns(jconf: &mut JailConf) -> Result<()> {
    mnt::init_ns(jconf)
}

pub fn pool_contain_init_mount_ns(jconf: &mut JailConf, pre: bool) -> Result<()> {
    if pre {
        mnt::pre_init_ns_pool(jconf)
    } else {
        mnt::post_init_ns_pool(jconf)
    }
}

pub fn contain_init_net_ns(jconf: &JailConf) -> Result<()> {
    net::init_ns_from_child(jconf)
}

pub fn contain_init_uts_ns(jconf: &JailConf) -> Result<()> {
    uts::init_ns(jconf)
}

pub fn contain_init_cgroup_ns(jconf: &JailConf) -> Result<()> {
    cgroupv1::init_ns(jconf)
}

pub fn contain_cpu(jconf: &JailConf) -> bool {
    cpu::init_cpu(jconf)
}

pub fn contain_set_limits(jconf: &JailConf) -> Result<()> {
    // See https://man7.org/linux/man-pages/man2/setrlimit.2.html

    if jconf.disable_rl {
        return Ok(());
    }

    let mut rl = libc::rlimit64 {
        rlim_cur: jconf.rl_as,
        rlim_max: jconf.rl_as,
    };
    if unsafe { libc::setrlimit64(libc::RLIMIT_AS, &rl as *const libc::rlimit64) } == -1 {
        return Err("error when setting RLIMIT_AS".into());
    }

    rl.rlim_cur = jconf.rl_core;
    rl.rlim_max = jconf.rl_core;
    if unsafe { libc::setrlimit64(libc::RLIMIT_CORE, &rl as *const libc::rlimit64) } == -1 {
        return Err("error when setting RLIMIT_CORE".into());
    }

    rl.rlim_cur = jconf.rl_cpu;
    rl.rlim_max = jconf.rl_cpu;
    if unsafe { libc::setrlimit64(libc::RLIMIT_CPU, &rl as *const libc::rlimit64) } == -1 {
        return Err("error when setting RLIMIT_CPU".into());
    }

    rl.rlim_cur = jconf.rl_fsize;
    rl.rlim_max = jconf.rl_fsize;
    if unsafe { libc::setrlimit64(libc::RLIMIT_FSIZE, &rl as *const libc::rlimit64) } == -1 {
        return Err("error when setting RLIMIT_FSIZE".into());
    }

    rl.rlim_cur = jconf.rl_nofile;
    rl.rlim_max = jconf.rl_nofile;
    if unsafe { libc::setrlimit64(libc::RLIMIT_NOFILE, &rl as *const libc::rlimit64) } == -1 {
        return Err("error when setting RLIMIT_NOFILE".into());
    }

    rl.rlim_cur = jconf.rl_nproc;
    rl.rlim_max = jconf.rl_nproc;
    if unsafe { libc::setrlimit64(libc::RLIMIT_NPROC, &rl as *const libc::rlimit64) } == -1 {
        return Err("error when setting RLIMIT_NPROC".into());
    }

    rl.rlim_cur = jconf.rl_stack;
    rl.rlim_max = jconf.rl_stack;
    if unsafe { libc::setrlimit64(libc::RLIMIT_STACK, &rl as *const libc::rlimit64) } == -1 {
        return Err("error when setting RLIMIT_STACK".into());
    }

    Ok(())
}

pub fn contain_prepare_env(jconf: &JailConf) -> Result<()> {
    // see contain.md for explanations
    if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) } == -1 {
        return Err("could not prctl(PR_SET_PDEATHSIG, SIGKILL)".into());
    }

    // see config.md for information on personnality call
    if jconf.personality != 0
        && unsafe { libc::personality(jconf.personality as libc::c_ulong) } == -1
    {
        return Err(format!("could not set personality to {}", jconf.personality).into());
    }

    unsafe { errno::clear() }; // equivalent to C++ errno = 0; Linux does never set errno to 0, so we can set it to 0 to check if it has changed

    // see contain.md for explanations of this call
    if unsafe {
        libc::setpriority(
            libc::PRIO_PROCESS as libc::__priority_which_t,
            0,
            jconf.nice_level as libc::c_int,
        )
    } == -1
        && errno::errno() != 0
    {
        return Err(format!(
            "could not set priority from config nice level to {}",
            jconf.nice_level
        )
        .into());
    }

    // Don't call setsid(), allows for terminal signal handling in the sandboxed process. Dangerous
    if !jconf.skip_setsid {
        unsafe { libc::setsid() };
    }
    Ok(())
}

// what the fuck does nsjail if it does not succeed in contain_make_fds_coe_proc
//
// It seems nsjail does that because sometimes linux can return an error when there are too many file descriptors to pass
// and so we manually pass the first 1024 in that case. Check that it is really because of that.
pub fn contain_make_fds_coe_naive(jconf: &JailConf) -> Result<()> {
    /*
     * Don't use getrlimit(RLIMIT_NOFILE) here, as it can return an artifically small value
     * (e.g. 32), which could be smaller than a maximum assigned number to file-descriptors
     * in this process. Just use some reasonably sane value (e.g. 1024)
     */

    println!("WARNING: Naive slow way of setting CLOEXEC for process fd used");
    for fd in 0..1024 {
        let flags = syscall_temp_failure_retry_raw!(unsafe {
            libc::fcntl(fd as libc::c_int, libc::F_GETFD, 0)
        });
        if flags == -1 {
            continue;
        }
        if contain_pass_fd(jconf, fd) {
            // this fd will be passes to the child process
            if syscall_temp_failure_retry_raw!(unsafe {
                libc::fcntl(
                    fd as libc::c_int,
                    libc::F_SETFD,
                    flags & !(libc::FD_CLOEXEC),
                )
            }) == -1
            {
                // in nsjail: flags & ~(FD_CLOEXEC) to negate the FD_CLOEXEC flag bit, in rust ! is the negation, equivalent of C++ ~
                return Err(
                    "could not negate/unset fd FD_CLOEXEC flags in contain_make_fds_coe_proc"
                        .into(),
                );
            }
        } else {
            // fd will be closed before execve()
            if syscall_temp_failure_retry_raw!(unsafe {
                libc::fcntl(fd as libc::c_int, libc::F_SETFD, flags | libc::FD_CLOEXEC)
            }) == -1
            {
                return Err(
                    "could not set fd FD_CLOEXEC flags in contain_make_fds_coe_proc".into(),
                );
            }
        }
    }
    Ok(())
}

// should do this in last step juste before toaster exe not for namespace in pool
// benchmark this
// maybe check out new syscall close_range() starting kernel 5.9
pub fn contain_make_fds_coe_proc(jconf: &JailConf) -> Result<()> {
    /*
     * NSjail uses open("/proc/self/fd", O_DIRECTORY | O_RDONLY | O_CLOEXEC) then fdopendir(dirfd) then readdir(dir)
     * Let's try with rust native function for once
     * Tested in mod tests at the bottom of this file
     */
    let fds = fs::read_dir("/proc/self/fd")?;
    for direntry in fds {
        let filename = (direntry?).file_name();
        let filename = filename.to_str().unwrap();
        if filename == "." || filename == ".." {
            continue;
        }

        let fd = match filename.parse::<i32>() {
            Ok(v) => v,
            Err(e) => {
                println!(
                    "cannot convert {} to a number in contain_make_fds_coe_proc due to {:?}",
                    filename, e
                );
                continue;
            }
        };

        let flags = syscall_temp_failure_retry_raw!(unsafe {
            libc::fcntl(fd as libc::c_int, libc::F_GETFD, 0)
        });
        if flags == -1 {
            return Err("could not get fd flags in contain_make_fds_coe_proc".into());
        }

        if contain_pass_fd(jconf, fd) {
            // this fd will be passes to the child process
            if syscall_temp_failure_retry_raw!(unsafe {
                libc::fcntl(
                    fd as libc::c_int,
                    libc::F_SETFD,
                    flags & !(libc::FD_CLOEXEC),
                )
            }) == -1
            {
                // in nsjail: flags & ~(FD_CLOEXEC) to negate the FD_CLOEXEC flag bit, in rust ! is the negation, equivalent of C++ ~
                return Err(
                    "could not negate/unset fd FD_CLOEXEC flags in contain_make_fds_coe_proc"
                        .into(),
                );
            }
        } else {
            // fd will be closed before execve()
            if syscall_temp_failure_retry_raw!(unsafe {
                libc::fcntl(fd as libc::c_int, libc::F_SETFD, flags | libc::FD_CLOEXEC)
            }) == -1
            {
                return Err(
                    "could not set fd FD_CLOEXEC flags in contain_make_fds_coe_proc".into(),
                );
            }
        }
    }
    Ok(())
}

pub fn contain_make_fds_coe(jconf: &JailConf) -> bool {
    match contain_make_fds_coe_proc(jconf) {
        Ok(_) => true,
        Err(e) => {
            println!(
                "WARNING error in contain_make_fds_coe contain_make_fds_coe_proc, maybe you did not mount /proc in the new namespace: {}",
                e
            );
            match contain_make_fds_coe_naive(jconf) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
    }
}

pub fn contain_pass_fd(jconf: &JailConf, fd: i32) -> bool {
    jconf.openfds.contains(&fd)
}

pub fn contain_drop_privs(jconf: &mut JailConf) -> Result<()> {
    if !jconf.disable_no_new_privs {
        // not doing this is dangerous - default is to do it
        if unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } == -1 {
            /* Only new kernels support it */

            /*
            * See http://man7.org/linux/man-pages/man2/prctl.2.html
            * PR_SET_NO_NEW_PRIVS (since Linux 3.5)
            * Set the calling thread's no_new_privs attribute to the value
            * in arg2.  With no_new_privs set to 1, execve(2) promises not
            * to grant privileges to do anything that could not have been
            * done without the execve(2) call (for example, rendering the
            * set-user-ID and set-group-ID mode bits, and file capabilities
            * non-functional).  Once set, this the no_new_privs attribute
            * cannot be unset.  The setting of this attribute is inherited
            * by children created by fork(2) and clone(2), and preserved
            * across execve(2).

            * Since Linux 4.10, the value of a thread's no_new_privs
            * attribute can be viewed via the NoNewPrivs field in the
            * /proc/[pid]/status file.

            * For more information, see the kernel source file Documenta‐
            * tion/userspace-api/no_new_privs.rst (or Documenta‐
            * tion/prctl/no_new_privs.txt before Linux 4.13).  See also
            * seccomp(2).
            */
            return Err(errno::Errno::last().into());
        }
    }

    caps::init_ns(jconf)
}

// TODO: check this
pub fn setup_fd(jconf: &mut JailConf) -> Result<()> {
    if jconf.stderr_to_null {
        jconf.fd_err = syscall_temp_failure_retry!(unsafe {
            libc::open(CONST_DEV_NULL.as_ptr() as *const libc::c_char, libc::O_RDWR)
        })?;
    }

    if jconf.is_silent {
        jconf.fd_err = syscall_temp_failure_retry!(unsafe {
            libc::open(CONST_DEV_NULL.as_ptr() as *const libc::c_char, libc::O_RDWR)
        })?;
        jconf.fd_in = jconf.fd_err;
        jconf.fd_out = jconf.fd_err;
    }

    /* Set stdin/stdout/stderr to the net if need be. Nsjail uses this to do nothing in standalonemode or to map stdin, stdout, stderr to the socket connection in listenmode */

    /* http://man7.org/linux/man-pages/man2/dup.2.html / https://linux.die.net/man/2/dup2
        If oldfd is a valid file descriptor, and newfd has the same value
          as oldfd, then dup2() does nothing, and returns newfd.

        If the file descriptor newfd
       was previously open, it is silently closed before being reused.

        After a successful return, the old and new file descriptors may be
       used interchangeably.

        Creates a copy of the file descriptor oldfd, using the file
       descriptor number specified in newfd for the new
       descriptor.
    */

    if jconf.fd_in != libc::STDIN_FILENO {
        syscall_temp_failure_retry!(unsafe { libc::dup2(jconf.fd_in, libc::STDIN_FILENO) })?;
    }
    if jconf.fd_out != libc::STDOUT_FILENO {
        syscall_temp_failure_retry!(unsafe { libc::dup2(jconf.fd_out, libc::STDOUT_FILENO) })?;
    }
    if jconf.fd_err != libc::STDERR_FILENO {
        syscall_temp_failure_retry!(unsafe { libc::dup2(jconf.fd_err, libc::STDERR_FILENO) })?;
    }

    Ok(())
}

pub fn contain_proc(jconf: &mut JailConf) -> Result<()> {
    if jconf.create_pooled_thread {
        contain_user_ns(jconf)?;
        contain_init_net_ns(jconf).map_err(|e| format!("contain_init_net_ns: {}", e))?;
        contain_init_uts_ns(jconf).map_err(|e| format!("contain_init_uts_ns: {}", e))?;
        contain_init_cgroup_ns(jconf).map_err(|e| format!("contain_init_cgroup_ns: {}", e))?;

        pool_contain_init_mount_ns(jconf, true)
            .map_err(|e| format!("pool_contain_init_mount_ns_pre: {}", e))?;

        // blocking until this pooled thread is awaken
        contain_init_pid_ns(jconf).map_err(|e| format!("contain_init_pid_ns: {}", e))?;

        pool_contain_init_mount_ns(jconf, false)
            .map_err(|e| format!("pool_contain_init_mount_ns_post: {}", e))?;
    } else {
        contain_user_ns(jconf)?;
        contain_init_pid_ns(jconf).map_err(|e| format!("contain_init_pid_ns: {}", e))?;
        contain_init_mount_ns(jconf).map_err(|e| format!("contain_init_mount_ns: {}", e))?;
        contain_init_net_ns(jconf).map_err(|e| format!("contain_init_net_ns: {}", e))?;
        contain_init_uts_ns(jconf).map_err(|e| format!("contain_init_uts_ns: {}", e))?;
        contain_init_cgroup_ns(jconf).map_err(|e| format!("contain_init_cgroup_ns: {}", e))?;
    }

    contain_drop_privs(jconf)?;

    /* */
    /* As non-root */

    if jconf.max_cpus > 0 && !contain_cpu(jconf) {
        return Err("could not contain_cpu".into());
    }
    contain_set_limits(jconf)?;

    if jconf.prepare_env_in_child {
        // checkout what it does exactly and compare with nsjail to see if wee need to spend time doing it or not
        contain_prepare_env(jconf)?;
    }

    if jconf.handle_fds_in_child && !contain_make_fds_coe(jconf) {
        // investigate too
        return Err("could not contain_make_fds_coe".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::i32;

    #[test]
    fn test_list_proc_fd() {
        let fds = fs::read_dir("/proc/self/fd").unwrap();
        for direntry in fds {
            let fd = i32::from_str_radix((direntry.unwrap()).file_name().to_str().unwrap(), 10)
                .expect("could not parse fd in contain_make_fds_coe_proc:");
            println!("{:?}", fd);
        }
    }
}
