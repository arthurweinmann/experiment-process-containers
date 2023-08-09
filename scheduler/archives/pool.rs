use crossbeam_queue::{ArrayQueue, PushError};
use evmap::{shallow_copy::ShallowCopy, ReadHandle, WriteHandle};

use std::sync::{Arc, Mutex};

use sys_util::errno::Errno;

pub struct Pool {
    empty_mnt_nm: ArrayQueue<i32>,
    ipc_nm: ArrayQueue<i32>,
    net_nm: ArrayQueue<i32>,
    user_nm: ArrayQueue<i32>,
    cgroup_nm: ArrayQueue<i32>,
    cgroup_parent: ArrayQueue<i32>, // cgroup id, to be converted into a path, correspond to cgroup in the parent to limit and monitor resources
    uts: i32,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct NM {
    // namespace reused after execution, e.g. put back into pool, may be used by any userid
    user: i32,
    net: i32,
    cgroup_parent: i32,
    // constant namespace, same for all execution for all users
    uts: i32,
    // per userid attributed namespace, same for all execution for the same userid
    mnt: i32,
    // namespace destroyed after each execution
    ipc: i32,
    cgroup: i32,
}

impl ShallowCopy for NM {
    unsafe fn shallow_copy(&mut self) -> Self {
        *self
    }
}

impl<'a> Pool {
    pub fn create_running_pid_handle() -> (ReadHandle<i32, NM>, WriteHandle<i32, NM>) {
        evmap::new()
    }

    pub fn create_user_mnt_nm_handler() -> (ReadHandle<String, i32>, WriteHandle<String, i32>) {
        evmap::new()
    }

    pub fn pull(
        &self,
        userid: String,
        userid_mnt_r: ReadHandle<String, i32>,
        userid_mnt_w: Arc<Mutex<WriteHandle<String, i32>>>,
    ) -> NM {
        let userid_mnt: i32;

        if let Some(result) = userid_mnt_r.get_and(userid.as_str(), |rs| rs[0]) {
            userid_mnt = result;
        } else {
            {
                let mut userid_mnt_w = userid_mnt_w.lock().unwrap();
                if let Some(result) = userid_mnt_r.get_and(userid.as_str(), |rs| rs[0]) {
                    // recheck in case another thread created it before we could acquire the lock
                    userid_mnt = result;
                } else {
                    userid_mnt = match self.empty_mnt_nm.pop() {
                        Ok(v) => v,
                        Err(_) => -1,
                    };

                    if userid_mnt > -1 {
                        userid_mnt_w.insert(userid, userid_mnt).refresh();
                    }
                }
            }
        }

        NM {
            user: match self.user_nm.pop() {
                Ok(v) => v,
                Err(_) => -1,
            },
            ipc: match self.ipc_nm.pop() {
                Ok(v) => v,
                Err(_) => -1,
            },
            net: match self.net_nm.pop() {
                Ok(v) => v,
                Err(_) => -1,
            },
            cgroup: match self.cgroup_nm.pop() {
                Ok(v) => v,
                Err(_) => -1,
            },
            cgroup_parent: match self.cgroup_parent.pop() {
                Ok(v) => v,
                Err(_) => -1,
            },
            mnt: userid_mnt,
            uts: self.uts,
        }
    }

    pub fn push(&self, nm: NM) {
        match self.net_nm.push(nm.net) {
            Ok(_) => {}
            Err(e) => {
                if unsafe{ libc::close(e.0) } == -1 {
                    panic!("could not close net namespace: {}", Errno::last());
                }
            }
        };
        match self.cgroup_parent.push(nm.cgroup_parent) {
            Ok(_) => {}
            Err(e) => {
                if unsafe{ libc::close(e.0) } == -1 {
                    panic!("could not close parent cgroup: {}", Errno::last());
                }
            }
        };
        match self.user_nm.push(nm.user) {
            Ok(_) => {}
            Err(e) => {
                if unsafe{ libc::close(e.0) } == -1 {
                    panic!("could not close user namespace: {}", Errno::last());
                }
            }
        };
        // closing IPC and CGROUP namespace
        if unsafe{ libc::close(nm.ipc) } == -1 {
            panic!("could not close ipc namespace: {}", Errno::last());
        }
        if unsafe{ libc::close(nm.cgroup) } == -1 {
            panic!("could not close cgroup namespace: {}", Errno::last());
        }
    }

    pub fn new(pool_size: usize, uts_nm: i32) -> Pool {
        Pool {
            empty_mnt_nm: ArrayQueue::new(pool_size),
            ipc_nm: ArrayQueue::new(pool_size),
            net_nm: ArrayQueue::new(pool_size),
            user_nm: ArrayQueue::new(pool_size),
            cgroup_nm: ArrayQueue::new(pool_size),
            cgroup_parent: ArrayQueue::new(pool_size),
            uts: uts_nm,
        }
    }

    pub fn close(self) {
        unsafe { libc::close(self.uts) };
    }
}
