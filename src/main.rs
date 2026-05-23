#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        std::io::stdin()
            .read_line(&mut command)
            .expect("Error reading input");

        match command.trim() {
            "exit" => exit(0),
            _ => println!("{}: command not found", command.trim()),
        }
    }
}
