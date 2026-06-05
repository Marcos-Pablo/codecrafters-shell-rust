use std::fs;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
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

            let args = tokenize(&input);

            let Some(command) = args.get(0) else {
                continue;
            };

            match command.as_str() {
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

    fn cd_cmd(&mut self, args: &[String]) {
        if args.len() > 2 {
            eprintln!("usage: cd <path>");
            return;
        }

        let path = match args.get(1) {
            Some(p) => p.as_str(),
            None => "~",
        };

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

fn tokenize(input: &str) -> Vec<String> {
    let mut inside_quotes = false;
    let mut tokens = Vec::new();
    let mut current = String::new();

    for c in input.trim().chars() {
        if c == '\'' && !inside_quotes {
            inside_quotes = true;
            continue;
        }

        if c == '\'' && inside_quotes {
            inside_quotes = false;
            continue;
        }

        if c.is_whitespace() && !inside_quotes {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            continue;
        }

        current.push(c);
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn is_built_in(command: &str) -> bool {
    matches!(command, "echo" | "exit" | "type" | "pwd")
}

fn command_path(cmd: &str) -> Option<PathBuf> {
    which(cmd).ok()
}

fn execute_command_path(exec_path: PathBuf, args: &[String]) {
    let status = Command::new(&exec_path)
        .arg0(&args[0])
        .args(&args[1..])
        .status();

    match status {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to execute command: {e}"),
    }
}

fn echo_cmd(args: &[String]) {
    let content = args[1..].join(" ");

    println!("{content}");
}

fn type_cmd(args: &[String]) {
    for arg in &args[1..] {
        match arg.as_str() {
            arg if is_built_in(arg) => println!("{arg} is a shell builtin"),
            arg if let Some(exec_path) = command_path(arg) => {
                println!("{arg} is {}", exec_path.display())
            }
            _ => println!("{arg}: not found"),
        }
    }
}
