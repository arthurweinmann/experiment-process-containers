use std::ffi::{CStr, CString};
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::os::unix::net::{UnixListener, UnixStream};

use super::config::{create_toaster_jconf, create_toaster_pool_jconf, set_jconf_as_join};
use super::gtvs_message::{GtvsMessageReader, GtvsMessageWriter};
use super::hash_table::{HashTable, Item};
use super::net::connect_unix_blocking;
use super::pool::{Item as PoolItem, NamespacePool};
use super::waiter::Waiter;
use jail::protobuf::parse_toaster_command;
use jail::protobuf::put_u32;

use jail::subproc;

use disk::overlay_fs::OverlayDir;

use sys_util::epoll::{epoll_ctl, EpollEvent, EpollFlags, EpollOp};
use sys_util::errno::Errno;
use sys_util::fcntl::open;
use sys_util::socket::{sendmsg_unix_listener, sendmsg_unix_rawfd, socketpair};
use sys_util::unistd::chown_cstr;

static LISTENER_PATH_NUL_TERMINATED: &'static [u8] = b"/toastate.sock\0";

pub fn execute_toaster<'a>(
    local_cloud_provider: &str,
    gtvs_mess_buffer_reader: &GtvsMessageReader,
    gtvs_mess_buffer_writer: &mut GtvsMessageWriter,
    num_cpus: i64,
    efd: i32,
    toaster_pool: &mut NamespacePool<'a>,
    hash_table: &mut HashTable<'a>,
    gw: &'a CStr,
    non_root_owner: libc::uid_t,
    non_root_group: libc::gid_t,
    read_endpoint_fd: i32,
    sendmsg_slice: &mut [u8; 4],
    waiter: &mut Waiter,
) {
    let (
        exe_id,
        pool,
        uid,
        btrfs_file_system,
        overlay_dir,
        lower_dirs,
        cwd,
        log_path,
        is_log_socket,
        is_socket_stdin,
        std_only,
        admin,
        command_name,
        command_args,
        env,
        ip,
    ) = get_toaster_message(gtvs_mess_buffer_reader);

    if let Some(cmd_name) = command_name {
        immediate_execution(
            local_cloud_provider,
            num_cpus,
            efd,
            toaster_pool,
            hash_table,
            gw,
            non_root_owner,
            non_root_group,
            read_endpoint_fd,
            sendmsg_slice,
            gtvs_mess_buffer_writer,
            waiter,
            exe_id,
            pool,
            uid,
            btrfs_file_system,
            overlay_dir,
            lower_dirs,
            cwd,
            log_path,
            is_log_socket,
            is_socket_stdin,
            std_only,
            admin,
            cmd_name,
            command_args,
            env,
            ip,
        );

        return;
    }

    create_pool_toaster(
        local_cloud_provider,
        num_cpus,
        non_root_owner,
        non_root_group,
        gw,
        gtvs_mess_buffer_writer,
        toaster_pool,
        pool,
        exe_id,
        btrfs_file_system,
        cwd,
        uid,
        ip,
        overlay_dir,
        admin,
        log_path,
    );
}

fn create_pool_toaster<'a>(
    local_cloud_provider: &str,
    num_cpus: i64,
    non_root_owner: libc::uid_t,
    non_root_group: libc::gid_t,
    gw: &'a CStr,
    gtvs_mess_buffer_writer: &mut GtvsMessageWriter,
    toaster_pool: &mut NamespacePool<'a>,
    // -
    // -
    // -
    pool: u16,
    exe_id: u32,
    btrfs_file_system: CString,
    cwd: String,
    uid: CString,
    ip: CString,
    overlay_dir: &[u8],
    admin: bool,
    log_path: Option<&str>,
) {
    if pool == 0 {
        panic!("pool cannot be null when no command name is provided");
    }

    let mut pool_item = create_pool_item(
        local_cloud_provider,
        btrfs_file_system,
        uid,
        ip,
        cwd,
        num_cpus,
        overlay_dir,
        non_root_owner,
        non_root_group,
        gw,
        admin,
        false,
    );

    if let Some(log_path) = log_path {
        let logfd =
            open(log_path, libc::O_APPEND | libc::O_RDWR, 0700).expect("could not open log file");
        pool_item.jconf.fd_out = logfd;
        pool_item.jconf.fd_err = logfd;
    }

    subproc::run_child_pidfd(&mut pool_item.jconf, subproc::child)
        .expect("could not create pooled thread");

    set_jconf_as_join(&mut pool_item.jconf);

    toaster_pool.push((pool - 1) as usize, pool_item);

    let mut mess: [u8; 19] = [0; 19];
    mess[0] = (17 >> 8) as u8; // len with len uint16 excluded
    mess[1] = 17 as u8;
    mess[2] = 100;
    put_u32(&mut mess, 3, exe_id);
    gtvs_mess_buffer_writer.write_mess(mess);
}

