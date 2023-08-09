use std::env;
use std::ffi::{CStr, CString};

use jail::config::{IDMapT, JailConf, MountT, MultiNetConfig, NSSIGS, PIDT};
use jail::init_package;
use jail::subproc::{
    child, run_monitor_child, subproc_new_proc, subproc_new_proc_exec, subproc_new_proc_setup,
};

use jail::contain::{self, setup_fd};
use jail::error::{Error, Result};
use jail::protobuf::create_pooled_wake_up_mess;
use jail::utils::{
    read_from_fd_ignore_err, to_exec_array, to_exec_array_cstring, write_message_to_fd, write_to_fd,
};
use jail::{cgroupv1, cgroupv2, net, sandbox, user};

use sys_util::execv;

use seccomp::*;

use std::convert::TryInto;

fn main() {
    init_package(1000, 1000);

    let args: Vec<String> = env::args().collect();

    let mut command = CString::new("/bin/sh").unwrap();
    let mut arguments = vec![
        CString::new("/bin/sh").unwrap(),
        CString::new("-i").unwrap(),
    ];
    let mut chroot = "/";
    let mut mount_dev_pts = false;
    let mut mount_readonly: Vec<CString> = vec![];
    let mut cwd = "/";
    let mut selfPath = "";
    let mut do_not_keep_env = false;
    let mut handle_double_virt = false;
    let mut inside_uid = 0;
    let mut inside_gid = 0;
    let mut uid_map = vec![0, 0, 1000, 1000];
    let mut gid_map = vec![0, 0, 1000, 1000];
    let mut multi_net = MultiNetConfig {
        iface_vs: vec![],
        iface_vs_ip: vec![],
        iface_vs_nm: vec![],
        iface_vs_gw: vec![],
        iface_vs_ma: vec![],
    };
    let mut env: Vec<CString> = vec![];

    if args.len() < 3 {
        panic!("not enough arguments")
    }

    if (args.len() - 1) % 2 != 0 {
        panic!("odd number of arguments: {}", args.len());
    }

    for i in (1..args.len()).step_by(2) {
        match args[i].as_str() {
            "--mount_readonly" => {
                // src:dst
                let spl: Vec<&str> = args[i + 1].split(":").collect();
                if spl.len() != 2 {
                    panic!(
                        "invalid --mount_readonly argument: {} {}",
                        args[i + 1],
                        spl.len()
                    );
                }
                mount_readonly.push(CString::new(spl[0]).unwrap());
                mount_readonly.push(CString::new(spl[1]).unwrap());
            }
            "--mount_dev_pts" => mount_dev_pts = true,
            "--command" => {
                command = CString::new(args[i + 1].clone()).unwrap();
                arguments = vec![];
            }
            "--commandArg" => {
                arguments.push(CString::new(args[i + 1].clone()).unwrap());
            }
            "--chroot" => {
                chroot = &args[i + 1];
            }
            "--inside_uid" => {
                inside_uid = args[i + 1].parse().unwrap();
            }
            "--inside_gid" => {
                inside_gid = args[i + 1].parse().unwrap();
            }
            "--uid_map" => {
                uid_map = vec![];
                for pair in args[i + 1].split(",").into_iter() {
                    let spl: Vec<&str> = pair.split(":").collect();
                    uid_map.push(spl[0].parse::<u32>().unwrap());
                    uid_map.push(spl[1].parse::<u32>().unwrap());
                }
            }
            "--gid_map" => {
                gid_map = vec![];
                for pair in args[i + 1].split(",").into_iter() {
                    let spl: Vec<&str> = pair.split(":").collect();
                    gid_map.push(spl[0].parse::<u32>().unwrap());
                    gid_map.push(spl[1].parse::<u32>().unwrap());
                }
            }
            "--cwd" => {
                cwd = &args[i + 1];
            }
            "--iface_vs" => {
                multi_net
                    .iface_vs
                    .push(CString::new(args[i + 1].to_owned()).unwrap());
            }
            "--iface_vs_ip" => {
                multi_net
                    .iface_vs_ip
                    .push(CString::new(args[i + 1].to_owned()).unwrap());
            }
            "--iface_vs_nm" => {
                multi_net
                    .iface_vs_nm
                    .push(CString::new(args[i + 1].to_owned()).unwrap());
            }
            "--iface_vs_gw" => {
                multi_net
                    .iface_vs_gw
                    .push(CString::new(args[i + 1].to_owned()).unwrap());
            }
            "--env" => {
                env.push(CString::new(CString::new(args[i + 1].to_owned()).unwrap()).unwrap());
            }
            "--do_not_keep_env" => {
                do_not_keep_env = true;
            }
            "--self_path" => {
                selfPath = &args[i + 1];
            }
            "--handle_double_virt" => {
                handle_double_virt = true;
            }
            _ => {
                panic!("invalid argument {}", args[i]);
            }
        };
    }

    // println!("Chroot: {},CWD: {}, Multinet: {:?}", chroot, cwd, multi_net);

    // Will not be used since multi_net will be defined
    let iface_vs = CStr::from_bytes_with_nul("\0".as_bytes()).unwrap();
    let iface_vs_ip = CString::new("0.0.0.0").unwrap();
    let iface_vs_nm = CStr::from_bytes_with_nul("255.255.255.0\0".as_bytes()).unwrap();
    let iface_vs_gw = CStr::from_bytes_with_nul("0.0.0.0\0".as_bytes()).unwrap();
    let iface_vs_ma = CStr::from_bytes_with_nul("\0".as_bytes()).unwrap();

    let mut jconf = JailConf::new_from_root(
        CString::new(chroot).unwrap(),
        iface_vs,
        iface_vs_ip,
        iface_vs_nm,
        iface_vs_gw,
        iface_vs_ma,
    );
    jconf.debug = true;

    let filter = SeccompFilter::new(vec![].into_iter().collect(), SeccompAction::Allow)
        .unwrap()
        .try_into()
        .unwrap();

    for i in (0..uid_map.len()).step_by(2) {
        jconf.uids.push(IDMapT {
            inside_id: uid_map[i],
            outside_id: uid_map[i + 1],
            count: 1,
            is_newidmap: false,
        });
    }

    for i in (0..gid_map.len()).step_by(2) {
        jconf.gids.push(IDMapT {
            inside_id: gid_map[i],
            outside_id: gid_map[i + 1],
            count: 1,
            is_newidmap: false,
        });
    }

    jconf.user_inside_uid = inside_uid;
    jconf.user_inside_gid = inside_gid;

    jconf.clone_newipc = true;
    jconf.clone_newnet = true;
    jconf.clone_newns = true;
    jconf.clone_newcgroup = true;
    jconf.clone_newpid = true;

    jconf.seccomp_filter = Some(filter);

    jconf.clone_newuser = false;

    jconf.prepare_env_in_child = true;

    jconf.keep_caps = true;
    jconf.keep_env = !do_not_keep_env;
    jconf.skip_setsid = true;
    jconf.disable_no_new_privs = true;

    jconf.mnt_ms_slave = false;

    jconf.clone_newuts = true;
    jconf.hostname = "toastate";

    jconf.is_root_rw = true;
    jconf.is_proc_rw = true;

    // set all soft limits to the hard one
    jconf.disable_rl = false;
    jconf.rl_as = jail::rlimit::get_rlimit64(libc::RLIMIT_AS)
        .unwrap()
        .rlim_max;
    jconf.rl_core = jail::rlimit::get_rlimit64(libc::RLIMIT_CORE)
        .unwrap()
        .rlim_max;
    jconf.rl_cpu = jail::rlimit::get_rlimit64(libc::RLIMIT_CPU)
        .unwrap()
        .rlim_max;
    jconf.rl_fsize = jail::rlimit::get_rlimit64(libc::RLIMIT_FSIZE)
        .unwrap()
        .rlim_max;
    jconf.rl_nofile = jail::rlimit::get_rlimit64(libc::RLIMIT_NOFILE)
        .unwrap()
        .rlim_max;
    jconf.rl_nproc = jail::rlimit::get_rlimit64(libc::RLIMIT_NPROC)
        .unwrap()
        .rlim_max;
    jconf.rl_stack = jail::rlimit::get_rlimit64(libc::RLIMIT_STACK)
        .unwrap()
        .rlim_max;

    jconf.cwd = cwd.to_owned();

    jconf.num_cpus = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };

    jconf.iface_lo = true;
    jconf.multi_net = Some(multi_net);

    jconf.with_default_mounts();

    if mount_dev_pts {
        jconf.mountpts.push(MountT {
            src: None,
            dst: MountT::transform_dst(&jconf, "dev/pts"), // empty string means /, see mnt.rs
            dst_in_pivot: CString::new("/dev/pts").unwrap(),
            fs_type: Some(CString::new("devpts").unwrap()),
            options: Some(CString::new("gid=4,mode=620").unwrap()),
            flags: 0,
            is_dir: true,
            is_mandatory: true,
            is_symlink: false,
            mounted: false,
        });
    }

    if mount_readonly.len() > 0 {
        for i in (0..mount_readonly.len()).step_by(2) {
            let src = mount_readonly[i].clone();
            let mut dst = mount_readonly[i + 1].as_bytes();

            if dst[0] == '/' as u8 {
                dst = &dst[1..];
            }

            let dst2 = [b"/", dst].concat();

            jconf.mountpts.push(MountT {
                src: Some(src),
                dst: MountT::transform_dst_bytes(&jconf, dst), // empty string means /, see mnt.rs
                dst_in_pivot: CString::new(dst2).unwrap(),
                fs_type: None,
                options: None,
                flags: libc::MS_BIND | libc::MS_REC | libc::MS_PRIVATE | libc::MS_RDONLY,
                is_dir: true,
                is_mandatory: true,
                is_symlink: false,
                mounted: false,
            });
        }
    }

    jconf.exec_file = Some(command);
    jconf.argv = Some(arguments);
    jconf.env = Some(env);

    if !handle_double_virt {
        std::process::exit(run_monitor_child(&mut jconf, child).unwrap());
    } else {
        std::process::exit(run_monitor_child(&mut jconf, unsecure_double_virt_child).unwrap());
    }
}

