use super::config::{JailConf, NSSIGS, PIDT};
use super::contain::{self, setup_fd};
use super::error::{Error, Result};
use super::protobuf::create_pooled_wake_up_mess;
use super::utils::{
    read_from_fd_ignore_err, to_exec_array, to_exec_array_cstring, write_message_to_fd, write_to_fd,
};
use super::{cgroupv1, cgroupv2, net, sandbox, user};

use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::ptr;
use std::time::SystemTime;

use sys_util::{errno::Errno, execv, fcntl, socket, unistd, SyscallReturnCode};

pub const STACK_SIZE: usize = 128 * 1024;

// See https://git2.toastate.io/fork/nsjail/-/commit/04e5fae0e3e0efd3e68a11e2d5b5e3e1a42811ac and https://patchwork.ozlabs.org/project/glibc/patch/alpine.DEB.2.21.1908052105150.25360@digraph.polyomino.org.uk/
const CLONE_PIDFD: i32 = 0x00001000;

/**
 * When you want to run a container and you do not care it it blocks the current thread while waiting for it to finish
 * In case you do care about performance, use future epoll with pidfd executor instead
 *
 * We force jconf.clone_newpid to be true so that we are sure that waiting for first child to finish is enough to be sure
 * that all other child (in this pid namespace) will also be terminated
 */
pub fn run_monitor_child(
    jconf: &mut JailConf,
    callback: extern "C" fn(*mut libc::c_void) -> libc::c_int,
) -> Result<i32> {
    jconf.clone_newpid();

    let child_pid = run_child(jconf, callback)?;

    // it is also while waiting that we must check jconf.tlimit, e.g. container max execution time if one was specified

    // we need to wait for all child and child's child to terminate

    // let exit_status = wait::reap_proc(true); -> apparemment ne marche pas ; ne wait pas correctement -> cf check dans nsjail il y a une boucle for en plus de celles qu'on fait wui tourne tant qu'il y a des process
    let mut wait_status: i32 = 0;

    if unsafe { libc::waitpid(child_pid, &mut wait_status as *mut libc::c_int, 0) } != child_pid {
        return Err(format!(
            "Error waiting the child process to finish: {}",
            Errno::last()
        )
        .into());
    }

    let mut status = 0;
    if unsafe { libc::WIFEXITED(wait_status) } {
        status = unsafe { libc::WEXITSTATUS(wait_status) };
    }

    clean_after_child(jconf, child_pid)?;

    // Ok(exit_status)
    Ok(status)
}

/// clean_after_child does not call wait syscall, do not forget to wait child_pid to avoid for it to become a zombie process
pub fn clean_after_child(jconf: &JailConf, child_pid: i32) -> Result<()> {
    if jconf.clone_newcgroup {
        // only namespace not to clean itself when no more process in it ?
        // so we may not need to keep him open with an opened fd for other container to run in the same namespace ? TODO: test this

        if jconf.use_cgroupv2 {
            cgroupv2::finish_from_parent(jconf, child_pid.to_string().as_str())?;
        } else {
            cgroupv1::finish_from_parent(jconf, child_pid.to_string().as_str())?;
        }
    }

    // if let Some(nms) = jconf.join_nms {
    //     println!("clean_after_child: closing pooled nms fds");
    //     for i in 0..nms.len() {
    //         unsafe { libc::close(nms[i]) };
    //     }
    // }

    Ok(())
}

/// do not forget to wait for the child pid after this call returns and then to run clean_after_child
pub fn run_child(
    jconf: &mut JailConf,
    callback: extern "C" fn(*mut libc::c_void) -> libc::c_int,
) -> Result<libc::pid_t> {
    if jconf.join_sleeping_thread {
        run_child_wake_up_pooled(jconf)?;
        return Ok(jconf.child_pid.unwrap());
    }

    let (p, _) = run_child_instantiate(jconf, false, callback)?;

    run_child_listen_fd(jconf, jconf.passed_admin_parent_fd)
        .map_err(|e| Error::ParsePid((p, None, e.to_string())))?;

    if !jconf.create_pooled_thread {
        unsafe { libc::close(jconf.passed_admin_parent_fd) };
    }

    jconf.child_pid = Some(p);

    Ok(p)
}