fn immediate_execution<'a>(
    local_cloud_provider: &str,
    num_cpus: i64,
    efd: i32,
    toaster_pool: &mut NamespacePool<'a>,
    hash_table: &mut HashTable<'a>,
    gw: &'a CStr,
    non_root_owner: libc::uid_t,
    non_root_group: libc::gid_t,
    read_endpoint_fd: i32,
    sendmsg_slice: &mut [u8; 4],
    gtvs_mess_buffer_writer: &mut GtvsMessageWriter,
    waiter: &mut Waiter,
    // -
    // -
    // -
    exe_id: u32,
    pool: u16,
    uid: CString,
    btrfs_file_system: CString,
    overlay_dir: &[u8],
    lower_dirs: Option<&[u8]>,
    cwd: String,
    log_path: Option<&str>,
    is_log_socket: bool,
    is_socket_stdin: bool,
    std_only: bool,
    admin: bool,
    command_name: CString,
    command_args: Option<Vec<CString>>,
    env: Option<Vec<CString>>,
    ip: CString,
) {
    let mut item = if pool > 0 && toaster_pool.len((pool - 1) as usize) > 0 {
        let mut item = toaster_pool.pop((pool - 1) as usize);
        item.jconf.cwd = cwd;
        if item.ovdir.uid != uid {
            panic!("{:?} != {:?}", item.ovdir.uid, uid)
        }
        item
    } else {
        create_pool_item(
            local_cloud_provider,
            btrfs_file_system,
            uid,
            ip,
            cwd,
            num_cpus,
            overlay_dir,
            non_root_owner,
            non_root_group,
            gw,
            admin,
            true,
        )
    };

    mount_overlayfs(lower_dirs, &mut item);

    let mut execution_listener = None;
    let mut std_sock = None;
    if std_only {
        if is_socket_stdin || is_log_socket {
            std_sock = Some(handle_out_socket(read_endpoint_fd, sendmsg_slice, exe_id));
        }
    } else {
        execution_listener = Some(handle_toaster_execution_listener(
            &item,
            non_root_owner,
            non_root_group,
            read_endpoint_fd,
            sendmsg_slice,
            exe_id,
        ));
    }

    if let Some(std_sock) = std_sock {
        if is_socket_stdin {
            item.jconf.fd_in = std_sock;
        }

        if !log_path.is_some() && is_log_socket {
            item.jconf.fd_out = std_sock;
            item.jconf.fd_err = std_sock;
        }
    }

    if let Some(log_path) = log_path {
        let logfd =
            open(log_path, libc::O_APPEND | libc::O_RDWR, 0700).expect("could not open log file");
        item.jconf.fd_out = logfd;
        item.jconf.fd_err = logfd;
    }

    item.jconf.exec_file = Some(command_name);
    item.jconf.argv = command_args;
    item.jconf.env = env;

    let (child_pid, child_pidfd) = match subproc::run_child_pidfd(&mut item.jconf, subproc::child) {
        Ok(v) => v,
        Err(jail::error::Error::ParsePid((p, _, _))) => {
            waiter.wait_pid_from_err(
                p,
                exe_id,
                gtvs_mess_buffer_writer,
                &item.jconf,
                Some(item.ovdir),
            );
            return;
        }
        Err(e) => {
            panic!("{}", e);
        }
    };

    epoll_register_running_child(child_pid as u64, efd, child_pidfd);

    hash_table.insert(Item {
        pid: child_pid,
        exe_id: exe_id,
        jconf: item.jconf,
        ovdir: Some(item.ovdir),
        toaster_listener: execution_listener,
    });
}

