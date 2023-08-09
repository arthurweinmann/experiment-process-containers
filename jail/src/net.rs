use super::config::JailConf;
use super::error::Result;
use std::ffi::CStr;
use sys_util::errno::Errno;
use sys_util::libnl::{link, socket};
use sys_util::sched::setns;

const IFACE_NAME_UNDERSCORE: &[u8] = "vs_".as_bytes();
const IFACE_NAME: &[u8] = "vs\0".as_bytes();
const LO: &[u8] = "lo\0".as_bytes();

pub fn clone_iface<'a>(
    iface_name: &'a CStr,
    iface_vs: &'a CStr,
    iface_vs_ma: &'a CStr,
    sk: &socket::NetlinkSocket,
    link_cache: &link::NetlinkCache,
    pid: libc::pid_t,
) -> bool {
    let rmv = match link::link_macvlan_alloc() {
        None => return false,
        Some(link) => link,
    };

    let master_index = match link::link_name2i(link_cache, iface_vs) {
        None => {
            link::link_put(rmv);
            return false;
        }
        Some(index) => index,
    };

    link::link_set_name(&rmv, iface_name);
    link::link_set_link(&rmv, master_index);
    link::link_set_ns_pid(&rmv, pid);

    if !iface_vs_ma.to_bytes().is_empty() {
        // iface_vs_ma, which is --macvlan_vs_ma in nsjail cmdline, MAC-address of the 'vs' interface (e.g. "ba:ad:ba:be:45:00"),
        // is empty for now when executing a Toaster. So,
        // Todo: complete

        /* nsjail
            struct nl_addr* nladdr = nullptr;
            if ((err = nl_addr_parse(nsjconf->iface_vs_ma.c_str(), AF_LLC, &nladdr)) < 0) {
                LOG_E("nl_addr_parse('%s', AF_LLC) failed: %s",
                    nsjconf->iface_vs_ma.c_str(), nl_geterror(err));
                return false;
            }
            rtnl_link_set_addr(rmv, nladdr);
            nl_addr_put(nladdr);
        */

        panic!("net->clone_iface->!iface_vs_ma.is_empty(): not implemented");
    }

    if !link::link_add(sk, &rmv, link::global_nlm_f_create()) {
        link::link_put(rmv);
        return false;
    }

    link::link_put(rmv);

    true
}

// only used to move existing network interface to the new NET namespace. See comment above in init_ns_from_parent.
pub fn move_to_ns(
    iface: &CStr,
    sk: &socket::NetlinkSocket,
    link_cache: &link::NetlinkCache,
    pid: libc::pid_t,
) -> bool {
    let orig_link = match link::link_get_by_name(link_cache, iface) {
        None => return false,
        Some(link) => link,
    };

    let new_link = match link::link_alloc() {
        None => {
            link::link_put(orig_link);
            return false;
        }
        Some(link) => link,
    };

    link::link_set_ns_pid(&new_link, pid);

    if !link::link_change(sk, &orig_link, &new_link, link::global_rtm_setlink()) {
        // link::global_rtm_setlink() returns libnl global constants RTM_SETLINK defined in rtnetlink.h
        link::link_put(new_link);
        link::link_put(orig_link);
        return false;
    }

    link::link_put(new_link);
    link::link_put(orig_link);

    true
}