/// do not forget to wait for the child pid after this call returns and then to run clean_after_child
pub fn run_child_pidfd(
    jconf: &mut JailConf,
    callback: extern "C" fn(*mut libc::c_void) -> libc::c_int,
) -> Result<(libc::pid_t, libc::pid_t)> {
    if jconf.join_sleeping_thread {
        run_child_wake_up_pooled(jconf)?;
        return Ok((jconf.child_pid.unwrap(), jconf.child_pidfd.unwrap()));
    }

    let (p, pidfd) = run_child_instantiate(jconf, true, callback)?;

    run_child_listen_fd(jconf, jconf.passed_admin_parent_fd)
        .map_err(|e| Error::ParsePid((p, pidfd, e.to_string())))?;

    if !jconf.create_pooled_thread {
        unsafe { libc::close(jconf.passed_admin_parent_fd) };
    }

    jconf.child_pid = Some(p);
    jconf.child_pidfd = Some(pidfd.unwrap());

    Ok((p, pidfd.unwrap()))
}

fn run_child_instantiate(
    jconf: &mut JailConf,
    gen_pidfd: bool,
    callback: extern "C" fn(*mut libc::c_void) -> libc::c_int,
) -> Result<(libc::pid_t, Option<libc::pid_t>)> {
    let (child_fd, parent_fd) =
        socket::socketpair(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0)?;

    let flags = run_child_setup_flags(jconf);

    jconf.passed_admin_child_fd = child_fd;
    jconf.passed_admin_parent_fd = parent_fd;

    let ref mut stack = [0; STACK_SIZE];

    // When passing jconf directly instead of Box::into_raw(Box::new(jconf.clone())), and when execv fails, the child panic on
    // a double free or unmmap error.
    let (p, pidfd) = if gen_pidfd {
        let (p, pidfd) = clone_proc_box_raw_pidfd(
            callback,
            stack,
            flags,
            Box::into_raw(Box::new(jconf.clone())),
        )?;
        (p, Some(pidfd))
    } else {
        let p = clone_proc_box_raw(
            callback,
            stack,
            flags,
            Box::into_raw(Box::new(jconf.clone())),
        )?;
        (p, None)
    };

    unsafe { libc::close(child_fd) };

    if p < 0 {
        unsafe { libc::close(parent_fd) };
        return Err(("clone failed", Errno::last()).into());
    }

    match init_parent(jconf, p, parent_fd) {
        Ok(_) => {}
        Err(e) => {
            unsafe { libc::close(parent_fd) };
            return Err(Error::ParsePid((p, pidfd, e.to_string())));
        }
    };

    Ok((p, pidfd))
}

fn run_child_setup_flags(jconf: &JailConf) -> libc::c_int {
    let mut flags: libc::c_int = 0;
    if jconf.clone_newnet {
        flags |= libc::CLONE_NEWNET;
    }
    if jconf.clone_newuser {
        flags |= libc::CLONE_NEWUSER;
    }
    if jconf.clone_newns {
        flags |= libc::CLONE_NEWNS;
    }
    if jconf.clone_newpid {
        flags |= libc::CLONE_NEWPID;
    }
    if jconf.clone_newipc {
        flags |= libc::CLONE_NEWIPC;
    }
    if jconf.clone_newuts {
        flags |= libc::CLONE_NEWUTS;
    }
    if jconf.clone_newcgroup {
        flags |= libc::CLONE_NEWCGROUP;
    }
    flags |= libc::SIGCHLD;

    flags
}

fn run_child_listen_fd(jconf: &JailConf, parent_fd: i32) -> Result<()> {
    let mut buf = [0; 1];
    if read_from_fd_ignore_err(parent_fd, &mut buf)? == 1 {
        match buf[0] as char {
            'W' => {}
            'E' => {
                if jconf.debug {
                    let mut buf = [0; 4096];
                    let len = read_from_fd_ignore_err(parent_fd, &mut buf)?;
                    let err = String::from_utf8(buf[..len].to_owned())
                        .map_err(|e| format!("could not read err: {}", e))?;

                    unsafe { libc::close(parent_fd) };
                    return Err(format!(
                        "Received error message from the child process before it has been executed: {}",
                        err
                    )
                    .into());
                }

                unsafe { libc::close(parent_fd) };
                return Err(
                    "Received error message from the child process before it has been executed"
                        .into(),
                );
            }
            _ => panic!("invalid response"),
        }
    }

    Ok(())
}

