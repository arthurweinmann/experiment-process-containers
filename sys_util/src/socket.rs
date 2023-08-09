use super::errno::Errno;
use super::uio::IoVec;
use super::SyscallReturnCode;

use std::mem;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::ptr;

/**
 * Example how to use from nsjail: let (fd1, fd2) = socketpair(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0)
 */
pub fn socketpair(
    domain: libc::c_int,
    ty: libc::c_int,
    protocol: libc::c_int,
) -> Result<(libc::c_int, libc::c_int), std::io::Error> {
    let mut fds = [-1, -1];

    SyscallReturnCode(unsafe { libc::socketpair(domain, ty, protocol, fds.as_mut_ptr()) })
        .into_empty_result()?;

    Ok((fds[0], fds[1]))
}

pub fn socket(
    domain: libc::c_int,
    ty: libc::c_int,
    protocol: libc::c_int,
) -> Result<libc::c_int, Errno> {
    Errno::result(unsafe { libc::socket(domain, ty, protocol) })
}

pub fn sendmsg_unix_stream(socket_fd: i32, l: UnixStream, slice: &[u8; 8]) -> Result<usize, Errno> {
    let stream_fd = l.as_raw_fd();
    let iov = [IoVec::from_slice(slice)];

    sendmsg(socket_fd, &iov, stream_fd, 0)
}

pub fn sendmsg_unix_listener(
    socket_fd: i32,
    l: &UnixListener,
    slice: &mut [u8; 4],
) -> Result<usize, Errno> {
    let listener_fd = l.as_raw_fd();
    let iov = [IoVec::from_slice(slice)];

    sendmsg(socket_fd, &iov, listener_fd, 0)
}

pub fn sendmsg_unix_rawfd(
    socket_fd: i32,
    l: RawFd,
    slice: &mut [u8; 4],
) -> Result<usize, Errno> {
    let iov = [IoVec::from_slice(slice)];

    sendmsg(socket_fd, &iov, l, 0)
}

/// Send data in scatter-gather vectors to a socket, possibly accompanied
/// by ancillary data. Optionally direct the message at the given address,
/// as with sendto.
///
/// Allocates if cmsgs is nonempty.
pub fn sendmsg(
    fd: i32,
    iov: &[IoVec<&[u8]>],
    cmsgs: i32,
    flags: libc::c_int,
) -> Result<usize, Errno> {
    let scm_right = &[cmsgs];
    let scm_right_len = mem::size_of_val(scm_right);

    let capacity = unsafe { libc::CMSG_SPACE(scm_right_len as libc::c_uint) as usize };

    // First size the buffer needed to hold the cmsgs.  It must be zeroed,
    // because subsequent code will not clear the padding bytes.
    let mut cmsg_buffer = vec![0u8; capacity];

    let mhdr = pack_mhdr_to_send(&mut cmsg_buffer[..], iov, scm_right, scm_right_len);

    let ret = unsafe { libc::sendmsg(fd, &mhdr, flags) };

    Errno::result(ret).map(|r| r as usize)
}

pub fn sendmsg_unix_listener_many(
    socket_fd: i32,
    listeners: &[i32],
    slice: &mut [u8; 4],
) -> Result<usize, Errno> {
    let iov = [IoVec::from_slice(slice)];

    sendmsg_many(socket_fd, &iov, listeners, 0)
}

/// Send data in scatter-gather vectors to a socket, possibly accompanied
/// by ancillary data. Optionally direct the message at the given address,
/// as with sendto.
///
/// Allocates if cmsgs is nonempty.
pub fn sendmsg_many(
    fd: i32,
    iov: &[IoVec<&[u8]>],
    cmsgs: &[i32],
    flags: libc::c_int,
) -> Result<usize, Errno> {
    let scm_right = cmsgs;
    let scm_right_len = mem::size_of_val(scm_right);

    let capacity = unsafe { libc::CMSG_SPACE(scm_right_len as libc::c_uint) as usize };

    // First size the buffer needed to hold the cmsgs.  It must be zeroed,
    // because subsequent code will not clear the padding bytes.
    let mut cmsg_buffer = vec![0u8; capacity];

    let mhdr = pack_mhdr_to_send(&mut cmsg_buffer[..], iov, scm_right, scm_right_len);

    let ret = unsafe { libc::sendmsg(fd, &mhdr, flags) };

    Errno::result(ret).map(|r| r as usize)
}

fn pack_mhdr_to_send<'a>(
    cmsg_buffer: &mut [u8],
    iov: &[IoVec<&[u8]>],
    scm_right: &[i32], // only scm right are supported (file descriptor)
    scm_right_len: usize,
) -> libc::msghdr {
    let capacity = cmsg_buffer.len();

    // The message header must be initialized before the individual cmsgs.
    let cmsg_ptr = if capacity > 0 {
        cmsg_buffer.as_ptr() as *mut libc::c_void
    } else {
        ptr::null_mut()
    };

    let name: *const libc::sockaddr = ptr::null();

    let mhdr = unsafe {
        // Musl's msghdr has private fields, so this is the only way to
        // initialize it.
        let mut mhdr = mem::MaybeUninit::<libc::msghdr>::zeroed();
        let p = mhdr.as_mut_ptr();
        (*p).msg_name = name as *mut _;
        (*p).msg_namelen = 0;
        // transmute iov into a mutable pointer.  sendmsg doesn't really mutate
        // the buffer, but the standard says that it takes a mutable pointer
        (*p).msg_iov = iov.as_ptr() as *mut _;
        (*p).msg_iovlen = iov.len() as _;
        (*p).msg_control = cmsg_ptr;
        (*p).msg_controllen = capacity as _;
        (*p).msg_flags = 0;
        mhdr.assume_init()
    };

    // Encode each cmsg.  This must happen after initializing the header because
    // CMSG_NEXT_HDR and friends read the msg_control and msg_controllen fields.
    // CMSG_FIRSTHDR is always safe
    let mut pmhdr: *mut libc::cmsghdr =
        unsafe { libc::CMSG_FIRSTHDR(&mhdr as *const libc::msghdr) };

    assert_ne!(pmhdr, ptr::null_mut());

    unsafe {
        (*pmhdr).cmsg_level = libc::SOL_SOCKET;
        (*pmhdr).cmsg_type = libc::SCM_RIGHTS;
        (*pmhdr).cmsg_len = libc::CMSG_LEN(scm_right_len as libc::c_uint) as usize;
    }

    let data_ptr = scm_right as *const _ as *const u8;

    // Safe because we know that pmhdr is valid, and we initialized it with
    // sufficient space
    unsafe { ptr::copy_nonoverlapping(data_ptr, libc::CMSG_DATA(pmhdr), scm_right_len) };

    // Safe because mhdr is valid
    // pmhdr = unsafe { libc::CMSG_NXTHDR(&mhdr as *const libc::msghdr, pmhdr) };

    mhdr
}