// we must first init_ns_from_parent before child init_ns_from_child, if init_ns_from_child is first
// it will fail since lo will be in state down in the absence of additional “virtual” network interfaces (tveth1)
// see https://www.toptal.com/linux/separation-anxiety-isolating-your-system-with-linux-namespaces
pub fn init_ns_from_parent(jconf: &JailConf, pid: libc::pid_t) -> Result<()> {
    if !jconf.clone_newnet {
        return Ok(());
    }

    let mut sk = match socket::alloc() {
        // libnl->nl_socket_alloc
        None => {
            return Err((
                "Could not allocate socket with nl_socket_alloc()",
                Errno::last(),
            )
                .into())
        }
        Some(sock) => sock,
    };

    let res_sock_connect = socket::connect(&mut sk, libc::NETLINK_ROUTE);
    if res_sock_connect < 0 {
        // libnl->nl_connect(sk, NETLINK_ROUTE)
        socket::free(sk); // libnl->nl_socket_free
        return Err(format!(
            "Unable to connect socket: {}",
            socket::get_error(res_sock_connect)
        )
        .into());
    }

    let link_cache = match link::alloc_cache(&mut sk, libc::AF_UNSPEC) {
        // rtnl_link_alloc_cache
        None => {
            socket::free(sk); // libnl->nl_socket_free
            return Err(format!(
                "rtnl_link_alloc_cache(): {}",
                socket::get_error(res_sock_connect)
            )
            .into());
        }
        Some(cache) => cache,
    };

    // ifaces is populated in nsjail with existing network interface that you want to move into the new NET namespace. In toaster exe, for now
    // we do not use it, but it may be useful for virtio and tun/tap
    for iface in jconf.ifaces.iter() {
        if !move_to_ns(iface, &sk, &link_cache, pid) {
            link::free_cache(link_cache); // libnl->nl_cache_free
            socket::free(sk); // libnl->nl_socket_free
            return Err(format!("Could not move {:?} to NS", iface).into());
        }
    }

    if let Some(ref multi_net) = jconf.multi_net {
        for i in 0..multi_net.iface_vs.len() {
            if i > 255 {
                panic!("no more than 255 network interfaces are supported");
            }

            let iface_name = [
                IFACE_NAME_UNDERSCORE,
                i.to_string().as_bytes(),
                "\0".as_bytes(),
            ]
            .concat();

            if !clone_iface(
                unsafe { CStr::from_bytes_with_nul_unchecked(&iface_name) },
                &multi_net.iface_vs[i],
                jconf.iface_vs_ma,
                &sk,
                &link_cache,
                pid,
            ) {
                link::free_cache(link_cache); // libnl->nl_cache_free
                socket::free(sk); // libnl->nl_socket_free
                return Err(format!(
                    "Could not clone one of many iface from JailConf: {}",
                    Errno::last()
                )
                .into());
            }
        }
    } else {
        if !jconf.iface_vs.to_bytes().is_empty()
            && !clone_iface(
                unsafe { CStr::from_bytes_with_nul_unchecked(IFACE_NAME) },
                jconf.iface_vs,
                jconf.iface_vs_ma,
                &sk,
                &link_cache,
                pid,
            )
        {
            link::free_cache(link_cache); // libnl->nl_cache_free
            socket::free(sk); // libnl->nl_socket_free
            return Err(format!(
                "Could not clone single iface from JailConf: {}",
                Errno::last()
            )
            .into());
        }
    }

    link::free_cache(link_cache); // libnl->nl_cache_free
    socket::free(sk); // libnl->nl_socket_free

    Ok(())
}

pub fn iface_config(iface: &CStr, ip: &CStr, mask: &CStr, gw: &CStr) -> Result<()> {
    let b = cpp_bindings::iface_config(iface, ip, mask, gw); // todo: port to native rust
    if !b {
        return Err(Errno::last().into());
    }

    Ok(())
}

// we must first init_ns_from_parent before child init_ns_from_child, if init_ns_from_child is first
// it will fail since lo will be in state down in the absence of additional “virtual” network interfaces (tveth1)
// see https://www.toptal.com/linux/separation-anxiety-isolating-your-system-with-linux-namespaces
pub fn init_ns_from_child(jconf: &JailConf) -> Result<()> {
    if !jconf.clone_newnet {
        return Ok(());
    }
    
    if jconf.iface_lo && !cpp_bindings::iface_up(unsafe { CStr::from_bytes_with_nul_unchecked(LO) })
    {
        return Err("lo is not upt".into());
    }

    if let Some(ref multi_net) = jconf.multi_net {
        for i in 0..multi_net.iface_vs.len() {
            let iface_name = [
                IFACE_NAME_UNDERSCORE,
                i.to_string().as_bytes(),
                "\0".as_bytes(),
            ]
            .concat();

            iface_config(
                unsafe { CStr::from_bytes_with_nul_unchecked(&iface_name) },
                &multi_net.iface_vs_ip[i],
                &multi_net.iface_vs_nm[i],
                &multi_net.iface_vs_gw[i],
            )
            .map_err(|e| {
                format!(
                    "iface_config failed for {:?} - {:?} - {:?}: {:?}",
                    multi_net.iface_vs_ip[i], multi_net.iface_vs_nm[i], multi_net.iface_vs_gw[i], e
                )
            })?;
        }
    } else {
        if !jconf.iface_vs.to_bytes().is_empty() {
            iface_config(
                unsafe { CStr::from_bytes_with_nul_unchecked(IFACE_NAME) },
                &jconf.iface_vs_ip,
                jconf.iface_vs_nm,
                &jconf.iface_vs_gw,
            )
            .map_err(|e| {
                format!(
                    "iface_config failed for {:?} - {:?} - {:?}: {:?}",
                    jconf.iface_vs_ip, jconf.iface_vs_nm, jconf.iface_vs_gw, e
                )
            })?;
        }
    }

    Ok(())
}

pub fn join_ns(fd: i32) -> Result<()> {
    setns(fd, libc::CLONE_NEWNET)?;
    Ok(())
}
