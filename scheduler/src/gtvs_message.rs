use std::convert::TryInto;
use std::ffi::CStr;
use std::io::{ErrorKind, Read, Write};
use std::ops::Range;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use std::ptr::copy;

pub const GTVSMESSAGEMAXSIZE: usize = 8192;

use sys_util::epoll::{epoll_ctl, EpollEvent, EpollFlags, EpollOp};

use jail::protobuf::{extract_u16, parse_uint32_cstr, put_u32};

pub struct GtvsMessageReader {
    inner: [u8; GTVSMESSAGEMAXSIZE],
    received: usize,
    size: usize,
    remaining_data: bool,
    conn_read: UnixStream,
}

pub struct GtvsMessageWriter {
    efd: i32,
    pub efd_armed: bool,
    writer_event: sys_util::epoll::EpollEvent,
    writer_fd: i32,
    conn_write: UnixStream,
    buffer: Vec<[u8; 19]>,
    write_index: usize,
}

impl GtvsMessageWriter {
    pub fn new(conn_write: UnixStream, efd: i32) -> GtvsMessageWriter {
        GtvsMessageWriter {
            efd: efd,
            efd_armed: false,
            writer_event: EpollEvent::new(
                EpollFlags::EPOLLOUT | EpollFlags::EPOLLERR | EpollFlags::EPOLLONESHOT,
                conn_write.as_raw_fd() as u64,
            ),
            writer_fd: conn_write.as_raw_fd(),
            conn_write: conn_write,
            buffer: Vec::with_capacity(8),
            write_index: 0,
        }
    }

    /// write return true if data remains to be written, false if it wrote all provided mess and all data remaining in its internal buffer
    pub fn write_mess(&mut self, mess: [u8; 19]) -> bool {
        if mess.len() > GTVSMESSAGEMAXSIZE {
            panic!("gtvs write mess mess.len() > GTVSMESSAGEMAXSIZE")
        }
        self.buffer.insert(0, mess);
        self.try_write()
    }

    /// write return true if data remains to be written, false if it wrote all data remaining in its internal buffer
    pub fn try_write(&mut self) -> bool {
        if self.buffer.len() == 0 {
            return false;
        }

        loop {
            let last = &self.buffer[self.buffer.len() - 1];

            let n = match self.conn_write.write(&last[self.write_index..]) {
                Ok(v) => v,
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        if !self.efd_armed {
                            epoll_ctl(
                                self.efd,
                                EpollOp::EpollCtlMod,
                                self.writer_fd,
                                &mut self.writer_event,
                            )
                            .expect("could not EpollCtlMod unixStream writer into epoll");
                            self.efd_armed = true;
                        }
                        return true;
                    }
                    panic!(format!("error writing to unix socket: {}", e))
                }
            };
            // println!("written {} / {} bytes back to gtvs", n, last.len());

            if n == 0 {
                panic!("try_write n == 0");
            }

            self.write_index = self.write_index + n;
            if self.write_index == last.len() {
                self.write_index = 0;
                self.buffer.truncate(self.buffer.len() - 1);
            }

            if self.buffer.len() == 0 {
                return false;
            }
        }
    }

    pub fn remaining_data_to_write(&self) -> bool {
        self.buffer.len() > 0
    }
}

impl GtvsMessageReader {
    pub fn new(conn_read: UnixStream) -> GtvsMessageReader {
        GtvsMessageReader {
            inner: [0; GTVSMESSAGEMAXSIZE],
            received: 0,
            size: 0,
            remaining_data: false,
            conn_read: conn_read,
        }
    }

    /// read return true it read a complete message, false otherwise
    pub fn read<'a>(&'a mut self) -> bool {
        loop {
            // println!("about to read");
            let n = match if self.size == 0 {
                self.conn_read.read(&mut self.inner[0..2])
            } else {
                self.conn_read.read(&mut self.inner[self.received..])
            } {
                Ok(v) => v,
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        // println!("read would block returning");
                        self.remaining_data = false;
                        return false;
                    }
                    panic!(format!("error reading from unix socket: {}", e))
                }
            };
            // println!("n: {}/{}", n, self.size);
            if n == 0 {
                panic!("gtvs reader returned 0 bytes, socket close etc should be handled in epoll")
            }
            self.received = self.received + n;
            if self.size == 0 {
                if self.received < 2 {
                    continue;
                }
                self.size = extract_u16(&self.inner, 0) as usize;
                // println!(
                //     "decoded size {}, {} {}",
                //     self.size, &self.inner[0], &self.inner[1]
                // );
                self.received = 0;
            } else if self.received >= self.size {
                self.remaining_data = true;
                return true;
            }
        }
    }

    pub fn more_data(&self) -> bool {
        self.remaining_data
    }

    pub fn message_available(&self) -> bool {
        self.size > 0 && self.received >= self.size as usize
    }

    pub fn get_message(&self) -> &[u8] {
        &self.inner[..self.size as usize]
    }

    pub fn reset_for_next_message(&mut self) -> bool {
        let mut diff = self.received - self.size as usize;
        if diff > 0 {
            if diff >= 2 {
                let tmp_size = u16::from_be_bytes(
                    self.inner[self.size as usize..self.size as usize + 2]
                        .try_into()
                        .expect("slice with incorrect length"),
                ) as usize;
                diff = diff - 2;
                if diff > 0 {
                    move_memory(&mut self.inner, self.size as usize + 2..self.received, 0);
                }
                self.size = tmp_size;
            } else {
                move_memory(&mut self.inner, self.size as usize..self.received, 0);
                self.size = 0;
            }
            self.received = diff;
            if self.size > 0 && self.received >= self.size as usize {
                return true;
            }
        } else {
            self.received = 0;
            self.size = 0;
        }

        false
    }
}

/// Copy the range `data[from]` onto the index `to` and following
///
/// **Panics** if `from` or `to` is out of bounds
pub fn move_memory<T: Copy>(data: &mut [T], from: Range<usize>, to: usize) {
    assert!(from.start <= from.end);
    assert!(from.end <= data.len());
    assert!(to <= data.len() - (from.end - from.start));
    unsafe {
        let ptr = data.as_mut_ptr();
        copy(
            ptr.offset(from.start as isize),
            ptr.offset(to as isize),
            from.end - from.start,
        )
    }
}
