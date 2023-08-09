//!
//! This implementation (which should be replace by future on pidfd once ubuntu adopt kernel version >= 5.2) is based on the following asumption:
//!
//! * (since Linux 2.4) a thread can, and by default will, wait on children of other threads in the same thread group (https://linux.die.net/man/2/wait)
//!
//! * Rust thread spawn internally uses linux clone syscall with flag `CLONE_THREAD`: If CLONE_THREAD is set, the child is placed in the same thread
//! group as the calling process. (http://squidarth.com/rc/rust/concurrency/2018/06/09/rust-threads-detach.html)
//!
//! These asumptions can be tested with function check_wait_asumptions()
//!

use cmd::exec::BashCommand;
use jail::error::Result;

use crossbeam_channel::{Receiver, TryRecvError};
use evmap::{ReadHandle, WriteHandle};

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use super::pool::{Pool, NM};

pub fn start(
    pool: Arc<Pool>,
    pid_read_handle: ReadHandle<i32, NM>,
    pid_write_handle: Arc<Mutex<WriteHandle<i32, NM>>>,
    stop: Receiver<bool>,
) -> JoinHandle<i32> {
    thread::spawn(move || {
        let mut i = 0;
        let mut status: i32 = 0;
        'L1: loop {
            if match stop.try_recv() {
                Ok(v) => v,
                Err(TryRecvError::Disconnected) => true,
                Err(TryRecvError::Empty) => false,
            } {
                break 'L1;
            }
            let pid = unsafe { libc::wait(&mut status as *mut libc::c_int) };
            if pid < 0 {
                panic!("Error waiting");
            }
            if let Some(result) = pid_read_handle.get_and(&pid, |rs| rs[0]) {
                // we delete the remaining slice in the hashmap, but we refresh it for readers only every N loops
                if i == 15 {
                    // we need a subscope so the mutex lock is dropped asap
                    {
                        let mut pid_write_handle = pid_write_handle.lock().unwrap();
                        pid_write_handle.empty(pid).refresh();
                    }
                    i = 0;
                } else {
                    {
                        let mut pid_write_handle = pid_write_handle.lock().unwrap();
                        pid_write_handle.empty(pid);
                    }
                    i = i + 1;
                }
                pool.push(result);
            }
        }

        0
    })
}

pub fn check_wait_asumptions() -> Result<()> {
    let parent_tuple = util_get_wait_check_info();

    let handle1 = thread::spawn(|| {
        let tupl = util_get_wait_check_info();

        let handle2 = thread::spawn(|| util_get_wait_check_info());

        (tupl, handle2.join().unwrap())
    });

    let res = handle1.join().unwrap();
    let res1 = res.0;
    let res2 = res.1;

    if res1 != parent_tuple {
        return Err("child 1 is not in same thread group".into());
    }
    if res2 != parent_tuple {
        return Err("child 2 is not in same thread group".into());
    }

    println!("parent:         {:?}", parent_tuple);
    println!("result child 1: {:?}", res1);
    println!("result child 2: {:?}", res2);

    Ok(())
}

fn util_get_wait_check_info() -> (String, String, String, String, i32, i32) {
    (
        BashCommand::new_sh(format!("cat /proc/{}/status | grep Tgid", std::process::id()).as_str())
            .run_utf8()
            .unwrap(),
        BashCommand::new_sh(format!("cat /proc/{}/status | grep PPid", std::process::id()).as_str())
            .run_utf8()
            .unwrap(),
        BashCommand::new_sh(format!("cat /proc/{}/status | grep NStgid", std::process::id()).as_str())
            .run_utf8()
            .unwrap(),
        BashCommand::new_sh(format!("cat /proc/{}/status | grep NSpgid", std::process::id()).as_str())
            .run_utf8()
            .unwrap(),
        unsafe { libc::getppid() },
        unsafe { libc::getpid() },
    )
}

#[cfg(test)]
mod tests {
    use super::check_wait_asumptions;

    #[test]
    fn test_wait_asumptions() {
        println!("");
        check_wait_asumptions().unwrap();
    }
}
