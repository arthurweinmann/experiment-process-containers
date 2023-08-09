use std::ffi::CStr;

use sys_util::errno::Errno;

use jail::config::JailConf;
use jail::sandbox::which_seccomp_violation_pid_only;
use jail::subproc;

use disk::overlay_fs::OverlayDir;

use super::gtvs_message::GtvsMessageWriter;
use super::hash_table::HashTable;
use jail::protobuf::{parse_uint32_cstr, put_u32};
use super::time_utils::{timestamp_micro, timestamp_second};

pub struct Waiter {
    wait_status: i32,
}

impl Waiter {
    pub fn new() -> Self {
        Waiter { wait_status: 0 }
    }

    pub fn kill_pid(&self, pid: i32, timeout_sec: u64, pid_hash_table: &mut HashTable) {
        if let Some(_) = pid_hash_table.borrow(pid) {
            if timeout_sec == 0 {
                if unsafe { libc::kill(pid, libc::SIGKILL) } != 0 {
                    println!("Could not SIGKILL {}: {}", pid, Errno::last());
                }
            } else {
                if unsafe { libc::kill(pid, libc::SIGTERM) } != 0 {
                    println!("Could not SIGTERM {}: {}", pid, Errno::last());
                }

                let to_kill =
                    pid_hash_table.put_in_kill_after(pid as u64, timestamp_second(timeout_sec));
                if to_kill.len() > 0 {
                    for p in to_kill {
                        if unsafe { libc::kill(p as i32, libc::SIGKILL) } != 0 {
                            println!("Could not SIGKILL {}: {}", p, Errno::last());
                        }
                    }
                }
            }
        }
    }

    pub fn kill_timed_out(&self, pid_hash_table: &mut HashTable) {
        let to_kill = pid_hash_table.extract_kill_after();
        if to_kill.len() > 0 {
            for p in to_kill {
                if unsafe { libc::kill(p as i32, libc::SIGTERM) } != 0 {
                    println!("Could not SIGTERM {}: {}", p, Errno::last());
                }
            }
        }
    }

    pub fn wait_pid<'a>(
        &mut self,
        pid_src: i32,
        gtvs_mess_buffer: &'a mut GtvsMessageWriter,
        pid_hash_table: &'a mut HashTable,
    ) {
        // println!("wait_pid: {}", pid_src);
        // println!("{}", which_seccomp_violation_pid_only(pid)); // TODO: send refused syscall to gtvs if one, test only in the case of an error

        let mut pid = unsafe {
            libc::waitpid(
                pid_src,
                &mut self.wait_status as *mut libc::c_int,
                libc::WNOHANG,
            )
        };
        if pid < 0 {
            println!("Error waiting: {}", Errno::last());

            pid = pid_src;
        }
        if let Some(item) = pid_hash_table.pop(pid) {
            // println!(
            //     "found item corresponding to pid in hash table, toaster exe {} ",
            //     item.exe_id
            // ); // timestamp_micro()-item.timestamp_micro

            subproc::clean_after_child(&item.jconf, pid).expect("could not clean_after_child");

            let mut mess: [u8; 19] = [0; 19];

            if let Some(ovdir) = item.ovdir {
                put_u32(&mut mess, 15, parse_uint32_cstr(ovdir.uid.as_c_str()));
                mess[2] = 2; // mess type

                // Do not forget in golang to delete btrfs subvolume and directory of deleted mount overlay
                // after doing needed OP like code saving in case of a compilation
                ovdir.kill().expect("could not kill ovdir in waiter");
            } else {
                mess[2] = 1; // mess type
            }

            mess[0] = (17 >> 8) as u8; // len with len uint16 excluded
            mess[1] = 17 as u8;
            put_u32(&mut mess, 3, item.exe_id); // exe id
            if unsafe { libc::WIFEXITED(self.wait_status) } {
                let status = unsafe { libc::WEXITSTATUS(self.wait_status) };
                put_u32(&mut mess, 7, status as u32);
            }
            if unsafe { libc::WIFSIGNALED(self.wait_status) } {
                let kill_by_signal = unsafe { libc::WTERMSIG(self.wait_status) };
                put_u32(&mut mess, 11, kill_by_signal as u32);
            }

            gtvs_mess_buffer.write_mess(mess);
        } else {
            println!("WARNING: could not find pid item: {}", pid);
        }
    }

    pub fn wait_pid_from_err<'a>(
        &mut self,
        pid: i32,
        exe_id: u32,
        gtvs_mess_buffer: &'a mut GtvsMessageWriter,
        jconf: &JailConf,
        ovdir: Option<OverlayDir>,
    ) {
        // println!("wait_pid_from_err: {} {}", pid, exe_id);
        // println!("{}", which_seccomp_violation_pid_only(pid)); // TODO: send refused syscall to gtvs if one, test only in the case of an error

        let pid = unsafe {
            libc::waitpid(
                pid as i32,
                &mut self.wait_status as *mut libc::c_int,
                libc::WNOHANG,
            )
        };
        if pid < 0 {
            panic!("Error waiting: {}", Errno::last());
        }

        subproc::clean_after_child(jconf, pid).expect("could not clean_after_child");

        let mut mess: [u8; 19] = [0; 19];

        if let Some(ovdir) = ovdir {
            put_u32(&mut mess, 15, parse_uint32_cstr(ovdir.uid.as_c_str()));
            mess[2] = 5; // mess type

            // Do not forget in golang to delete btrfs subvolume and directory of deleted mount overlay
            // after doing needed OP like code saving in case of a compilation
            ovdir.kill().expect("could not kill ovdir in waiter");
        } else {
            mess[2] = 4; // mess type
        }

        mess[0] = (17 >> 8) as u8; // len with len uint16 excluded
        mess[1] = 17 as u8;
        put_u32(&mut mess, 3, exe_id);
        if unsafe { libc::WIFEXITED(self.wait_status) } {
            let mut status = unsafe { libc::WEXITSTATUS(self.wait_status) };
            if status == 0 {
                status = 1;
            }
            put_u32(&mut mess, 7, status as u32);
        }
        if unsafe { libc::WIFSIGNALED(self.wait_status) } {
            let kill_by_signal = unsafe { libc::WTERMSIG(self.wait_status) };
            put_u32(&mut mess, 11, kill_by_signal as u32);
        }

        gtvs_mess_buffer.write_mess(mess);
    }
}
