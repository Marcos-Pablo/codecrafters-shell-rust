use std::io;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use which::which;

use crate::command::{Redirect, ShellCommand};
use crate::open_file_redirects;
use crate::pipeline::Pipeline;

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

pub fn exec_pipeline(pipeline: Pipeline) {
    let mut commands = vec![];
    let mut children: Vec<Child> = vec![];

    for shell_command in pipeline.commands {
        let Some(exec_path) = command_path(&shell_command.name) else {
            eprintln!("{}: command not found", shell_command.name);
            return;
        };

        let extern_cmd = build_command(exec_path, &shell_command);
        commands.push(extern_cmd);
    }

    let len = commands.len();

    for (i, mut command) in commands.into_iter().enumerate() {
        if i > 0 {
            let prev_child = &mut children[i - 1];

            let Some(stdout) = prev_child.stdout.take() else {
                eprintln!("Failed to take stdout of previous command");
                break;
            };

            command.stdin(stdout);
        }

        let is_last = i + 1 == len;
        if !is_last {
            command.stdout(Stdio::piped());
        }

        let Ok(child) = command.spawn() else {
            eprintln!(
                "Failed to spawn command: {}",
                command.get_program().to_string_lossy()
            );
            break;
        };

        children.push(child);
    }

    for child in &mut children {
        match child.wait() {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to wait for command: {e}"),
        }
    }
}
