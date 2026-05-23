#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

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
            "exit" => exit(0),
            "echo" => echo(command, args),
            _ => println!("{}: command not found", input.trim()),
        }
    }
}

fn echo(_command: &str, args: Vec<&str>) {
    let output = args.join(" ");
    println!("{output}");
}