fn run_child_wake_up_pooled(jconf: &mut JailConf) -> Result<()> {
    // let now = std::time::Instant::now();

    write_message_to_fd(
        jconf.passed_admin_parent_fd,
        &create_pooled_wake_up_mess(jconf),
    )?;

    // let now2 = std::time::Instant::now();
    // println!("write_message_to_fd: {:?}", now2.duration_since(now));

    run_child_listen_fd(jconf, jconf.passed_admin_parent_fd)?;
    unsafe { libc::close(jconf.passed_admin_parent_fd) };

    Ok(())
}

pub extern "C" fn child(data: *mut libc::c_void) -> libc::c_int {
    let ref mut jconf = *unsafe { Box::from_raw(data as *mut JailConf) };

    // check jconf to see if we need to join existing namespaces

    unsafe { libc::close(jconf.passed_admin_parent_fd) };

    let err = match subproc_new_proc(jconf) {
        Ok(_) => {
            panic!("should not happen")
            // unsafe { libc::close(jconf.passed_admin_child_fd) };
            // return 0;
        }
        Err(e) => e,
    };

    if !write_to_fd(jconf.passed_admin_child_fd, "E".as_bytes()) {
        println!("failed to write to child fd that child failed");
    }

    if jconf.debug {
        if !write_to_fd(
            jconf.passed_admin_child_fd,
            format!("err child: {}", err).as_bytes(),
        ) {
            println!("failed to write err to child fd: {}", err);
        }
    } else {
        println!("could not run command: {}", err);
    }

    unsafe { libc::close(jconf.passed_admin_child_fd) };

    1
}

pub fn init_parent(jconf: &mut JailConf, pid: libc::pid_t, pipefd: libc::c_int) -> Result<()> {
    // if jconf.create_pooled_nms {
    //     // in order: user, mnt, net, uts, cgroup, ipc
    //     jconf.join_nms = Some([
    //         fcntl::open(&format!("/proc/{}/ns/user", pid), libc::O_RDONLY, 0)?,
    //         fcntl::open(&format!("/proc/{}/ns/mnt", pid), libc::O_RDONLY, 0)?,
    //         fcntl::open(&format!("/proc/{}/ns/net", pid), libc::O_RDONLY, 0)?,
    //         fcntl::open(&format!("/proc/{}/ns/uts", pid), libc::O_RDONLY, 0)?,
    //         fcntl::open(&format!("/proc/{}/ns/cgroup", pid), libc::O_RDONLY, 0)?,
    //         fcntl::open(&format!("/proc/{}/ns/ipc", pid), libc::O_RDONLY, 0)?,
    //     ])
    // }

    if jconf.clone_newnet {
        net::init_ns_from_parent(jconf, pid)?;
    }

    let pid_string = pid.to_string();

    if jconf.clone_newcgroup {
        if jconf.use_cgroupv2 {
            cgroupv2::init_ns_from_parent(jconf, &pid_string)?; // std::io::error is coerced to jail::error::Error if any
        } else {
            cgroupv1::init_ns_from_parent(jconf, &pid_string)?;
        }
    }

    if jconf.clone_newuser {
        user::init_ns_from_parent(jconf, &pid_string, jconf.borrow_env())?;
    }

    if !write_to_fd(pipefd, "D".as_bytes()) {
        return Err("Couldn't signal the new process via a socketpair".into());
    }

    Ok(())
}

/* Reset the execution environment for the new process */
/*
SIG_ERR	// Error return.
SIG_DFL	// Default action.
SIG_IGN	// Ignore signal.
*/
pub fn reset_env() -> bool {
    /* Set all previously changed signals to their default behavior */

    // &sig to destructure the pattern
    for &sig in NSSIGS.iter() {
        if unsafe { libc::signal(sig, libc::SIG_DFL) } == libc::SIG_ERR {
            return false;
        }
    }

    /* Unblock all signals */
    // see https://doc.rust-lang.org/std/mem/union.MaybeUninit.html
    // This new feature may soon be stable, see if it should replace maybeunit: https://doc.rust-lang.org/std/boxed/struct.Box.html#method.new_uninit
    let mut sset = MaybeUninit::<libc::sigset_t>::uninit();
    match SyscallReturnCode(unsafe { libc::sigemptyset(sset.as_mut_ptr()) }).into_empty_result() {
        Err(_) => return false,
        _ => (),
    }
    let sset = unsafe { sset.assume_init() };

    if unsafe { libc::sigprocmask(libc::SIG_SETMASK, &sset, ptr::null_mut()) } == -1 {
        return false;
    }

    true
}

