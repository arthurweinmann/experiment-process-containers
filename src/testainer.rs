use std::env;
use std::ffi::{CStr, CString};
use std::path::PathBuf;
use std::time::Instant;

use cmd::exec::bash_cmd_stdout;
use disk::btrfs::{new_subvolume_cstr, snapshot};
use disk::overlay_fs::OverlayDir;
use jail::subproc;
use scheduler::config;
use sys_util::errno::Errno;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args[1].as_str() {
        "arbitrary_test" => arbitrary_test(
            &args[2],
            &args[3],
            args[4].parse::<u32>().unwrap(),
            args[5].parse::<u32>().unwrap(),
        ),
        "pool_test" => pool_test(
            &args[2],
            &args[3],
            args[4].parse::<u32>().unwrap(),
            args[5].parse::<u32>().unwrap(),
            args[6].parse::<u32>().unwrap(),
        ),
        _ => {
            panic!("invalid test case")
        }
    }
}

fn pool_test(
    btrfs_file_system: &str,
    overlayfs_mount_point: &str,
    non_root_uid: u32,
    non_root_gid: u32,
    gateway: u32,
) {
    let max_opened_pipes = cmd::exec::bash_cmd_stdout("cat /proc/sys/fs/pipe-user-pages-soft")
        .trim()
        .parse::<u64>()
        .unwrap()
        / 16;

    println!("Max Opened Pipes: {}", max_opened_pipes);

    let rl_nofile = jail::rlimit::get_rlimit64(libc::RLIMIT_NOFILE).unwrap();
    println!(
        "RLIMIT_NOFILE: cur: {}, max: {}, libc::RLIM_INFINITY: {}",
        rl_nofile.rlim_cur,
        rl_nofile.rlim_max,
        libc::RLIM_INFINITY
    );

    println!("non root: {} {}", non_root_uid, non_root_gid);

    let btrfs_file_system_cstring = CString::new(btrfs_file_system).unwrap();

    jail::init_package(non_root_uid, non_root_gid);

    let subvolume_name1 = [btrfs_file_system.as_bytes(), b"/", b"images/1"].concat();

    let num_cpus = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };

    // println!("num_cpus: {}", num_cpus);

    // let command = CString::new("/bin/bash").unwrap();
    // let arg1 = CString::new("-c").unwrap();
    // let arg2 = CString::new("ls -alh /").unwrap();

    let command = CString::new("/main").unwrap();
    let arg1 = CString::new("/main").unwrap();

    let env1 =
        CString::new("PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin").unwrap();
    let env2 = CString::new("TERM=xterm-color").unwrap();
    let env3 = CString::new("HOME=/home/ubuntu").unwrap();

    let gw = CString::new(uint_ip_to_string(gateway)).unwrap();

    let efd_epoll =
        sys_util::epoll::epoll_create1(sys_util::epoll::EpollCreateFlags::EPOLL_CLOEXEC)
            .expect("could not create epoll");
    let mut events = [sys_util::epoll::EpollEvent::empty(); 64];

    for i in 0..10 {
        let start = Instant::now();

        let uid0 = CString::new(i.to_string()).unwrap();
        let subvolume_name0 = [btrfs_file_system_cstring.to_bytes(), b"/", i.to_string().as_bytes()].concat();
        let subvolume_name0 = unsafe { CString::from_vec_unchecked(subvolume_name0) };
        new_subvolume_cstr(&btrfs_file_system_cstring, &uid0, &subvolume_name0, 0).unwrap();
        let mut ovdir = OverlayDir::new(
            btrfs_file_system_cstring.clone(),
            overlayfs_mount_point.as_bytes(),
            uid0.clone(),
            non_root_uid,
            non_root_gid,
            true,
        )
        .expect("could not create overlaydir");

        let ip = CString::new(uint_ip_to_string(gateway + i)).unwrap();

        // println!("ip: {:?}; gw: {:?}", &ip, &gw);

        let mut jconf = config::create_toaster_pool_jconf(
            "local",
            ovdir.mount_point.clone(),
            num_cpus,
            ip,
            &gw,
            String::from("/"),
            true,
            false,
        );

        // ----------------------------------------------------------------------------------------
        // Pool

        subproc::run_child_pidfd(&mut jconf, subproc::child)
            .expect("could not create pooled thread");

        // ----------------------------------------------------------------------------------------

        config::set_jconf_as_join(&mut jconf);

        let end = Instant::now();
        println!("pool creation took: {:?}", end.duration_since(start));

        let start = Instant::now();

        ovdir
            .mount(&subvolume_name1)
            .expect("could not mount overlay ovdir");

        // ----------------------------------------------------------------------------------------
        // Execution

        jconf.argv = Some(vec![command.clone(), arg1.clone()]);
        jconf.env = Some(vec![env3.clone(), env2.clone(), env1.clone()]);
        jconf.exec_file = Some(command.clone());

        let (child_pid, child_pidfd) = subproc::run_child_pidfd(&mut jconf, subproc::child)
            .expect("subproc::run_child_pidfd:");
        // ----------------------------------------------------------------------------------------

        let end = Instant::now();
        println!("execution took: {:?}", end.duration_since(start));

        scheduler::commands_toaster::epoll_register_running_child(
            child_pid as u64,
            efd_epoll,
            child_pidfd,
        );

        let ready = scheduler::net::poll_fd_events(efd_epoll, &mut events, -1);
        if ready != 1 {
            panic!("ready is {}", ready);
        }
        if events[0].data() != child_pid as u64 {
            panic!("{} != {}",events[0].data(), child_pid);
        }

        let end = Instant::now();
        println!(
            "execution including epoll took: {:?}",
            end.duration_since(start)
        );

        let mut wait_status: i32 = 0;

        let pid = unsafe {
            libc::waitpid(
                child_pid,
                &mut wait_status as *mut libc::c_int,
                libc::WNOHANG,
            )
        };

        let end = Instant::now();
        println!(
            "execution including epoll and wait took: {:?} ; wait_status: {}",
            end.duration_since(start),
            wait_status
        );

        if pid < 0 {
            panic!("Error waiting: {}", Errno::last());
        }

        subproc::clean_after_child(&jconf, child_pid).expect("could not clean_after_child");

        let end = Instant::now();
        println!(
            "execution including epoll and wait and including clean_after_child took: {:?} ; wait_status: {}",
            end.duration_since(start),
            wait_status
        );
    }

    std::thread::sleep(std::time::Duration::from_millis(5000));

    println!(
        "*******************************************\n*******************************************"
    );
    println!("Exe Without Pool test");
    println!("*******************************************");



    for i in 12..22 {
        let start = Instant::now();

        let uid0 = CString::new(i.to_string()).unwrap();
        let subvolume_name0 = [btrfs_file_system_cstring.to_bytes(), b"/", i.to_string().as_bytes()].concat();
        let subvolume_name0 = unsafe { CString::from_vec_unchecked(subvolume_name0) };
        new_subvolume_cstr(&btrfs_file_system_cstring, &uid0, &subvolume_name0, 0).unwrap();
        let mut ovdir = OverlayDir::new(
            btrfs_file_system_cstring.clone(),
            overlayfs_mount_point.as_bytes(),
            uid0.clone(),
            non_root_uid,
            non_root_gid,
            true,
        )
        .expect("could not create overlaydir");

        let ip = CString::new(uint_ip_to_string(gateway + i)).unwrap();

        // println!("ip: {:?}; gw: {:?}", &ip, &gw);

        let mut jconf = config::create_toaster_jconf(
            "local",
            ovdir.mount_point.clone(),
            num_cpus,
            ip,
            &gw,
            String::from("/"),
            true,
            false,
        );

        let end = Instant::now();
        println!("subvolume + jconf creation took: {:?}", end.duration_since(start));

        let start = Instant::now();

        ovdir
            .mount(&subvolume_name1)
            .expect("could not mount overlay ovdir");

        // ----------------------------------------------------------------------------------------
        // Execution

        jconf.argv = Some(vec![command.clone(), arg1.clone()]);
        jconf.env = Some(vec![env3.clone(), env2.clone(), env1.clone()]);
        jconf.exec_file = Some(command.clone());

        let (child_pid, child_pidfd) = subproc::run_child_pidfd(&mut jconf, subproc::child)
            .expect("subproc::run_child_pidfd:");
        // ----------------------------------------------------------------------------------------

        let end = Instant::now();
        println!("execution took: {:?}", end.duration_since(start));

        scheduler::commands_toaster::epoll_register_running_child(
            child_pid as u64,
            efd_epoll,
            child_pidfd,
        );

        let ready = scheduler::net::poll_fd_events(efd_epoll, &mut events, -1);
        if ready != 1 {
            panic!("ready is {}", ready);
        }
        if events[0].data() != child_pid as u64 {
            panic!("{} != {}",events[0].data(), child_pid);
        }

        let end = Instant::now();
        println!(
            "execution including epoll took: {:?}",
            end.duration_since(start)
        );

        let mut wait_status: i32 = 0;

        let pid = unsafe {
            libc::waitpid(
                child_pid,
                &mut wait_status as *mut libc::c_int,
                libc::WNOHANG,
            )
        };

        let end = Instant::now();
        println!(
            "execution including epoll and wait took: {:?} ; wait_status: {}",
            end.duration_since(start),
            wait_status
        );

        if pid < 0 {
            panic!("Error waiting: {}", Errno::last());
        }

        subproc::clean_after_child(&jconf, child_pid).expect("could not clean_after_child");

        let end = Instant::now();
        println!(
            "execution including epoll and wait and including clean_after_child took: {:?} ; wait_status: {}",
            end.duration_since(start),
            wait_status
        );
    }



    std::thread::sleep(std::time::Duration::from_millis(5000));

    println!(
        "*******************************************\n*******************************************"
    );
    println!("Pool load test");
    println!("*******************************************");

    let mut pool: Vec<scheduler::pool::Item> = vec![];
    // for i in 0..min(
    //     min(max_opened_pipes / 3, 65536),
    //     min(rl_nofile.rlim_cur / 3, rl_nofile.rlim_max / 3),
    // ) {
    for i in 50..60 {
        println!("creating pool {}", i);

        let uid = (i + 50).to_string();
        let ip = CString::new(uint_ip_to_string(gateway + i as u32)).unwrap();

        let uid0 = CString::new(uid.as_bytes()).unwrap();
        let subvolume_name0 = [btrfs_file_system_cstring.to_bytes(), b"/", i.to_string().as_bytes()].concat();
        let subvolume_name0 = unsafe { CString::from_vec_unchecked(subvolume_name0) };
        new_subvolume_cstr(&btrfs_file_system_cstring, &uid0, &subvolume_name0, 0).unwrap();

        let ovdir = OverlayDir::new(
            btrfs_file_system_cstring.clone(),
            overlayfs_mount_point.as_bytes(),
            uid0.clone(),
            non_root_uid,
            non_root_gid,
            true,
        )
        .expect("could not create overlaydir");

        let mut jconf = config::create_toaster_pool_jconf(
            "local",
            ovdir.mount_point.clone(),
            num_cpus,
            ip,
            &gw,
            String::from("/"),
            true,
            false,
        );

        jconf.argv = Some(vec![command.clone(), arg1.clone()]);
        jconf.env = Some(vec![env3.clone(), env2.clone(), env1.clone()]);
        jconf.exec_file = Some(command.clone());

        subproc::run_child_pidfd(&mut jconf, subproc::child)
            .expect("could not create pooled thread");

        config::set_jconf_as_join(&mut jconf);

        pool.push(scheduler::pool::Item {
            ovdir: ovdir,
            jconf: jconf,
        });

        // std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // std::thread::sleep(std::time::Duration::from_millis(10000000));
}

