use codecrafters_shell::Shell;

fn main() {
    let Ok(curr_dir) = std::env::current_dir() else {
        panic!("Error getting current directory");
    };

    let shell = Shell::new(curr_dir);
    shell.run();
}
