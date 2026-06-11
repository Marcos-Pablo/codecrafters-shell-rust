use std::fs::{self, File};
use std::io;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use which::which;

use crate::parser::ShellCommand;

mod parser;

pub struct Shell {
    curr_dir: PathBuf,
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

            let mut file_handle;
            let writer: &mut dyn Write = match &command.redirect {
                Some(parser::Redirect::Stdout(file_path)) => match File::create(&file_path) {
                    Ok(file) => {
                        file_handle = Some(file);
                        file_handle.as_mut().unwrap()
                    }
                    Err(err) => {
                        eprintln!("Error opening redirect file: {file_path}: {err}");
                        continue;
                    }
                },
                _ => &mut io::stdout(),
            };

            match command.name.as_str() {
                "exit" => std::process::exit(0),
                "echo" => echo_cmd(&command, writer),
                "type" => type_cmd(&command, writer),
                "pwd" => self.pwd_cmd(writer),
                "cd" => self.cd_cmd(&command),
                cmd if let Some(exec_path) = command_path(cmd) => {
                    execute_command(exec_path, &command);
                }
                _ => println!("{}: command not found", command.name),
            }
        }
    }

    fn pwd_cmd(&self, writer: &mut dyn Write) {
        writeln!(writer, "{}", self.curr_dir.display()).expect("Error writing to stdout");
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

fn echo_cmd(command: &ShellCommand, writer: &mut dyn Write) {
    let mut first = true;

    for arg in &command.args {
        if !first {
            write!(writer, " ").expect("Error writing to stdout");
        }
        write!(writer, "{arg}").expect("Error writing to stdout");

        first = false;
    }
    writeln!(writer).expect("Error writing to stdout");
}

fn type_cmd(command: &ShellCommand, writer: &mut dyn Write) {
    for arg in &command.args {
        match arg.as_str() {
            arg if is_built_in(arg) => {
                writeln!(writer, "{arg} is a shell builtin").expect("Error writing to stdout");
            }
            arg if let Some(exec_path) = command_path(arg) => {
                writeln!(writer, "{arg} is {}", exec_path.display())
                    .expect("Error writing to stdout");
            }
            _ => writeln!(writer, "{arg}: not found").expect("Error writing to stdout"),
        }
    }
}
