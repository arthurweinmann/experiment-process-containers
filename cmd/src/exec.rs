use std::error;
use std::fmt;
use std::io::{self, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

pub fn shell_cmd_stdout(cmd: &str) -> String {
    let res = Command::new("/bin/sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute process")
        .stdout;
    String::from(from_utf8(res.as_slice()).unwrap())
}

pub fn bash_cmd_stdout(cmd: &str) -> String {
    let res = Command::new("/bin/bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute process")
        .stdout;
    String::from(from_utf8(res.as_slice()).unwrap())
}

pub fn shell_cmd_stderr(cmd: &str) -> String {
    let res = Command::new("/bin/sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute process")
        .stderr;
    String::from(from_utf8(res.as_slice()).unwrap())
}

pub struct BashCommand {
    cmd: Command,
}

// See https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/wrap_error.html

#[derive(Debug)]
pub struct StdError {
    stderr: String,
    code: i32,
    is_signal: bool,
}

impl fmt::Display for StdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Stderr: {}\nCode: {} - is_signal: {}\n",
            self.stderr, self.code, self.is_signal
        )
    }
}

impl error::Error for StdError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug)]
pub enum CommandError {
    Empty,
    ParseIO(io::Error),
    ParseStdErr(StdError),
}

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CommandError::Empty => Ok(()),
            CommandError::ParseIO(ref e) => e.fmt(f),
            CommandError::ParseStdErr(ref e) => e.fmt(f),
        }
    }
}

// This is important for other errors to wrap this one.
impl error::Error for CommandError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            CommandError::Empty => None,
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            CommandError::ParseIO(ref e) => Some(e),
            CommandError::ParseStdErr(ref e) => Some(e),
        }
    }
}

impl From<io::Error> for CommandError {
    fn from(err: io::Error) -> CommandError {
        CommandError::ParseIO(err)
    }
}

impl From<StdError> for CommandError {
    fn from(err: StdError) -> CommandError {
        CommandError::ParseStdErr(err)
    }
}

impl BashCommand {
    pub fn new_sh(cmd: &str) -> Self {
        let mut bcmd = BashCommand {
            cmd: Command::new("sh"),
        };
        bcmd.cmd.arg("-c").arg(cmd);
        bcmd
    }

    pub fn new_bash(cmd: &str) -> Self {
        let mut bcmd = BashCommand {
            cmd: Command::new("bash"),
        };
        bcmd.cmd.arg("-c").arg(cmd);
        bcmd
    }

    pub fn cd<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.cmd.current_dir(path);
        self
    }

    pub fn new_empty() -> Self {
        let mut bcmd = BashCommand {
            cmd: Command::new("sh"),
        };
        bcmd.cmd.arg("-c");
        bcmd
    }

    pub fn with_cmd(mut self, cmd: &str) -> Self {
        self.cmd.arg(cmd);
        self
    }

    pub fn run_standard_output(&mut self) {
        let output = match self.cmd.output() {
            Ok(v) => v,
            Err(e) => {
                io::stderr().write_all(format!("{}", e).as_bytes()).unwrap();
                return;
            }
        };

        println!("status: {}", output.status);
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    }

    pub fn run_utf8(&mut self) -> Result<String, CommandError> {
        let output = self.cmd.output()?;

        if !output.status.success() {
            return Err(StdError {
                stderr: String::from(from_utf8(output.stderr.as_slice()).unwrap()),
                code: output
                    .status
                    .code()
                    .ok_or_else(|| {
                        output.status.signal().unwrap_or(0) // on Unix, if it was a signal, it is not in code() which returns None
                    })
                    .unwrap(),
                is_signal: false,
            }
            .into());
        }

        Ok(String::from(from_utf8(output.stdout.as_slice()).unwrap()))
    }
}
