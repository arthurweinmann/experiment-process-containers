use std::ffi::CString;
/// Light & Simple Uni - loop scheduler
/// To scale, launch 1 uni-loop scheduler per processor core, since golang handles code download and comunication anyway so it can round robin the load across all such schedulers
/// This way we avoid the burden (time) of Rust Futures that we should implement at the same time we get completely rid of golang for virtualization by implementing the session
/// package in Rust
///
/// The other advantage is that every state (toaster timeout array, namespace pool, waiting child pid array, etc) are in the same function so no need for mutexes and atomics
/// We call wait syscall in a non blocking way so it can also be done sequentially in the loop when it is the most suited
///
/// Golang receives execution from LBs, so it also handles: toaster code download and gc, maybe some monitoring, runner/lb session proxy, maybe btrfs but we could do that in the rust loop too
///
/// function toastloop() is the one to call
///
/// Notes:
///     - with EpollFlags::EPOLLOUT epollwait never blocks, when implementing writes use epolloneshot flag and ream ctl event each time
///     - if it is too much of a performance bottleneck to clone child in the same loop as the rest, implement a pool of thread to specifically do this and shard clone calls
///
///
///
///
use std::os::unix::io::AsRawFd;

use sys_util::epoll::EpollEvent;

use super::commands_toaster::execute_toaster;

use super::gtvs_message::{GtvsMessageReader, GtvsMessageWriter};
use super::hash_table::HashTable;
use super::init::init_miscellaneous;
use super::net::{init_net_epoll, poll_fd_events};
use super::pool::NamespacePool;
use super::waiter::Waiter;
use jail::protobuf::parse_kill_mess;

#[derive(Debug, PartialEq)]
enum State {
    CheckFdEvent,
    BlockUntilFdEvent,
    ReadEpoll,
    ReadCommand,
    HandleCommand,
    KillTimedOut,
}

pub fn start() {
    let (
        local_cloud_provider,
        socket_path_incoming,
        socket_path_outgoing,
        num_cpus,
        non_root_uid,
        non_root_gid,
    ) = init_miscellaneous();

    let (
        listener,
        endpoint_read,
        endpoint_read_fd,
        endpoint_write,
        efd,
        gateway,
        nb_pool,
        pool_size,
    ) = init_net_epoll(&socket_path_incoming, &socket_path_outgoing);

    let gateway = CString::new(format!(
        "{}.{}.{}.{}",
        (gateway >> 24) as u8,
        (gateway >> 16) as u8,
        (gateway >> 8) as u8,
        gateway as u8
    ))
    .unwrap();

    let endpoint_read_fd_u64 = endpoint_read_fd as u64;
    let endpoint_write_fd_u64 = endpoint_write.as_raw_fd() as u64;

    let mut sendmsg_slice = [0u8; 4];

    let mut gtvs_mess_buffer_reader = GtvsMessageReader::new(endpoint_read);
    let mut gtvs_mess_buffer_writer = GtvsMessageWriter::new(endpoint_write, efd);
    let mut pid_hash_table = HashTable::new();
    let mut waiter = Waiter::new();

    let mut events = [EpollEvent::empty(); 64];
    let mut ready: usize = 0;

    let mut namespace_pool = NamespacePool::new(nb_pool as usize, pool_size as usize);

    let mut state = State::BlockUntilFdEvent;

    // let mut i = 0;
    loop {
        // i = i + 1;
        // if i == 100 {
        //     // be careful and test that the loop does not run continuously if there is nothing to do not to consume 100% of a cpu !!!!! you can block on epoll
        //     // remove or comment this test in production
        //     panic!("too many loop");
        // }

        match state {
            // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
            // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
            State::CheckFdEvent => {
                ready = poll_fd_events(efd, &mut events, 0);
                if ready > 0 {
                    state = State::ReadEpoll;
                } else {
                    state = State::KillTimedOut;
                }
            }

            // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
            // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
            State::BlockUntilFdEvent => {
                // println!("+++++++++++++ BlockUntilFdEvent");
                ready = poll_fd_events(efd, &mut events, -1);
                if ready > 0 {
                    state = State::ReadEpoll;
                }
            }

            // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
            // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
            State::ReadEpoll => {
                for i in 0..ready {
                    match events[i].data() {
                        x if x == endpoint_read_fd_u64 => {
                            if events[i].is_read_closed() {
                                panic!("endpoint read was closed");
                            }

                            if events[i].is_error() {
                                panic!("endpoint read is in error state");
                            }

                            if events[i].is_readable() {
                                if gtvs_mess_buffer_reader.read() {
                                    state = State::HandleCommand;
                                }
                            }
                        }
                        x if x == endpoint_write_fd_u64 => {
                            if events[i].is_write_closed() {
                                panic!("endpoint write was closed");
                            }

                            if events[i].is_error() {
                                panic!("endpoint write is in error state");
                            }

                            gtvs_mess_buffer_writer.efd_armed = false;
                            if gtvs_mess_buffer_writer.remaining_data_to_write()
                                && events[i].is_writable()
                            {
                                gtvs_mess_buffer_writer.try_write();
                            }
                        }
                        pid => {
                            // println!("pid triggered: {} {}", pid, i);
                            waiter.wait_pid(
                                pid as i32,
                                &mut gtvs_mess_buffer_writer,
                                &mut pid_hash_table,
                            );
                        }
                    };
                }

                if state == State::ReadEpoll {
                    state = State::KillTimedOut;
                }
            }

            State::ReadCommand => {
                if gtvs_mess_buffer_reader.read() {
                    state = State::HandleCommand;
                } else {
                    state = State::KillTimedOut;
                }
            }

            // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
            // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
            State::HandleCommand => {
                // println!("------------ HandleCommand");
                match gtvs_mess_buffer_reader.get_message()[0] {
                    // 1 => create_image_handler(
                    //     &local_cloud_provider,
                    //     &mut gtvs_mess_buffer_reader,
                    //     &mut gtvs_mess_buffer_writer,
                    //     &socket_path_outgoing,
                    //     num_cpus,
                    //     efd,
                    //     &mut namespace_pool,
                    //     &mut pid_hash_table,
                    //     &mut ip_allocator,
                    //     &mut waiter,
                    //     &gateway,
                    // ),
                    2 => execute_toaster(
                        &local_cloud_provider,
                        &gtvs_mess_buffer_reader,
                        &mut gtvs_mess_buffer_writer,
                        num_cpus,
                        efd,
                        &mut namespace_pool,
                        &mut pid_hash_table,
                        &gateway,
                        non_root_uid,
                        non_root_gid,
                        endpoint_read_fd,
                        &mut sendmsg_slice,
                        &mut waiter,
                    ),
                    3 => {
                        let mess = gtvs_mess_buffer_reader.get_message();
                        let (exe_id, timeout_sec) =
                            parse_kill_mess(mess).expect("invalid kill message");
                        if let Some(pid) = pid_hash_table.pop_exe_id(exe_id) {
                            waiter.kill_pid(pid as i32, timeout_sec, &mut pid_hash_table);
                        }
                    }
                    _ => panic!("invalid mess first byte"),
                }

                // once done with the message, not before
                if gtvs_mess_buffer_reader.reset_for_next_message() {
                    state = State::HandleCommand;
                } else if gtvs_mess_buffer_reader.more_data() {
                    state = State::ReadCommand;
                } else {
                    state = State::KillTimedOut;
                }
            }

            // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
            // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
            State::KillTimedOut => {
                waiter.kill_timed_out(&mut pid_hash_table);
                state = State::BlockUntilFdEvent;
            }
        }
    }
}
