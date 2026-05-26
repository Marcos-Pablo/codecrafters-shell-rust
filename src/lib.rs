use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use which::which;

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

            let args: Vec<&str> = input.trim().split_whitespace().collect();
            let Some(&command) = args.get(0) else {
                continue;
            };

            match command {
                "exit" => std::process::exit(0),
                "echo" => echo_cmd(&args),
                "type" => type_cmd(&args),
                "pwd" => self.pwd_cmd(),
                "cd" => self.cd_cmd(&args),
                cmd if let Some(exec_path) = command_path(cmd) => {
                    execute_command_path(exec_path, &args);
                }
                _ => println!("{command}: command not found"),
            }
        }
    }

    fn pwd_cmd(&self) {
        println!("{}", self.curr_dir.display());
    }

    fn cd_cmd(&mut self, args: &[&str]) {
        if args.len() != 2 {
            eprintln!("usage: cd <path>");
            return;
        }

        let absolute_path = args[1];
        let dest = Path::new(absolute_path);
        match dest.is_dir() {
            true => self.curr_dir = dest.to_path_buf(),
            false => println!("cd: {absolute_path}: No such file or directory"),
        }
    }
}

fn is_built_in(command: &str) -> bool {
    matches!(command, "echo" | "exit" | "type" | "pwd")
}

fn command_path(cmd: &str) -> Option<PathBuf> {
    which(cmd).ok()
}

fn execute_command_path(exec_path: PathBuf, args: &[&str]) {
    let status = Command::new(&exec_path)
        .arg0(&args[0])
        .args(&args[1..])
        .status();

    match status {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to execute command: {e}"),
    }
}

fn echo_cmd(args: &[&str]) {
    let output = args[1..].join(" ");
    println!("{output}");
}

fn type_cmd(args: &[&str]) {
    for &arg in &args[1..] {
        match arg {
            arg if is_built_in(arg) => println!("{arg} is a shell builtin"),
            arg if let Some(exec_path) = command_path(arg) => {
                println!("{arg} is {}", exec_path.display())
            }
            _ => println!("{arg}: not found"),
        }
    }
}