pub fn subproc_new_proc(jconf: &mut JailConf) -> Result<()> {
    subproc_new_proc_setup(jconf)?;
    subproc_new_proc_exec(jconf)
}

pub fn subproc_new_proc_setup(jconf: &mut JailConf) -> Result<()> {
    setup_fd(jconf)?;

    if !reset_env() {
        // I think we need to do that only if we do set custom sig handlers as nsjail, investigate
        return Err("could not reset env".into());
    }

    if jconf.passed_admin_child_fd == -1 {
        panic!("should not happen");
    }

    {
        let mut buf = [0; 1];
        if read_from_fd_ignore_err(jconf.passed_admin_child_fd, &mut buf)? != 1 {
            return Err(format!("could not read from pipefd: {}", Errno::last()).into());
        }
        if buf[0] as char != 'D' {
            return Err("read invalid char from pipefd".into());
        }
    }

    contain::contain_proc(jconf)?;

    if !jconf.keep_env {
        if unsafe { libc::clearenv() } < 0 {
            return Err(format!(
                "Could not clearenv in subproc_new_proc_setup: {}",
                Errno::last()
            )
            .into());
        }
    }

    // We must borrow env, because a modification to the string used by libc::putenv also modifies the environment variable and thus
    // when rust drops the taken array env when this if statement ends, it also unsets the corresponding just set environment variable
    if let Some(env) = jconf.borrow_env() {
        for i in 0..env.len() {
            if unsafe { libc::putenv(env[i].as_c_str().as_ptr() as *mut libc::c_char) } < 0 {
                return Err(format!(
                    "Could not putenv in subproc_new_proc_setup: {}",
                    Errno::last()
                )
                .into());
            }
        }
    }

    /* Should be the last one in the sequence */
    if !sandbox::apply_policy(jconf) {
        return Err("could not apply seccomp policy".into());
    }

    Ok(())
}

pub fn subproc_new_proc_exec(jconf: &mut JailConf) -> Result<()> {
    let err = {
        if jconf.use_exec_caveat {
            execv::execveat(
                jconf.exec_fd as libc::c_int,
                None::<CString>,
                jconf.argv.take().unwrap().as_slice(),
                jconf.env.take().unwrap().as_slice(),
                libc::AT_EMPTY_PATH,
            )
        } else {
            if let Some(exec_file) = &jconf.exec_file {
                execv::execv(exec_file.as_c_str(), jconf.argv.take().unwrap().as_slice())
            } else {
                return Err("no exec file specified".into());
            }
        }
    };

    // if execv or execveat succeeds, it replaces the current program image with the new one being executed, so if we end up here something bad happened
    Err(format!("exec failed: {:?}", err).into())
}

/*
 * See https://blog.tanelpoder.com/2013/02/21/peeking-into-linux-kernel-land-using-proc-filesystem-for-quickndirty-troubleshooting/ and http://man7.org/linux/man-pages/man5/proc.5.html
 *
 * The /proc/[pid]/syscall file exposes the system call number and argument regis‐
 * ters for the system call currently being executed by the
 * process, followed by the values of the stack pointer and pro‐
 * gram counter registers.  The values of all six argument regis‐
 * ters are exposed, although most system calls use fewer regis‐
 * ters.

 * If the process is blocked, but not in a system call, then the
 * file displays -1 in place of the system call number, followed
 * by just the values of the stack pointer and program counter.
 * If process is not blocked, then the file contains just the
 * string "running".

 * This file is present only if the kernel was configured with
 * CONFIG_HAVE_ARCH_TRACEHOOK.

 * Permission to access this file is governed by a ptrace access
 * mode PTRACE_MODE_ATTACH_FSCREDS check; see ptrace(2).
*/
pub fn add_proc(jconf: &mut JailConf, pid: libc::pid_t) -> Result<()> {
    let proc_syscall = format!("/proc/{}/syscall", pid);
    let proc_syscall = CString::new(proc_syscall).unwrap();
    let pid_t = PIDT {
        pid: pid,
        start: SystemTime::now(),
        remote_txt: "[STANDALONE MODE]",
        pid_syscall_fd: unsafe {
            libc::open(proc_syscall.as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC)
        },
    };

    if pid_t.pid_syscall_fd < 0 {
        return Err(("Could not open /proc/{pid}/syscall", Errno::last()).into());
    }

    if jconf.pids.iter().any(|i| i.pid == pid) {
        return Err("pid already exists".into());
    }

    jconf.pids.push(pid_t);

    Ok(())
}

