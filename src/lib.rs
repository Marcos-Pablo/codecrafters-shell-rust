use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

use rustyline::Editor;
use rustyline::config::CompletionType;
use rustyline::history::FileHistory;

use crate::builtin::{BUILTINS, Output, echo_cmd, is_built_in, type_cmd};
use crate::command::{Redirect, ShellCommand};
use crate::completer::ShellHelper;
use crate::external::{command_path, execute_ext_command_foreground};
use crate::jobs::Jobs;
use crate::pipeline::ExecutionMode;

mod builtin;
mod command;
mod completer;
mod executor;
mod external;
mod history;
mod jobs;
mod parser;
mod pipeline;
mod token;
mod vars;

pub struct Shell {
    curr_dir: PathBuf,
    jobs: Jobs,
    editor: Editor<ShellHelper, FileHistory>,
    history_baseline: usize,
    vars: HashMap<String, String>,
}

const HISTFILE_ENV_VAR: &str = "HISTFILE";

impl Shell {
    pub fn new(curr_dir: PathBuf) -> Shell {
        let config = rustyline::Config::builder()
            .completion_type(CompletionType::List)
            .history_ignore_dups(false)
            .expect("Error creating rustyline config")
            .build();

        let mut editor = match Editor::with_config(config) {
            Ok(rl) => rl,
            Err(e) => panic!("Error creating editor: {e}"),
        };

        let helper = ShellHelper::new(BUILTINS);
        editor.set_helper(Some(helper));

        let mut shell = Shell {
            curr_dir,
            jobs: Jobs::new(),
            editor,
            history_baseline: 0,
            vars: HashMap::new(),
        };

        env::var(HISTFILE_ENV_VAR)
            .ok()
            .map(|path| shell.load_history(&path));

        shell
    }

    pub fn run(mut self) {
        loop {
            self.jobs.reap();
            let input = match self.editor.readline("$ ") {
                Ok(line) => line,
                Err(rustyline::error::ReadlineError::Eof) => break,
                Err(rustyline::error::ReadlineError::Interrupted) => continue,
                Err(err) => {
                    eprintln!("Error reading input: {err}");
                    break;
                }
            };

            let _ = self.editor.add_history_entry(input.as_str());

            let Ok((remaining, tokens)) = parser::tokenize(&input.trim()) else {
                continue;
            };

            if !remaining.is_empty() {
                eprintln!("Couldn't parse the entire input, remaining portion: {remaining}");
                continue;
            }

            let pipeline = match parser::parse_pipeline(&tokens) {
                Ok(p) => p,
                Err(err) => {
                    eprintln!("{err}");
                    continue;
                }
            };

            match pipeline.commands.len() {
                0 => eprintln!("Couldn't parse the input to a valid pipeline of commands"),
                1 => {
                    let command = &pipeline.commands[0];
                    if is_built_in(&command.name.as_str()) {
                        let (stdout, stderr) = match builtin::open_builtin_redirects(&command) {
                            Ok(pair) => pair,
                            Err(err) => {
                                eprintln!("{err}");
                                continue;
                            }
                        };

                        let mut output = Output::new(stdout, stderr);
                        self.run_built_in(command, &mut output);
                        continue;
                    }

                    if let Some(exec_path) = command_path(&command.name.as_str()) {
                        match pipeline.exec_mode {
                            ExecutionMode::Foreground => {
                                execute_ext_command_foreground(exec_path, &command);
                            }
                            ExecutionMode::Background => {
                                self.execute_ext_command_background(exec_path, &command);
                            }
                        }
                        continue;
                    }

                    println!("{}: command not found", command.name)
                }
                _ => self.exec_pipeline(pipeline),
            }
        }
    }

    fn run_built_in(&mut self, command: &ShellCommand, output: &mut Output) {
        match command.name.as_str() {
            "exit" => self.exit_cmd(0),
            "cd" => self.cd_cmd(&command),
            "echo" => echo_cmd(&command, output),
            "type" => type_cmd(&command, output),
            "pwd" => self.pwd_cmd(output),
            "complete" => self.complete_cmd(output, &command),
            "jobs" => self.jobs.list(output),
            "history" => self.history_cmd(&command, output),
            "declare" => self.declare_cmd(command, output),
            _ => println!("{}: command not found", command.name),
        }
    }

    fn exit_cmd(&mut self, code: i32) {
        env::var(HISTFILE_ENV_VAR)
            .ok()
            .map(|path| self.write_history(&path));

        std::process::exit(code);
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
            "-r" => self.remove_completion(output, command),
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
        let helper = self.editor.helper_mut().expect("ShellHelper not set");

        helper.register_ext_completion(target.to_string(), path.to_string());
    }

    fn get_completion(&mut self, output: &mut Output, command: &ShellCommand) {
        let Some(target) = command.args.get(1) else {
            writeln!(output.stdout, "usage: complete -p <command>")
                .expect("Error writing to stdout");
            return;
        };

        let helper = self.editor.helper_mut().expect("ShellHelper not set");
        match helper.get_ext_completion(target) {
            Some(path) => {
                let completion = format!("complete -C '{path}' {target}");
                writeln!(output.stdout, "{completion}").expect("Error writing to stdout")
            }
            None => writeln!(
                output.stdout,
                "complete: {target}: no completion specification",
            )
            .expect("Error writing to stdout"),
        }
    }

    fn remove_completion(&mut self, output: &mut Output, command: &ShellCommand) {
        let Some(target) = command.args.get(1) else {
            writeln!(output.stdout, "usage: complete -p <command>")
                .expect("Error writing to stdout");
            return;
        };

        let helper = self.editor.helper_mut().expect("ShellHelper not set");
        helper.remove_ext_completion(target);
    }
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
