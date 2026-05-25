use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use which::which;

fn main() {
    let mut input = String::new();

    loop {
        input.clear();
        print!("$ ");
        io::stdout().flush().expect("Error writing to stdout!");

        std::io::stdin()
            .read_line(&mut input)
            .expect("Error reading input");

        let mut input_iter = input.trim().split_whitespace();
        let command = match input_iter.next() {
            Some(val) => val,
            None => continue,
        };

        let args: Vec<&str> = input_iter.collect();

        match command {
            "exit" => std::process::exit(0),
            "echo" => echo_cmd(args),
            "type" => type_cmd(args),
            arg if let Some(exec_path) = command_path(arg) => {
                execute_command_path(exec_path, args);
            }
            _ => println!("{command}: command not found"),
        }
    }
}

fn is_built_in(command: &str) -> bool {
    matches!(command, "echo" | "exit" | "type")
}

fn command_path(arg: &str) -> Option<PathBuf> {
    which(arg).ok()
}

fn execute_command_path(exec_path: PathBuf, args: Vec<&str>) {
    let status = Command::new(exec_path).args(args).status();
    match status {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to execute command: {e}"),
    }
}

fn echo_cmd(args: Vec<&str>) {
    let output = args.join(" ");
    println!("{output}");
}

fn type_cmd(args: Vec<&str>) {
    for arg in args {
        match arg {
            arg if is_built_in(arg) => println!("{arg} is a shell builtin"),
            arg if let Some(exec_path) = command_path(arg) => {
                println!("{arg} is {}", exec_path.display())
            }
            _ => println!("{arg}: not found"),
        }
    }
}
