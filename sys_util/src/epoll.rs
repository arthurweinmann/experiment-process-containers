use super::errno::Errno;
use libc::{self, c_int};
use std::os::unix::io::RawFd;
use std::ptr;
use std::mem;

libc_bitflags!(
    pub struct EpollFlags: c_int {
        EPOLLIN;
        EPOLLPRI;
        EPOLLOUT;
        EPOLLRDNORM;
        EPOLLRDBAND;
        EPOLLWRNORM;
        EPOLLWRBAND;
        EPOLLMSG;
        EPOLLERR;
        EPOLLHUP;
        EPOLLRDHUP;
        #[cfg(target_os = "linux")]  // Added in 4.5; not in Android.
        EPOLLEXCLUSIVE;
        #[cfg(not(target_arch = "mips"))]
        EPOLLWAKEUP;
        EPOLLONESHOT;
        EPOLLET;
    }
);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum EpollOp {
    EpollCtlAdd = libc::EPOLL_CTL_ADD,
    EpollCtlDel = libc::EPOLL_CTL_DEL,
    EpollCtlMod = libc::EPOLL_CTL_MOD,
}

libc_bitflags!{
    pub struct EpollCreateFlags: c_int {
        EPOLL_CLOEXEC;
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(C)]
pub struct EpollEvent {
    event: libc::epoll_event,
}

impl EpollEvent {
    pub fn new(events: EpollFlags, data: u64) -> Self {
        EpollEvent { event: libc::epoll_event { events: events.bits() as u32, u64: data } }
    }

    pub fn empty() -> Self {
        unsafe { mem::zeroed::<EpollEvent>() }
    }

    pub fn events(&self) -> EpollFlags {
        EpollFlags::from_bits(self.event.events as c_int).unwrap()
    }

    pub fn data(&self) -> u64 {
        self.event.u64
    }

    pub fn is_readable(&self) -> bool {
        (self.event.events as libc::c_int & libc::EPOLLIN) != 0
            || (self.event.events as libc::c_int & libc::EPOLLPRI) != 0
    }

    pub fn is_writable(&self) -> bool {
        (self.event.events as libc::c_int & libc::EPOLLOUT) != 0
    }

    pub fn is_error(&self) -> bool {
        (self.event.events as libc::c_int & libc::EPOLLERR) != 0
    }

    pub fn is_read_closed(&self) -> bool {
        // Both halves of the socket have closed
        self.event.events as libc::c_int & libc::EPOLLHUP != 0
            // Socket has received FIN or called shutdown(SHUT_RD)
            || (self.event.events as libc::c_int & libc::EPOLLIN != 0
                && self.event.events as libc::c_int & libc::EPOLLRDHUP != 0)
    }

    pub fn is_write_closed(&self) -> bool {
        // Both halves of the socket have closed
        self.event.events as libc::c_int & libc::EPOLLHUP != 0
            // Unix pipe write end has closed
            || (self.event.events as libc::c_int & libc::EPOLLOUT != 0
                && self.event.events as libc::c_int & libc::EPOLLERR != 0)
    }

    pub fn is_priority(&self) -> bool {
        (self.event.events as libc::c_int & libc::EPOLLPRI) != 0
    }
}

#[inline]
pub fn epoll_create() -> Result<RawFd, Errno> {
    let res = unsafe { libc::epoll_create(1024) };

    Errno::result(res)
}

#[inline]
pub fn epoll_create1(flags: EpollCreateFlags) -> Result<RawFd, Errno> {
    let res = unsafe { libc::epoll_create1(flags.bits()) };

    Errno::result(res)
}

#[inline]
pub fn epoll_ctl<'a, T>(epfd: RawFd, op: EpollOp, fd: RawFd, event: T) -> Result<(), Errno>
    where T: Into<Option<&'a mut EpollEvent>>
{
    let mut event: Option<&mut EpollEvent> = event.into();
    if event.is_none() && op != EpollOp::EpollCtlDel {
        Err(Errno::EINVAL)
    } else {
        let res = unsafe {
            if let Some(ref mut event) = event {
                libc::epoll_ctl(epfd, op as c_int, fd, &mut event.event)
            } else {
                libc::epoll_ctl(epfd, op as c_int, fd, ptr::null_mut())
            }
        };
        Errno::result(res).map(drop)
    }
}

#[inline]
pub fn epoll_wait(epfd: RawFd, events: &mut [EpollEvent], timeout_ms: isize) -> Result<usize, Errno> {
    let res = unsafe {
        libc::epoll_wait(epfd, events.as_mut_ptr() as *mut libc::epoll_event, events.len() as c_int, timeout_ms as c_int)
    };

    Errno::result(res).map(|r| r as usize)
}

/*

Resources: 

- https://github.com/nix-rust/nix/blob/master/src/sys/epoll.rs
- https://github.com/tokio-rs/mio/blob/master/src/sys/unix/selector/epoll.rs
- http://man7.org/linux/man-pages/man2/epoll_wait.2.html
- https://github.com/nathansizemore/epoll/blob/master/src/lib.rs#L19
- https://github.com/Gilnaa/epoll-rs/blob/master/src/lib.rs 
- https://github.com/nix-rust/nix/blob/master/src/sys/epoll.rs 

*/ 