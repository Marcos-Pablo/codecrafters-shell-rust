#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::PathBuf;
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

fn echo_cmd(args: Vec<&str>) {
    let output = args.join(" ");
    println!("{output}\n");
}

fn type_cmd(args: Vec<&str>) {
    for arg in args {
        match arg {
            arg if is_built_in(arg) => println!("{arg} is a shell builtin"),
            arg if let Some(exec_path) = command_path(arg) => {
                println!("{arg}: is {}", exec_path.display())
            }
            _ => println!("{arg}: not found"),
        }
    }
    println!();
}