/*
 * See clone_proc documentation (this is an experiment for now)
 *
 * Follow these tricks: https://blog.seantheprogrammer.com/neat-rust-tricks-passing-rust-closures-to-c
*/
pub fn clone_proc_closure() {}

/*
 * See clone_proc documentation and why this is better than clone_proc
*/
pub fn clone_proc_box_raw(
    callback: extern "C" fn(*mut libc::c_void) -> libc::c_int,
    stack: &mut [u8],
    flags: libc::c_int,
    fd: *mut JailConf, // unsafe{Box::into_raw(Box::new(jconf_clone))}
) -> Result<libc::pid_t> {
    if flags & libc::CLONE_VM != 0 {
        // parent and child would share the same memory space, see linux man for clone
        return Err("cannot use flag CLONE_VM".into());
    }

    let pid = unsafe {
        /*
         * Avoid the problem of the stack growing up/down under different CPU architectures,
         * by using middle of the static stack buffer (which is temporary, and used only
         * inside of the cloneFunc()
         */
        let ptr = stack.as_mut_ptr().offset((stack.len() / 2) as isize);

        // Some CPU archs (e.g. aarch64) must have it aligned.
        let ptr_aligned = ptr.offset((ptr as usize % 16) as isize * -1);

        libc::clone(
            /* Callback function example
                    extern "C" fn mnt_child(data: *mut libc::c_void) -> c_int {
                        let jconf = unsafe{Box::from_raw(data as *mut JailConf)};
                    }
            */
            callback,
            ptr_aligned as *mut libc::c_void,
            flags,
            fd as *mut _ as *mut libc::c_void,
        )
    };

    if pid < 0 {
        Err(Errno::last().into())
    } else {
        Ok(pid)
    }
}

pub fn clone_proc_box_raw_pidfd(
    callback: extern "C" fn(*mut libc::c_void) -> libc::c_int,
    stack: &mut [u8],
    mut flags: libc::c_int,
    fd: *mut JailConf, // unsafe{Box::into_raw(Box::new(jconf_clone))}
) -> Result<(libc::pid_t, libc::pid_t)> {
    if flags & libc::CLONE_VM != 0 {
        // parent and child would share the same memory space, see linux man for clone
        return Err("cannot use flag CLONE_VM".into());
    }

    flags |= CLONE_PIDFD;
    let mut parent_tid: libc::pid_t = 0;

    let pid = unsafe {
        // On processors that run stacks should grow downward but there are a few exceptions
        // this is why we point to the middle of the stack
        let ptr = stack.as_mut_ptr().offset((stack.len() / 2) as isize);

        // Some CPU archs (e.g. aarch64) must have it aligned.
        let ptr_aligned = ptr.offset((ptr as usize % 16) as isize * -1);

        libc::clone(
            /* Callback function example
                    extern "C" fn mnt_child(data: *mut libc::c_void) -> c_int {
                        let jconf = unsafe{Box::from_raw(data as *mut JailConf)};
                    }
            */
            callback,
            ptr_aligned as *mut libc::c_void,
            flags,
            fd as *mut _ as *mut libc::c_void,
            &mut parent_tid as *mut libc::pid_t,
        )
    };

    if pid < 0 {
        Err(Errno::last().into())
    } else {
        Ok((pid, parent_tid))
    }
}

/**
 * CloneFd takes ownership of JailConf which is moved inside it,
 * so there will be no race conditions once the extern C child function
 * handles the passed JailConf.
 * use JailConf.clone derived trait. It takes ~150 nanoseconds to clone JailConf struct.
 *
 * Check out https://blog.rust-lang.org/2020/01/30/Rust-1.41.0.html#more-guarantees-when-using-boxt%3E-in-ffi: "So if you have an extern "C" Rust function,
 * called from C, your Rust function can now use Box<T>, for some specific T, while using T* in C for the corresponding function."
 */
pub type CloneFd<'a> = Box<JailConf<'a>>;

