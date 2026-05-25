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

        let args: Vec<&str> = input.trim().split_whitespace().collect();
        let Some(&command) = args.get(0) else {
            continue;
        };

        match command {
            "exit" => std::process::exit(0),
            "echo" => echo_cmd(&args),
            "type" => type_cmd(&args),
            cmd if let Some(exec_path) = command_path(cmd) => {
                execute_command_path(exec_path, &args);
            }
            _ => println!("{command}: command not found"),
        }
    }
}

fn is_built_in(command: &str) -> bool {
    matches!(command, "echo" | "exit" | "type")
}

fn command_path(cmd: &str) -> Option<PathBuf> {
    which(cmd).ok()
}

fn execute_command_path(exec_path: PathBuf, args: &[&str]) {
    dbg!(&exec_path);
    dbg!(&args[1..]);
    let status = Command::new(exec_path).args(&args[1..]).status();
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
