use std::io;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use which::which;

use crate::command::{Redirect, ShellCommand};
use crate::open_file_redirects;

pub fn execute_ext_command_foreground(exec_path: PathBuf, command: &ShellCommand) {
    let mut extern_cmd = build_command(exec_path, command);
    match extern_cmd.status() {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to execute command: {e}"),
    }
}

pub fn build_command(exec_path: PathBuf, command: &ShellCommand) -> Command {
    let mut extern_cmd = Command::new(&exec_path);
    extern_cmd.arg0(&command.name);
    extern_cmd.args(&command.args);

    if let Some(redirect) = &command.redirect {
        let file = match open_file_redirects(redirect) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{e}");
                return extern_cmd;
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

    extern_cmd
}
pub fn command_path(cmd: &str) -> Option<PathBuf> {
    which(cmd).ok()
}

pub fn ext_command_output(
    exec_path: PathBuf,
    args: &[&str],
    envs: &[(&str, &str)],
) -> io::Result<Output> {
    let mut extern_cmd = Command::new(&exec_path);
    extern_cmd.args(args);
    extern_cmd.envs(envs.iter().map(|&p| p));
    extern_cmd.output()
}
