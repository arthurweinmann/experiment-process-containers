use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crossbeam_channel::{bounded, Sender};
use evmap::{ReadHandle, WriteHandle};

use super::pool::{self, Pool};
use super::thread_pool::ThreadPool;
use super::nm_creator;
use super::wait;

use sys_util::num_cpu;

pub struct Scheduler {
    wait_loop_handle: JoinHandle<i32>,
    stop_wait_loop: Sender<bool>,
    userid_mnt_read: ReadHandle<String, i32>,
    userid_mnt_write: Arc<Mutex<WriteHandle<String, i32>>>,
    threads: ThreadPool,
    pool: Arc<Pool>,
}

pub fn start() -> Scheduler {
    let mut num_cpu = num_cpu::get() / 2;
    if num_cpu <= 0 {
        num_cpu = 1;
    }

    let (userid_mnt_read, userid_mnt_write) = pool::Pool::create_user_mnt_nm_handler();
    let (running_pid_read, running_pid_write) = pool::Pool::create_running_pid_handle();

    let tmpjconf = nm_creator::create_uts_namespace("TOASTATE").expect("Could not create uts namespace, aborting since everything else has a great chance of not working");

    let pool = pool::Pool::new(num_cpu * 10, tmpjconf.uts_nm_fd.unwrap());

    let pid_atomic_ref_counter_multi_writer = Arc::new(Mutex::new(running_pid_write));
    let pool_atomic_ref_counter = Arc::new(pool);

    let (wait_stopper, wait_stop_reader) = bounded::<bool>(1);

    // thread waiting for children pid to terminate. Push back NM used by child into pool.
    let wait_loop_handle = wait::start(
        pool_atomic_ref_counter.clone(),
        running_pid_read.clone(),
        pid_atomic_ref_counter_multi_writer.clone(),
        wait_stop_reader,
    );

    Scheduler {
        wait_loop_handle: wait_loop_handle,
        stop_wait_loop: wait_stopper,
        userid_mnt_read: userid_mnt_read,
        userid_mnt_write: Arc::new(Mutex::new(userid_mnt_write)),
        threads: ThreadPool::new(num_cpu),
        pool: pool_atomic_ref_counter,
    }
}

impl Scheduler {
    pub fn wait(self) {
        self.wait_loop_handle.join().unwrap();
    }

    pub fn stop(&self) {
        self.stop_wait_loop.send(true).unwrap();
    }

    pub fn execute(&self, userid: String) {
        let pool_ref = self.pool.clone();
        let userid_mnt_read_ref = self.userid_mnt_read.clone();
        let userid_mnt_write_ref = self.userid_mnt_write.clone();

        self.threads.execute(move || {
            let nm_to_join = pool_ref.pull(userid, userid_mnt_read_ref, userid_mnt_write_ref);

            // use jail package to execute
        });
    }
}