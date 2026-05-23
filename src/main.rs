#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut command = String::new();
    std::io::stdin()
        .read_line(&mut command)
        .expect("Error reading input");

    println!("{}: command not found", command.trim());
}
