use std::{error, fmt, io, result};
use sys_util::errno::Errno;
use cmd::exec::CommandError;
use std::env;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Empty,
    ParseIO(io::Error),
    ParseStr(&'static str),
    ParseString(String),
    ParseErrno(&'static str, Errno),
    ParseErrnoAlone(Errno),
    ParseCMD(CommandError),
    ParseVarErr(env::VarError),
    EOF,
    ParsePid((libc::pid_t, Option<libc::pid_t>, String))
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Empty => Ok(()),
            Error::ParseIO(ref e) => e.fmt(f),
            Error::ParseStr(ref e) => e.fmt(f),
            Error::ParseString(ref e) => e.fmt(f),
            Error::ParseErrno(ref s, ref e) => write!(f, "Message: {} < || > Errno: {}", s, e),
            Error::ParseErrnoAlone(ref e) => e.fmt(f),
            Error::ParseCMD(ref e) => e.fmt(f),
            Error::ParseVarErr(ref e) => e.fmt(f),
            Error::EOF => write!(f, "EOF"),
            Error::ParsePid(ref tuple) => write!(f, "Child_PID: {:?}, Child_PIDFD: {:?}, Error: {}", tuple.0, tuple.1, tuple.2),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Empty => None,
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            Error::ParseIO(ref e) => Some(e),
            Error::ParseStr(_) => None,
            Error::ParseString(_) => None,
            Error::ParseErrno(_, ref e) => Some(e),
            Error::ParseErrnoAlone(ref e) => Some(e),
            Error::ParseCMD(ref e) => Some(e),
            Error::ParseVarErr(ref e) => Some(e),
            Error::EOF => None,
            Error::ParsePid(_) => None,
        }
    }
}

impl Error {
    pub fn is_eof(&self) -> bool {
        match *self {
            Error::EOF => true,
            _ => false,
        }
    }
}

impl From<CommandError> for Error {
    fn from(err: CommandError) -> Error {
        Error::ParseCMD(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::ParseIO(err)
    }
}

impl From<env::VarError> for Error {
    fn from(err: env::VarError) -> Error {
        Error::ParseVarErr(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::ParseString(err)
    }
}

impl From<&'static str> for Error {
    fn from(err: &'static str) -> Error {
        Error::ParseStr(err)
    }
}

impl From<(&'static str, Errno)> for Error {
    fn from(err: (&'static str, Errno)) -> Error {
        Error::ParseErrno(err.0, err.1)
    }
}

impl From<Errno> for Error {
    fn from(err: Errno) -> Error {
        Error::ParseErrnoAlone(err)
    }
}
