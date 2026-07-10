use std::io;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use which::which;

use crate::open_file_redirects;
use crate::parser::{Redirect, ShellCommand};

pub fn execute_ext_command(exec_path: PathBuf, command: &ShellCommand) {
    let mut extern_cmd = Command::new(&exec_path);
    extern_cmd.arg0(&command.name);
    extern_cmd.args(&command.args);

    if let Some(redirect) = &command.redirect {
        let file = match open_file_redirects(redirect) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };

        match redirect {
            Redirect::Stdout(_) | Redirect::AppendStdout(_) => {
                extern_cmd.stdout(Stdio::from(file));
            }
            Redirect::Stderr(_) | Redirect::AppendStderr(_) => {
                extern_cmd.stderr(Stdio::from(file));
            }
        }
    }

    match extern_cmd.status() {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to execute command: {e}"),
    }
}

pub fn command_path(cmd: &str) -> Option<PathBuf> {
    which(cmd).ok()
}

pub fn ext_command_output(exec_path: PathBuf, args: &[&str]) -> io::Result<Output> {
    let mut extern_cmd = Command::new(&exec_path);
    extern_cmd.args(args);
    extern_cmd.output()
}