fn min(a: u64, b: u64) -> u64 {
    if a < b {
        return a;
    }

    b
}

fn uint_ip_to_string(ipuint: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        (ipuint >> 24) as u8,
        (ipuint >> 16) as u8,
        (ipuint >> 8) as u8,
        ipuint as u8
    )
}

fn arbitrary_test(
    btrfs_file_system: &str,
    overlayfs_mount_point: &str,
    non_root_uid: u32,
    non_root_gid: u32,
) {
    let btrfs_file_system = CString::new(btrfs_file_system).unwrap();

    let uid1 = CString::new("1").unwrap();
    let subvolume_name1 = [btrfs_file_system.to_bytes(), b"/", uid1.to_bytes()].concat();
    let subvolume_name1_cstr = unsafe { CString::from_vec_unchecked(subvolume_name1.clone()) };

    let uid2 = CString::new("2").unwrap();
    let subvolume_name2 = [btrfs_file_system.to_bytes(), b"/", uid2.to_bytes()].concat();
    let subvolume_name2_cstr = unsafe { CString::from_vec_unchecked(subvolume_name2.clone()) };

    let uid3 = CString::new("3").unwrap();
    let subvolume_name3 = [btrfs_file_system.to_bytes(), b"/", uid3.to_bytes()].concat();
    let subvolume_name3_cstr = unsafe { CString::from_vec_unchecked(subvolume_name3.clone()) };

    let uid4 = CString::new("4").unwrap();
    let subvolume_name4 = [btrfs_file_system.to_bytes(), b"/", uid4.to_bytes()].concat();
    let subvolume_name4_cstr = unsafe { CString::from_vec_unchecked(subvolume_name4.clone()) };

    let uid0 = CString::new("0").unwrap();
    let subvolume_name0 = [btrfs_file_system.to_bytes(), b"/0"].concat();
    let subvolume_name0 = unsafe { CString::from_vec_unchecked(subvolume_name0) };
    new_subvolume_cstr(&btrfs_file_system, &uid0, &subvolume_name0, 0).unwrap();
    let mut ovdir = OverlayDir::new(
        btrfs_file_system.clone(),
        overlayfs_mount_point.as_bytes(),
        uid0.clone(),
        non_root_uid,
        non_root_gid,
        false,
    )
    .expect("could not create overlaydir");

    new_subvolume_cstr(&btrfs_file_system, &uid1, &subvolume_name1_cstr, 0).unwrap();
    new_subvolume_cstr(&btrfs_file_system, &uid2, &subvolume_name2_cstr, 0).unwrap();
    new_subvolume_cstr(&btrfs_file_system, &uid3, &subvolume_name3_cstr, 0).unwrap();
    new_subvolume_cstr(&btrfs_file_system, &uid4, &subvolume_name3_cstr, 0).unwrap();

    let mut p1 = PathBuf::from(String::from_utf8(subvolume_name1).unwrap());
    let mut p2 = PathBuf::from(String::from_utf8(subvolume_name2).unwrap());
    let mut p3 = PathBuf::from(String::from_utf8(subvolume_name3).unwrap());
    let mut p4 = PathBuf::from(String::from_utf8(subvolume_name4).unwrap());

    // p1.push("sub");
    // std::fs::create_dir(&p1).unwrap();
    // p1.pop();

    // p2.push("sub");
    // std::fs::create_dir(&p2).unwrap();
    // p2.pop();

    // p3.push("sub");
    // std::fs::create_dir(&p3).unwrap();
    // p3.pop();

    // p4.push("sub");
    // std::fs::create_dir(&p4).unwrap();
    // p4.pop();

    {
        // p3.push("sub/bob.txt");
        p3.push("bob.txt");
        std::fs::File::create(&p3).unwrap();
        p3.pop();
        // p3.pop();

        p4.push("many");
        std::fs::create_dir(&p4).unwrap();

        for i in 0..1000 {
            p4.push(format!("{}", i));
            std::fs::create_dir(&p4).unwrap();

            p4.push(format!("{}.txt", i));
            std::fs::File::create(&p4).unwrap();

            p4.pop();
            // p4.pop();
        }

        // p4.push("alice.txt");
        // std::fs::File::create(&p4).unwrap();
        // p4.pop();
        // // p4.pop();

        // p4.push("bob.txt");
        // std::fs::File::create(&p4).unwrap();
        // p4.pop();
        // // p4.pop();
    }

    let lowerdirs = format!("{}:{}", p1.to_str().unwrap(), p2.to_str().unwrap());
    ovdir.mount(lowerdirs.as_bytes()).unwrap();

    let lscmd = format!("ls -alh {}", ovdir.mount_point.to_str().unwrap());
    let lscmd2 = format!("ls -alh {}/sub", ovdir.mount_point.to_str().unwrap());
    let lscmd3 = format!("ls -alh {}/sub", ovdir.workdir.to_str().unwrap());
    // println!("{}", bash_cmd_stdout(&lscmd));
    // println!("{}", bash_cmd_stdout(&lscmd2));
    // println!("{}", bash_cmd_stdout(&lscmd3));

    // p1.push("sub");
    // p2.push("sub");
    // p3.push("sub");
    // p4.push("sub");

    // println!(
    //     "{:?}, {:?} -> {:?}, {:?} -> {:?}",
    //     &ovdir.mount_point, p3, p1, p4, p2
    // );

    let subdir = PathBuf::from("sub");

    let now = Instant::now();
    snapshot(&p3, &p1, &subdir).unwrap();
    let now2 = Instant::now();
    println!("snapshot {:?}", now2.checked_duration_since(now).unwrap());

    let now = Instant::now();
    snapshot(&p4, &p2, &subdir).unwrap();
    let now2 = Instant::now();
    println!("snapshot {:?}", now2.checked_duration_since(now).unwrap());

    // let now = Instant::now();
    // let res = unsafe {
    //     libc::mount(
    //         CString::new(p3.to_str().unwrap().to_owned())
    //             .unwrap()
    //             .as_ptr(),
    //         CString::new(p1.to_str().unwrap().to_owned())
    //             .unwrap()
    //             .as_ptr(),
    //         std::ptr::null(),
    //         libc::MS_BIND,
    //         std::ptr::null() as *const libc::c_void,
    //     )
    // };
    // if res == -1 {
    //     panic!("{}", Errno::last());
    // }
    // let now2 = Instant::now();
    // println!("bind mount {:?}", now2.checked_duration_since(now).unwrap());

    // let now = Instant::now();
    // let res = unsafe {
    //     libc::mount(
    //         CString::new(p4.to_str().unwrap().to_owned())
    //             .unwrap()
    //             .as_ptr(),
    //         CString::new(p2.to_str().unwrap().to_owned())
    //             .unwrap()
    //             .as_ptr(),
    //         std::ptr::null(),
    //         libc::MS_BIND,
    //         std::ptr::null() as *const libc::c_void,
    //     )
    // };
    // if res == -1 {
    //     panic!("{}", Errno::last());
    // }
    // let now2 = Instant::now();
    // println!("bind mount {:?}", now2.checked_duration_since(now).unwrap());

    println!("{}", bash_cmd_stdout(&lscmd));
    println!("{}", bash_cmd_stdout(&lscmd2));
    println!("{}", bash_cmd_stdout(&lscmd3));
}
