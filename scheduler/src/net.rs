use std::io::Read;
use std::io::Result as IOResult;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};

use sys_util::epoll::{
    epoll_create1, epoll_ctl, epoll_wait, EpollCreateFlags, EpollEvent, EpollFlags, EpollOp,
};

use jail::protobuf::extract_u32;

pub fn init_net_epoll(
    socket_path_incoming: &str,
    socket_path_outgoing: &str,
) -> (
    UnixListener,
    UnixStream,
    RawFd,
    UnixStream,
    RawFd,
    u32,
    u32,
    u32,
) {
    let listener = match UnixListener::bind(socket_path_incoming) {
        Ok(v) => v,
        Err(e) => panic!(format!(
            "Could not start listening for unix connection: {}",
            e
        )),
    };

    let endpoint_write = connect_unix(socket_path_outgoing)
        .expect("Could not set write unix socket to non blocking mode");

    let mut endpoint_read = match listener.accept() {
        Ok((socket, _)) => socket,
        Err(e) => panic!(format!("unix accept function failed: {}", e)),
    };

    let mut init_mess: [u8; 12] = [0; 12];
    if endpoint_read
        .read(&mut init_mess)
        .expect("could not read initialization message")
        != 12
    {
        panic!("could not read entire initialization message");
    }
    let gateway = extract_u32(&init_mess, 0);
    let nb_pool = extract_u32(&init_mess, 4);
    let pool_size = extract_u32(&init_mess, 8);

    match endpoint_read.set_nonblocking(true) {
        Ok(v) => v,
        Err(e) => panic!(format!(
            "Could not set read unix socket to non blocking mode: {}",
            e
        )),
    };

    let endpoint_read_fd = endpoint_read.as_raw_fd();

    let efd = epoll_create1(EpollCreateFlags::EPOLL_CLOEXEC).expect("could not create epoll");
    let mut event = EpollEvent::new(
        EpollFlags::EPOLLIN | EpollFlags::EPOLLERR,
        endpoint_read_fd as u64,
    );
    epoll_ctl(efd, EpollOp::EpollCtlAdd, endpoint_read_fd, &mut event)
        .expect("could not register unixStream into epoll");

    println!("registered unix socket fd {:?} to epoll", endpoint_read_fd);

    (
        listener,
        endpoint_read,
        endpoint_read_fd,
        endpoint_write,
        efd,
        gateway,
        nb_pool,
        pool_size,
    )
}

pub fn connect_unix(socket_path: &str) -> IOResult<UnixStream> {
    let endpoint = UnixStream::connect(socket_path)
        .expect(format!("unix: could not connect back: {}", socket_path).as_str());
    match endpoint.set_nonblocking(true) {
        Ok(_) => {}
        Err(e) => panic!(format!(
            "Could not set write unix socket to non blocking mode: {}",
            e
        )),
    };
    Ok(endpoint)
}

pub fn connect_unix_blocking(socket_path: &str) -> IOResult<UnixStream> {
    Ok(UnixStream::connect(socket_path)
        .expect(format!("unix: could not connect back: {}", socket_path).as_str()))
}

pub fn poll_fd_events(epfd: RawFd, events: &mut [EpollEvent], timeout_ms: isize) -> usize {
    match epoll_wait(epfd, events, timeout_ms) {
        Ok(v) => v,
        Err(e) => panic!(format!("epoll wait err {}", e)),
    }
}
