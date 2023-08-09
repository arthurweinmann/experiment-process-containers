use super::config::JailConf;
use super::error::Result;
use sys_util::uts;
use sys_util::sched::setns;

pub fn init_ns(jconf: &JailConf) -> Result<()> {
    uts::sethostname(jconf.hostname)?; // shoudn't we use a null terminated string ?
    Ok(())
}

pub fn join_ns(fd: i32) -> Result<()> {
    setns(fd, libc::CLONE_NEWUTS).map_err(|e| format!("could not join uts namespace: {}", e))?;
    Ok(())
}