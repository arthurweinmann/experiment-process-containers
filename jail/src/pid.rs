use super::config::JailConf;
use super::error::Result;
use super::protobuf::parse_pooled_wake_up;
use super::utils::{read_from_fd_ignore_err, read_message_from_fd, write_to_fd};

//// If the "init" process of a PID namespace terminates, the kernel
/// terminates all of the processes in the namespace via a SIGKILL
/// signal; it is not possible to create a new process in a PID
/// namespace whose "init" process has terminated.
pub fn init_ns(jconf: &mut JailConf) -> Result<()> {
    if !jconf.clone_newpid || !jconf.create_pooled_thread {
        return Ok(());
    }

    if !write_to_fd(jconf.passed_admin_child_fd, "W".as_bytes()) {
        println!("failed to write to child fd that pid init process will go in waiting mode");
    }

    let mess = match read_message_from_fd(jconf.passed_admin_child_fd, 1024) {
        Ok(mess) => mess,
        Err(e) => {
            if e.is_eof() {
                // parent closed the fd either because it closed itself or because of an error
                println!("parent closed, closing child..");
                std::process::exit(0);
            }

            return Err(e);
        }
    };

    // let now = std::time::Instant::now();

    let (cwd, exec_file, argv, env) = parse_pooled_wake_up(&mess);

    // println!("Pooled toaster received env: {:?}", env);

    jconf.cwd = cwd;
    jconf.exec_file = exec_file;
    jconf.argv = argv;
    jconf.env = env;

    // let now2 = std::time::Instant::now();
    // println!("parse_pooled_wake_up took: {:?}", now2.duration_since(now));

    Ok(())
}
