pub mod caps;
pub mod cgroupv1;
pub mod cgroupv2;
pub mod config;
pub mod contain;
pub mod cpu;
pub mod error;
pub mod ipc;
pub mod mnt;
pub mod net;
pub mod pid;
pub mod protobuf;
pub mod rlimit;
pub mod sandbox;
pub mod subproc;
pub mod user;
pub mod utils;
pub mod uts;
pub mod wait;

/// init_package must be called only once and not concurrently at beginning of time before any execution
pub fn init_package(non_root_owner: libc::uid_t, non_root_group: libc::gid_t) {
    unsafe { config::init_statics() };

    mnt::init_mount_folder(non_root_owner, non_root_group);
}
