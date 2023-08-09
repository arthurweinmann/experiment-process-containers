use libc::{self, c_int};

/// Wrapper to interpret syscall exit codes and provide a rustacean `io::Result`
pub struct SyscallReturnCode(pub c_int);

impl SyscallReturnCode {
    /// Returns the last OS error if value is -1 or Ok(value) otherwise.
    /// see http://man7.org/linux/man-pages/man3/errno.3.html
    pub fn into_result(self) -> std::io::Result<c_int> {
        if self.0 == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(self.0)
        }
    }

    /// Returns the last OS error if value is -1 or Ok(()) otherwise.
    pub fn into_empty_result(self) -> std::io::Result<()> {
        self.into_result().map(|_| ())
    }
}

/* Implementation of nsjail C++ macro TEMP_FAILURE_RETRY
    # define TEMP_FAILURE_RETRY(expression) \
        (__extension__							      \
            ({ long int __result;						      \
            do __result = (long int) (expression);				      \
            while (__result == -1L && errno == EINTR);			      \
            __result; }))
*/

/* Evaluate EXPRESSION, and repeat as long as it returns -1 with `errno'
set to EINTR.  */

// test if this macro works
// test if after reader errno manually it is stil the same in std::io::Error::last_os_error()
#[macro_export]
macro_rules! syscall_temp_failure_retry {
    ( $x:expr ) => {
        loop {
            use sys_util::errno;
            let res = $x;
            if res != -1 || errno::Errno::last() != errno::Errno::EINTR {
                // errno::Errno::last().desc() != "Interrupted system call"
                if res == -1 {
                    break Err(std::io::Error::last_os_error());
                }
                break Ok(res);
            }
        }
    };
}

#[macro_export]
macro_rules! syscall_temp_failure_retry_raw {
    ( $x:expr ) => {
        loop {
            use sys_util::errno;
            let res = $x;
            if res != -1 || errno::Errno::last() != errno::Errno::EINTR {
                // errno::Errno::last().desc() != "Interrupted system call"
                break res;
            }
        }
    };
}