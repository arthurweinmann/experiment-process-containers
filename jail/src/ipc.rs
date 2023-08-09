use super::config::JailConf;
use super::error::Result;
use sys_util::sched::setns;

pub fn join_ns(fd: i32) -> Result<()> {
    setns(fd, libc::CLONE_NEWIPC)?;
    Ok(())
}