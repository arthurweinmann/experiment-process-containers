use std::env;

use jail::init_package;

pub fn init_miscellaneous() -> (String, String, String, i64, u32, u32) {
    let args: Vec<String> = env::args().collect();

    let socket_path_incoming = format!("{}/t_0_{}.sock", &args[2], &args[1]);
    let socket_path_outgoing = format!("{}/t_1_{}.sock", &args[2], &args[1]);

    let local_cloud_provider = args[3].clone();

    let non_root_uid: u32 = (&args[4]).parse().unwrap();

    let non_root_gid: u32 = (&args[5]).parse().unwrap();

    init_package(non_root_uid, non_root_gid);

    (
        local_cloud_provider,
        socket_path_incoming,
        socket_path_outgoing,
        unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) },
        non_root_uid,
        non_root_gid,
    )
}
