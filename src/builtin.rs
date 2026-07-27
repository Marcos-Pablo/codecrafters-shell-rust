use std::io;
use std::io::Write;

use crate::command::{Redirect, ShellCommand};
use crate::open_file_redirects;

pub const BUILTINS: &[&str] = &[
    "echo", "exit", "type", "pwd", "complete", "jobs", "cd", "history",
];

pub const PIPELINE_BUILTINS: &[&str] = &["echo", "type", "pwd", "history"];

pub struct Output {
    pub stdout: Box<dyn Write>,
    pub _stderr: Box<dyn Write>,
}

impl Output {
    pub fn new(stdout: Box<dyn Write>, stderr: Box<dyn Write>) -> Output {
        Output {
            stdout,
            _stderr: stderr,
        }
    }
}

pub fn echo_cmd(command: &ShellCommand, output: &mut Output) {
    let mut first = true;

    for arg in &command.args {
        if !first {
            write!(output.stdout, " ").expect("Error writing to stdout");
        }
        write!(output.stdout, "{arg}").expect("Error writing to stdout");

        first = false;
    }
    writeln!(output.stdout).expect("Error writing to stdout");
}

pub fn type_cmd(command: &ShellCommand, output: &mut Output) {
    for arg in &command.args {
        match arg.as_str() {
            arg if is_built_in(arg) => {
                writeln!(output.stdout, "{arg} is a shell builtin")
                    .expect("Error writing to stdout");
            }
            arg if let Some(exec_path) = crate::command_path(arg) => {
                writeln!(output.stdout, "{arg} is {}", exec_path.display())
                    .expect("Error writing to stdout");
            }
            _ => writeln!(output.stdout, "{arg}: not found").expect("Error writing to stdout"),
        }
    }
}

pub fn open_builtin_redirects(
    command: &ShellCommand,
) -> Result<(Box<dyn Write>, Box<dyn Write>), io::Error> {
    let mut stdout: Box<dyn Write> = Box::new(io::stdout());
    let mut stderr: Box<dyn Write> = Box::new(io::stderr());

    if let Some(redirect) = &command.redirect {
        let file = Box::new(open_file_redirects(redirect)?);
        match redirect {
            Redirect::Stdout(_) | Redirect::AppendStdout(_) => stdout = file,
            Redirect::Stderr(_) | Redirect::AppendStderr(_) => stderr = file,
        }
    }

    Ok((stdout, stderr))
}

pub fn is_built_in(cmd_name: &str) -> bool {
    BUILTINS.contains(&cmd_name)
}

pub fn is_pipeline_built_in(cmd_name: &str) -> bool {
    PIPELINE_BUILTINS.contains(&cmd_name)
}
