use libc::{c_int, c_void};
use std::ffi::CStr;

#[repr(C)]
pub struct nl_sock {
    _private: [u8; 0],
} // without field _private, RUST warns that the struct is not FFI safe, see https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs

#[link(name = "nl-3")]
extern "C" {
    // Exposed socket functions
    fn nl_socket_alloc() -> *const nl_sock;
    fn nl_socket_free(socket: *const nl_sock);
    fn nl_connect(socket: *const nl_sock, protocol: i32) -> i32;
    fn nl_geterror(err: i32) -> *const libc::c_char;
    fn nl_close(socket: *const nl_sock);
}

pub struct NetlinkSocket {
    pub ptr: *const nl_sock, // test for memory leak since I do no think rust can free this - see readme for this package
}

pub fn alloc() -> Option<NetlinkSocket> {
    let ptr = unsafe { nl_socket_alloc() };

    match ptr as isize {
        0x0 => None, // nl_socket_alloc returns NULL in case of an error
        _ => Some(NetlinkSocket { ptr: ptr }),
    }
}

pub fn free(sock: NetlinkSocket) {
    unsafe {
        nl_socket_free(sock.ptr);
    }
}

pub fn connect(sock: &mut NetlinkSocket, protocol: i32) -> i32 {
    unsafe { nl_connect(sock.ptr, protocol) }
}

pub fn get_error(err: i32) -> String {
    let c_buf: *const libc::c_char = unsafe{ nl_geterror(err) };
    let c_str: &CStr = unsafe{ CStr::from_ptr(c_buf)};
    let str_slice: &str = c_str.to_str().unwrap();
    let str_buf: String = str_slice.to_owned();
    str_buf
}

pub fn close(sock: &mut NetlinkSocket) {
    unsafe { nl_close(sock.ptr) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alloc_test() {
        if let Some(sock) = alloc() {
            free(sock);
        } else {
            panic!("no socket in option");
        }
    }
}
