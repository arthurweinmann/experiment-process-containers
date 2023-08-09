use super::socket;
use cpp_bindings;
use std::ffi::CStr;
use std::mem::MaybeUninit;

#[repr(C)]
pub struct nl_cache {
    _private: [u8; 0],
} // without field _private, RUST warns that the struct is not FFI safe, see https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs

#[repr(C)]
pub struct rtnl_link {
    _private: [u8; 0],
} // without field _private, RUST warns that the struct is not FFI safe, see https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs

#[link(name = "nl-route-3")]
extern "C" {
    fn rtnl_link_alloc_cache(
        socket: *const socket::nl_sock,
        addr_family: i32,
        cache: *const *const nl_cache,
    ) -> i32;
    fn nl_cache_free(cache: *const nl_cache);

    fn rtnl_link_get_by_name(cache: *const nl_cache, name: *const libc::c_char)
        -> *const rtnl_link;
    fn rtnl_link_alloc() -> *const rtnl_link;
    fn rtnl_link_put(link: *const rtnl_link);
    fn rtnl_link_set_ns_pid(link: *const rtnl_link, pid: libc::pid_t);
    fn rtnl_link_set_name(link: *const rtnl_link, name: *const libc::c_char);
    fn rtnl_link_set_link(link: *const rtnl_link, index: libc::c_int);
    fn rtnl_link_change(
        sk: *const socket::nl_sock,
        orig: *const rtnl_link,
        changes: *const rtnl_link,
        flags: libc::c_int,
    ) -> i32;
    fn rtnl_link_macvlan_alloc() -> *const rtnl_link;
    fn rtnl_link_name2i(cache: *const nl_cache, name: *const libc::c_char) -> i32;
    fn rtnl_link_add(
        socket: *const socket::nl_sock,
        link: *const rtnl_link,
        flags: libc::c_int,
    ) -> i32;
}

pub struct NetlinkCache {
    pub ptr: *const nl_cache, // test for memory leak since I do no think rust can free this - see readme for this package
}

pub struct RtnlLink {
    pub ptr: *const rtnl_link, // test for memory leak since I do no think rust can free this - see readme for this package
}

// the needed value is in a C unnamed enum, how to link it in RUST ?
pub fn global_rtm_setlink() -> libc::c_int {
    cpp_bindings::get_libnl_rtm_setlink()
}

pub fn global_nlm_f_create() -> libc::c_int {
    cpp_bindings::get_libnl_nlm_f_create() /* Create, if it does not exist	*/
}

// test this :thinking
pub fn alloc_cache(sock: &mut socket::NetlinkSocket, addr_family: i32) -> Option<NetlinkCache> {
    let mut link_cache = MaybeUninit::<*const nl_cache>::uninit();
    let res = unsafe {
        rtnl_link_alloc_cache(
            sock.ptr,
            addr_family,
            link_cache.as_mut_ptr() as *const *const nl_cache,
        )
    };

    if res != 0 {
        return None;
    }

    let link_cache = unsafe { link_cache.assume_init() };
    Some(NetlinkCache { ptr: link_cache })
}

pub fn free_cache(cache: NetlinkCache) {
    unsafe { nl_cache_free(cache.ptr) } // as *const nl_cache
}

pub fn link_get_by_name(cache: &NetlinkCache, name: &CStr) -> Option<RtnlLink> {
    let ptr = unsafe { rtnl_link_get_by_name(cache.ptr, name.as_ptr()) };

    match ptr as isize {
        0x0 => None, // nl_socket_alloc returns NULL in case of an error
        _ => Some(RtnlLink { ptr: ptr }),
    }
}

pub fn link_alloc() -> Option<RtnlLink> {
    let ptr = unsafe { rtnl_link_alloc() };

    match ptr as isize {
        0x0 => None, // nl_socket_alloc returns NULL in case of an error
        _ => Some(RtnlLink { ptr: ptr }),
    }
}

pub fn link_macvlan_alloc() -> Option<RtnlLink> {
    let ptr = unsafe { rtnl_link_macvlan_alloc() };

    match ptr as isize {
        0x0 => None, // nl_socket_alloc returns NULL in case of an error
        _ => Some(RtnlLink { ptr: ptr }),
    }
}

pub fn link_name2i(cache: &NetlinkCache, name: &CStr) -> Option<i32> {
    let res = unsafe { rtnl_link_name2i(cache.ptr, name.as_ptr()) };
    if res == 0 {
        return None;
    }
    Some(res)
}

pub fn link_add(sock: &socket::NetlinkSocket, link: &RtnlLink, flags: libc::c_int) -> bool {
    if unsafe { rtnl_link_add(sock.ptr, link.ptr, flags) } < 0 {
        return false;
    }
    true
}

pub fn link_set_name(link: &RtnlLink, name: &CStr) {
    unsafe { rtnl_link_set_name(link.ptr, name.as_ptr()) };
}

pub fn link_set_link(link: &RtnlLink, index: libc::c_int) {
    unsafe { rtnl_link_set_link(link.ptr, index) };
}

pub fn link_put(link: RtnlLink) {
    unsafe { rtnl_link_put(link.ptr) };
}

pub fn link_set_ns_pid(link: &RtnlLink, pid: libc::pid_t) {
    unsafe { rtnl_link_set_ns_pid(link.ptr, pid) };
}

/**
 * Change link
 * @arg sk              netlink socket.
 * @arg orig            original link to be changed
 * @arg changes         link containing the changes to be made
 * @arg flags           additional netlink message flags
 *
 * Builds a \c RTM_NEWLINK netlink message requesting the change of
 * a network link. If -EOPNOTSUPP is returned by the kernel, the
 * message type will be changed to \c RTM_SETLINK and the message is
 * resent to work around older kernel versions.
 *
 * The link to be changed is looked up based on the interface index
 * supplied in the \p orig link. Optionaly the link name is used but
 * only if no interface index is provided, otherwise providing an
 * link name will result in the link name being changed.
 *
 * If no matching link exists, the function will return
 * -NLE_OBJ_NOTFOUND.
 *
 * After sending, the function will wait for the ACK or an eventual
 * error message to be received and will therefore block until the
 * operation has been completed.
 *
 * @copydoc auto_ack_warning
 *
 * @note The link name can only be changed if the link has been put
 *       in opertional down state. (~IF_UP)
 *
 * @return 0 on success or a negative error code.
 */
pub fn link_change(
    sk: &socket::NetlinkSocket,
    orig: &RtnlLink,
    changes: &RtnlLink,
    flags: libc::c_int,
) -> bool {
    if unsafe { rtnl_link_change(sk.ptr, orig.ptr, changes.ptr, flags) } < 0 {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_globals() {
        assert_eq!(global_rtm_setlink(), 19);
        assert_eq!(global_nlm_f_create(), 0x400);
    }
}