/*
 * Nsjail uses jmp_buf and longjmp to store env and restore it, but rust doesn't support them
 * see https://github.com/rust-lang/rfcs/issues/2625
 *
 * Avoid problems with caching of PID/TID in glibc - when using syscall(__NR_clone) glibc doesn't
 * update the internal PID/TID caches, what can lead to invalid values being returned by getpid()
 * or incorrect PID/TIDs used in raise()/abort() functions
 *
 * Some CPU archs (e.g. aarch64) must have the stack aligned (align(16)). Check if the way we align
 * it works fine
 *
 * Stacks grow downward on all processclone_procors that run Linux (except the HP PA processors), so child_stack usually
 * points to the topmost address of the memory space set up for the child stack. But to be sure to avoid the problem
 * of the stack growing up/down under different CPU architectures, we use middle of the static stack buffer that you provide
 * (which is temporary, and used only inside of the cloneFunc().
 *
 * The static stack buffer you provide should be a multiple of 16.
 *
 * If you need to allocate a very big byte slice without issues, may need to look in the future at allocate_byte_array in utils.allocate_byte_array().
 * It is an unstable feature from RFC2116. More info in utils.rs. See also https://github.com/nix-rust/nix/issues/555.
 *
 * Clone proc takes ~100 microseconds with flags libc::CLONE_NEWUTS | libc::SIGCHLD -> why ?
 *
 * See https://blog.seantheprogrammer.com/neat-rust-tricks-passing-rust-closures-to-c ; maybe a way to avoid transmute
 *
 * This way of passing jconf works for number fields inside jconf, but as soon as we try to access some Rust Smart pointers like String, &str or vec! from the child,
 * the child exits without any messages.
 * Some day one could search this and find the explanation which could be useful and have some unexpected applications. One possible explanation could be that the address
 * passed to the child (the box) remains the same in the new memory space (replicated from parent), but that pointers (addresses) that were inside the passed jconf changed,
 * e.g. String, &str, vec, ... (maybe other but only tested for these three rust smart pointers). Check this out.
 */
pub fn clone_proc(
    callback: extern "C" fn(*mut libc::c_void) -> libc::c_int,
    stack: &mut [u8],
    flags: libc::c_int,
    mut fd: CloneFd,
) -> Result<libc::pid_t> {
    // in nsjail ```if (flags & CLONE_VM) {```, see https://stackoverflow.com/questions/1479100/how-is-if-statement-evaluated-in-c
    // Cannot use clone(flags & CLONE_VM)
    if flags & libc::CLONE_VM != 0 {
        return Err("cannot use flag CLONE_VM".into()); // parent and child would share the same memory space, see linux man for clone
    }

    let pid = unsafe {
        let ptr = stack.as_mut_ptr().offset((stack.len() / 2) as isize);
        // Some CPU archs (e.g. aarch64) must have it aligned.
        let ptr_aligned = ptr.offset((ptr as usize % 16) as isize * -1);

        libc::clone(
            // mem::transmute(callback as extern "C" fn(*mut Box<dyn FnMut() -> isize>) -> i32),
            callback,
            ptr_aligned as *mut libc::c_void,
            flags,
            &mut fd as *mut _ as *mut libc::c_void,
        )
    };

    if pid < 0 {
        Err(Errno::last().into())
    } else {
        Ok(pid)
    }
}

pub fn system_exe_wrapper(cmd: &str, env: Option<&[CString]>) -> Result<i64> {
    system_exe(
        CString::new("/bin/sh").unwrap().as_c_str(),
        &[
            CString::new(b"".as_ref()).unwrap().as_c_str(),
            CString::new(b"-c".as_ref()).unwrap().as_c_str(),
            CString::new(cmd).unwrap().as_c_str(),
        ],
        env,
    )
}

