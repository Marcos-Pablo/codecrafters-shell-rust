use std::error::Error;
use std::fs::{self, File};
use std::io;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use which::which;

use crate::parser::{Redirect, ShellCommand};

mod parser;

pub struct Shell {
    curr_dir: PathBuf,
}

struct Output {
    stdout: Box<dyn Write>,
    _stderr: Box<dyn Write>,
}

impl Output {
    fn new(command: &ShellCommand) -> Result<Output, Box<dyn Error>> {
        let mut stdout: Box<dyn Write> = Box::new(io::stdout());
        let mut stderr: Box<dyn Write> = Box::new(io::stderr());

        if let Some(redirect) = &command.redirect {
            match redirect {
                Redirect::Stdout(file_path) => match File::create(&file_path) {
                    Ok(file) => stdout = Box::new(file),
                    Err(e) => return Err(Box::new(e)),
                },
                Redirect::Stderr(file_path) => match File::create(&file_path) {
                    Ok(file) => stderr = Box::new(file),
                    Err(e) => return Err(Box::new(e)),
                },
                Redirect::AppendStdout(file_path) => {
                    match fs::OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(file_path)
                    {
                        Ok(file) => stdout = Box::new(file),
                        Err(e) => return Err(Box::new(e)),
                    }
                }
            }
        }

        Ok(Output {
            stdout,
            _stderr: stderr,
        })
    }
}

impl Shell {
    pub fn new(curr_dir: PathBuf) -> Shell {
        Shell { curr_dir }
    }

    pub fn run(mut self) {
        let mut input = String::new();

        loop {
            input.clear();
            print!("$ ");
            io::stdout().flush().expect("Error writing to stdout!");

            std::io::stdin()
                .read_line(&mut input)
                .expect("Error reading input");

            let Ok((remaining, tokens)) = parser::tokenize(&input.trim()) else {
                continue;
            };

            if !remaining.is_empty() {
                eprintln!("Couldn't parse the entire input, remaining portion: {remaining}");
                continue;
            }

            let command = match parser::parse_command(&tokens) {
                Ok(c) => c,
                Err(err) => {
                    eprintln!("{err}");
                    continue;
                }
            };

            match command.name.as_str() {
                "exit" => std::process::exit(0),
                "echo" => echo_cmd(&command),
                "type" => type_cmd(&command),
                "pwd" => self.pwd_cmd(&command),
                "cd" => self.cd_cmd(&command),
                cmd if let Some(exec_path) = command_path(cmd) => {
                    execute_command(exec_path, &command);
                }
                _ => println!("{}: command not found", command.name),
            }
        }
    }

    fn pwd_cmd(&self, command: &ShellCommand) {
        let mut output = match Output::new(&command) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };
        writeln!(output.stdout, "{}", self.curr_dir.display()).expect("Error writing to stdout");
    }

    fn cd_cmd(&mut self, command: &ShellCommand) {
        if command.args.len() > 1 {
            eprintln!("usage: cd <path>");
            return;
        }

        let path = command
            .args
            .get(0)
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string())
            .unwrap_or("~".to_string());

        let dest = if let Some(rest) = path.strip_prefix("~") {
            let Some(home_path) = home::home_dir() else {
                eprintln!("cd: cannot determine home directory");
                return;
            };
            home_path.join(rest.trim_start_matches('/'))
        } else {
            self.curr_dir.join(path)
        };

        match fs::canonicalize(&dest) {
            Ok(p) if p.is_dir() => self.curr_dir = p,
            _ => eprintln!("cd: {}: No such file or directory", dest.display()),
        }
    }
}

fn is_built_in(command: &str) -> bool {
    matches!(command, "echo" | "exit" | "type" | "pwd")
}

fn command_path(cmd: &str) -> Option<PathBuf> {
    which(cmd).ok()
}

fn execute_command(exec_path: PathBuf, command: &ShellCommand) {
    let mut extern_cmd = Command::new(&exec_path);
    extern_cmd.arg0(&command.name);
    extern_cmd.args(&command.args);

    if let Some(redirect) = &command.redirect {
        match redirect {
            Redirect::Stdout(file_path) => {
                let file = match File::create(&file_path) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("{e}");
                        return;
                    }
                };
                extern_cmd.stdout(Stdio::from(file));
            }
            Redirect::Stderr(file_path) => {
                let file = match File::create(&file_path) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("{e}");
                        return;
                    }
                };
                extern_cmd.stderr(Stdio::from(file));
            }
            Redirect::AppendStdout(file_path) => {
                let file = match fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(file_path)
                {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("{e}");
                        return;
                    }
                };
                extern_cmd.stdout(Stdio::from(file));
            }
        }
    }
    match &command.redirect {
        Some(parser::Redirect::Stdout(file_path)) => {
            let Ok(file) = File::create(&file_path) else {
                eprintln!("Error opening the file: {file_path}");
                return;
            };
            extern_cmd.stdout(Stdio::from(file));
        }
        _ => (),
    }

    match extern_cmd.status() {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to execute command: {e}"),
    }
}

fn echo_cmd(command: &ShellCommand) {
    let mut output = match Output::new(&command) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

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

fn type_cmd(command: &ShellCommand) {
    let mut output = match Output::new(&command) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    for arg in &command.args {
        match arg.as_str() {
            arg if is_built_in(arg) => {
                writeln!(output.stdout, "{arg} is a shell builtin")
                    .expect("Error writing to stdout");
            }
            arg if let Some(exec_path) = command_path(arg) => {
                writeln!(output.stdout, "{arg} is {}", exec_path.display())
                    .expect("Error writing to stdout");
            }
            _ => writeln!(output.stdout, "{arg}: not found").expect("Error writing to stdout"),
        }
    }
}
