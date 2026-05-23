#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut cmd = String::new();
    std::io::stdin()
        .read_line(&mut cmd)
        .expect("Error reading input");

    let cmd = cmd.trim();

    println!("{cmd}: command not found");
}
