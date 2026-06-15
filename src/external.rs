use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::os::unix::process::CommandExt;

use crate::open_file_redirects;
use crate::parser::{Redirect, ShellCommand};

pub fn execute_command(exec_path: PathBuf, command: &ShellCommand) {
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