fn get_toaster_message<'a>(
    gtvs_mess_buffer_reader: &'a GtvsMessageReader,
) -> (
    u32,
    u16,
    CString,
    CString,
    &'a [u8],
    Option<&'a [u8]>,
    String,
    Option<&'a str>,
    bool,
    bool,
    bool,
    bool,
    Option<CString>,
    Option<Vec<CString>>,
    Option<Vec<CString>>,
    CString,
) {
    let mess = gtvs_mess_buffer_reader.get_message();
    parse_toaster_command(mess)
}

fn create_pool_item<'a>(
    local_cloud_provider: &str,
    btrfs_file_system: CString,
    uid: CString,
    ip: CString,
    cwd: String,
    num_cpus: i64,
    overlay_dir: &[u8],
    non_root_owner: libc::uid_t,
    non_root_group: libc::gid_t,
    gw: &'a CStr,
    admin: bool,
    immediate_execution: bool,
) -> PoolItem<'a> {
    let ovdir = OverlayDir::new(
        btrfs_file_system,
        overlay_dir,
        uid,
        non_root_owner,
        non_root_group,
        !immediate_execution,
    )
    .expect("could not create overlaydir");

    let jconf = if immediate_execution {
        create_toaster_jconf(
            local_cloud_provider,
            ovdir.mount_point.clone(),
            num_cpus,
            ip,
            gw,
            cwd,
            false,
            admin,
        )
    } else {
        create_toaster_pool_jconf(
            local_cloud_provider,
            ovdir.mount_point.clone(),
            num_cpus,
            ip,
            gw,
            cwd,
            true,
            admin,
        )
    };

    PoolItem {
        jconf: jconf,
        ovdir: ovdir,
    }
}

fn mount_overlayfs(lower_dirs: Option<&[u8]>, item: &mut PoolItem) {
    if let Some(lower_dirs) = lower_dirs {
        item.ovdir
            .mount(lower_dirs)
            .expect("could not mount overlay ovdir");
    } else {
        panic!("no lower_dirs provided");
    }
}

fn handle_out_socket(read_endpoint_fd: i32, sendmsg_slice: &mut [u8; 4], exe_id: u32) -> i32 {
    // let us = connect_unix_blocking(socket_path_pass_conn).unwrap();

    let (child_fd, parent_fd) =
        socketpair(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0)
            .expect("could not create socketpair in handle_out_socket");

    put_u32(sendmsg_slice, 0, exe_id);

    sendmsg_unix_rawfd(read_endpoint_fd, parent_fd, sendmsg_slice)
        .expect("could not send socketpair in handle_out_socket");

    unsafe { libc::close(parent_fd) };

    child_fd
}

fn handle_toaster_execution_listener(
    item: &PoolItem,
    non_root_owner: libc::uid_t,
    non_root_group: libc::gid_t,
    read_endpoint_fd: i32,
    sendmsg_slice: &mut [u8; 4],
    exe_id: u32,
) -> UnixListener {
    let mpp = [
        item.ovdir.mount_point.to_bytes(),
        LISTENER_PATH_NUL_TERMINATED,
    ]
    .concat();

    let p_cstr = unsafe { CStr::from_bytes_with_nul_unchecked(mpp.as_slice()) };

    let p = unsafe { std::str::from_utf8_unchecked(&mpp[..mpp.len() - 1]) };

    let lst = UnixListener::bind(p)
        .map_err(|e| format!("{}: {}", e, p))
        .expect("could not establish listener in handle_toaster_execution_listener");

    chown_cstr(p_cstr, non_root_owner, non_root_group).expect("could not chown toaster listener");

    put_u32(sendmsg_slice, 0, exe_id);

    sendmsg_unix_listener(read_endpoint_fd, &lst, sendmsg_slice)
        .expect("could not send listener in handle_toaster_execution_listener");

    lst
}

pub fn epoll_register_running_child(child_pid: u64, epfd: RawFd, child_pidfd: RawFd) {
    let mut event = EpollEvent::new(EpollFlags::EPOLLIN | EpollFlags::EPOLLONESHOT, child_pid);
    epoll_ctl(epfd, EpollOp::EpollCtlAdd, child_pidfd, &mut event)
        .expect("could not register child_pidfd into epoll");
}