extern "C" fn unsecure_double_virt_child(data: *mut libc::c_void) -> libc::c_int {
    let ref mut jconf = *unsafe { Box::from_raw(data as *mut JailConf) };

    // check jconf to see if we need to join existing namespaces

    unsafe { libc::close(jconf.passed_admin_parent_fd) };

    let err = match subproc_new_proc_setup(jconf) {
        Ok(_) => {
            let res = unsafe { libc::unshare(libc::CLONE_NEWNS) };
            if res < 0 {
                panic!("Could not unshare: {}", sys_util::errno::Errno::last());
            }

            let res = unsafe {
                libc::mount(
                    jail::mnt::CONST_C_STR_ROOT.as_ptr() as *const libc::c_char,
                    jail::mnt::CONST_C_STR_ROOT.as_ptr() as *const libc::c_char,
                    std::ptr::null(),
                    libc::MS_REC | libc::MS_SHARED,
                    std::ptr::null() as *const libc::c_void,
                )
            };
            if res < 0 {
                panic!(
                    "Could not make mounts shared: {}",
                    sys_util::errno::Errno::last()
                );
            }

            match subproc_new_proc_exec(jconf) {
                Ok(_) => {
                    panic!("should not happen");
                }
                Err(e) => e,
            }
        }
        Err(e) => e,
    };

    if !write_to_fd(jconf.passed_admin_child_fd, "E".as_bytes()) {
        println!("failed to write to child fd that child failed");
    }

    if jconf.debug {
        if !write_to_fd(
            jconf.passed_admin_child_fd,
            format!("err child: {}", err).as_bytes(),
        ) {
            println!("failed to write err to child fd: {}", err);
        }
    }

    unsafe { libc::close(jconf.passed_admin_child_fd) };

    1
}