// env in nsjail environ is defined as a global variable in the Glibc source file posix/environ.c
// maybe we can get the same with https://doc.rust-lang.org/std/env/index.html ?
pub fn system_exe(path: &CStr, args: &[&CStr], env: Option<&[CString]>) -> Result<i64> {
    let mut exec_failed = false;

    let args_p = to_exec_array(args);
    let env_p = if let Some(env) = env {
        to_exec_array_cstring(env)
    } else {
        vec![]
    };

    let sv = match unistd::pipe2(libc::O_CLOEXEC) {
        Ok(v) => v,
        Err(e) => return Err(e.into()),
    };

    /* Clone the calling process, creating an exact copy.
    Return -1 for errors, 0 to the new process,
    and the process ID of the new process to the old process.  */
    // see sys_util::unistd for a rust more idiomatic approach and doc
    let pid = unsafe { libc::fork() };

    if pid == -1 {
        unsafe {
            libc::close(sv.0);
            libc::close(sv.1);
        }
        return Err(Errno::last().into());
    }

    if pid == 0 {
        // child/new process
        unsafe {
            libc::close(sv.0);
            // see subproc.md
            libc::execve(path.as_ptr(), args_p.as_ptr(), env_p.as_ptr());
        }
        println!("{}", sys_util::errno::Errno::last());
        write_to_fd(sv.1, "A".as_bytes());
        ::std::process::exit(0);
    }

    unsafe {
        libc::close(sv.1);
    }

    let mut buf = [0; 1];
    if read_from_fd_ignore_err(sv.0, &mut buf)? > 0 {
        exec_failed = true;
    }
    unsafe {
        libc::close(sv.0);
    }

    loop {
        let mut status: libc::c_int = 0;
        let ret = unsafe { libc::wait4(pid, &mut status, libc::__WALL, std::ptr::null_mut()) };
        if ret == -1 && sys_util::errno::Errno::last() == sys_util::errno::EINTR {
            continue;
        }
        if ret == -1 {
            return Err(Errno::last().into());
        }
        if unsafe { libc::WIFEXITED(status) } {
            let exit_code = unsafe { libc::WEXITSTATUS(status) };
            if exec_failed {
                return Err("exec failed".into());
            } else if exit_code == 0 {
                return Ok(0);
            } else {
                return Ok(1);
            }
        }
        if unsafe { libc::WIFSIGNALED(status) } {
            // let exit_signal = unsafe{libc::WTERMSIG(status)};
            return Ok(2);
        }

        // unknown exit status: status
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::ffi::CStr;

//     #[test]
//     fn test_system_exe() {
//         println!("");
//         let res = system_exe(
//             CString::new("/bin/sh").unwrap().as_c_str(),
//             &[
//                 CString::new(b"".as_ref()).unwrap().as_c_str(),
//                 CString::new(b"-c".as_ref()).unwrap().as_c_str(),
//                 CString::new(b"ls /".as_ref()).unwrap().as_c_str(),
//             ],
//             &[],
//         );
//         println!("{}\n", res);

//         let res = system_exe(
//             CString::new("/bin/ls").unwrap().as_c_str(),
//             &[
//                 CString::new(b"".as_ref()).unwrap().as_c_str(),
//                 CString::new(b"/home".as_ref()).unwrap().as_c_str(),
//             ],
//             &[],
//         );
//         println!("{}\n", res);

//         // without the leading empty b"", next arguments are ignored, why ?
//         let res = system_exe(
//             CString::new("/bin/ls").unwrap().as_c_str(),
//             &[CString::new(b"/home".as_ref()).unwrap().as_c_str()],
//             &[],
//         );
//         println!("{}\n", res);

//         let res = system_exe(
//             &CString::new("/bin/echo").unwrap(),
//             &[
//                 &CString::new(b"".as_ref()).unwrap(),
//                 &CString::new(b"foo=$foo".as_ref()).unwrap(),
//             ],
//             &[CString::new(b"foo=bar".as_ref()).unwrap()],
//         );
//         println!("{}\n", res);

//         // for env tab to work, we need to do sh -c {cmd}
//         let res = system_exe(
//             CString::new("/bin/sh").unwrap().as_c_str(),
//             &[
//                 CString::new(b"".as_ref()).unwrap().as_c_str(),
//                 CString::new(b"-c".as_ref()).unwrap().as_c_str(),
//                 CString::new(b"echo foo=$foo".as_ref()).unwrap().as_c_str(),
//             ],
//             &[CString::new(b"foo=CString".as_ref()).unwrap()],
//         );
//         println!("{}\n", res);

//         let res = system_exe(
//             CString::new("/bin/sh").unwrap().as_c_str(),
//             &[
//                 CString::new(b"".as_ref()).unwrap().as_c_str(),
//                 CString::new(b"-c".as_ref()).unwrap().as_c_str(),
//                 CString::new(b"newuidmap".as_ref()).unwrap().as_c_str(),
//             ],
//             &[],
//         );
//         println!("{}\n", res);

//         let res = system_exe_wrapper(
//             "ls /home && /usr/bin/newgidmap && echo bob=$bob",
//             &[CString::new(b"bob=alice".as_ref()).unwrap()],
//         );
//         println!("{}\n", res);
//     }
// }
