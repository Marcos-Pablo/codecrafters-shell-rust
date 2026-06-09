use std::fs;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use which::which;

use crate::parser::{Token, TokenPart};

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

            let Ok((_, args)) = parser::tokenize(&input.trim()) else {
                continue;
            };
            dbg!(&args);

            let Some(first_token) = args.get(0) else {
                continue;
            };

            let Some(first_part) = first_token.parts.get(0) else {
                continue;
            };

            let command = match first_part {
                TokenPart::Unquoted(part) => part,
                _ => continue,
            };

            match command.as_str() {
                "exit" => std::process::exit(0),
                "echo" => echo_cmd(&args),
                "type" => type_cmd(&args),
                "pwd" => self.pwd_cmd(),
                "cd" => self.cd_cmd(&args),
                cmd if let Some(exec_path) = command_path(cmd) => {
                    execute_command(exec_path, &args);
                }
                _ => println!("{command}: command not found"),
            }
        }
    }

    fn pwd_cmd(&self) {
        println!("{}", self.curr_dir.display());
    }

    fn cd_cmd(&mut self, args: &[Token]) {
        if args.len() > 2 {
            eprintln!("usage: cd <path>");
            return;
        }

        let path = args
            .get(1)
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

fn execute_command(exec_path: PathBuf, args: &[Token]) {
    let arg0 = args[0].to_string();
    let args: Vec<String> = args.iter().skip(1).map(|token| token.to_string()).collect();

    let status = Command::new(&exec_path).arg0(&arg0).args(&args).status();

    match status {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to execute command: {e}"),
    }
}

fn echo_cmd(args: &[Token]) {
    args[1..]
        .iter()
        .for_each(|token| print!("{}", token.to_string()));
    println!();
}

fn type_cmd(args: &[Token]) {
    for arg in &args[1..] {
        let token = arg.to_string();
        match token.as_str() {
            arg if is_built_in(arg) => println!("{arg} is a shell builtin"),
            arg if let Some(exec_path) = command_path(arg) => {
                println!("{arg} is {}", exec_path.display())
            }
            _ => println!("{token}: not found"),
        }
    }
}
