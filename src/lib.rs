use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use rustyline::Editor;
use rustyline::config::CompletionType;
use which::which;

use crate::builtin::{Output, echo_cmd, type_cmd};
use crate::completer::ShellHelper;
use crate::external::execute_command;
use crate::parser::{Redirect, ShellCommand};

mod builtin;
mod completer;
mod external;
mod parser;

pub struct Shell {
    curr_dir: PathBuf,
    completions: HashMap<String, String>,
}

impl Shell {
    pub fn new(curr_dir: PathBuf) -> Shell {
        Shell {
            curr_dir,
            completions: HashMap::new(),
        }
    }

    pub fn run(mut self) {
        let config = rustyline::Config::builder()
            .completion_type(CompletionType::List)
            .build();

        let mut editor = match Editor::with_config(config) {
            Ok(rl) => rl,
            Err(e) => panic!("Error creating editor: {e}"),
        };

        let builtins = builtin::get_builtins();
        let helper = ShellHelper::new(builtins);
        editor.set_helper(Some(helper));

        loop {
            let input = editor.readline("$ ").expect("Error reading input");

            let Ok((remaining, tokens)) = parser::tokenize(&input.trim()) else {
                continue;
            };

            if !remaining.is_empty() {
                eprintln!("Couldn't parse the entire input, remaining portion: {remaining}");
                continue;
            }

            let command = match parser::parse_command(&tokens) {
                Ok(c) => c,
                Err(err) => {
                    eprintln!("{err}");
                    continue;
                }
            };

            match command.name.as_str() {
                "exit" => std::process::exit(0),
                "cd" => self.cd_cmd(&command),
                "echo" | "type" | "pwd" | "complete" => {
                    let (stdout, stderr) = match builtin::open_builtin_redirects(&command) {
                        Ok(pair) => pair,
                        Err(err) => {
                            eprintln!("{err}");
                            continue;
                        }
                    };
                    let mut output = Output::new(stdout, stderr);

                    match command.name.as_str() {
                        "echo" => echo_cmd(&command, &mut output),
                        "type" => type_cmd(&command, &mut output),
                        "pwd" => self.pwd_cmd(&mut output),
                        "complete" => self.complete_cmd(&mut output, &command),
                        _ => unreachable!(),
                    }
                }
                cmd if let Some(exec_path) = command_path(cmd) => {
                    execute_command(exec_path, &command);
                }
                _ => println!("{}: command not found", command.name),
            }
        }
    }

    fn pwd_cmd(&self, output: &mut Output) {
        writeln!(output.stdout, "{}", self.curr_dir.display()).expect("Error writing to stdout");
    }

    fn cd_cmd(&mut self, command: &ShellCommand) {
        if command.args.len() > 1 {
            eprintln!("usage: cd <path>");
            return;
        }

        let path = command
            .args
            .get(0)
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string())
            .unwrap_or("~".to_string());

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
            Ok(p) if p.is_dir() => {
                std::env::set_current_dir(&p)
                    .expect("Error while updating the current process dir");
                self.curr_dir = p;
            }
            _ => eprintln!("cd: {}: No such file or directory", dest.display()),
        }
    }

    fn complete_cmd(&mut self, output: &mut Output, command: &ShellCommand) {
        let Some(flag) = command.args.get(0) else {
            writeln!(output.stdout, "The flag is required").expect("Error writing to stdout");
            return;
        };

        match flag.as_str() {
            "-C" => self.register_completion(output, command),
            "-p" => self.get_completion(output, command),
            _ => {
                writeln!(output.stdout, "Invalid flag").expect("Error writing to stdout");
            }
        }
    }

    fn register_completion(&mut self, output: &mut Output, command: &ShellCommand) {
        let path = command.args.get(1);
        let target = command.args.get(2);

        if path.is_none() || target.is_none() {
            writeln!(output.stdout, "usage: complete -C <path> <command>")
                .expect("Error writing to stdout");
            return;
        }

        let path = path.unwrap();
        let target = target.unwrap();

        let completion = format!("complete -C '{path}' {target}");

        self.completions.insert(target.clone(), completion);
    }

    fn get_completion(&self, output: &mut Output, command: &ShellCommand) {
        let Some(target) = command.args.get(1) else {
            writeln!(output.stdout, "usage: complete -p <command>")
                .expect("Error writing to stdout");
            return;
        };

        match self.completions.get(target) {
            Some(completion) => {
                writeln!(output.stdout, "{completion}").expect("Error writing to stdout")
            }
            None => writeln!(
                output.stdout,
                "complete: {target}: no completion specification",
            )
            .expect("Error writing to stdout"),
        }
    }
}

fn command_path(cmd: &str) -> Option<PathBuf> {
    which(cmd).ok()
}

fn open_file_redirects(redirect: &Redirect) -> std::io::Result<std::fs::File> {
    match redirect {
        Redirect::Stdout(file_path) | Redirect::Stderr(file_path) => {
            Ok(std::fs::File::create(&file_path)?)
        }
        Redirect::AppendStdout(file_path) | Redirect::AppendStderr(file_path) => {
            Ok(fs::OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(file_path)?)
        }
    }
}
