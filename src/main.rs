#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().expect("Error writing to stdout!");

        let mut input = String::new();
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
            "echo" => echo(args),
            "type" => type_cmd(args),
            _ => println!("{}: command not found", input.trim()),
        }
    }
}

fn is_built_in(command: &str) -> bool {
    matches!(command, "echo" | "exit" | "type")
}

fn echo(args: Vec<&str>) {
    let output = args.join(" ");
    println!("{output}");
}

fn type_cmd(args: Vec<&str>) {
    for arg in args {
        match is_built_in(arg) {
            true => println!("{arg} is a shell builtin"),
            false => println!("{arg}: not found"),
        }
    }
}
