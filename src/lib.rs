use std::fs;
use std::io::Write;
use std::path::PathBuf;

use rustyline::Editor;
use rustyline::config::CompletionType;

use crate::builtin::{Output, echo_cmd, type_cmd};
use crate::completer::ShellHelper;
use crate::external::{build_command, command_path, execute_ext_command_foreground};
use crate::parser::{ExecutionMode, Redirect, ShellCommand};

mod builtin;
mod completer;
mod external;
mod parser;

pub struct Shell {
    curr_dir: PathBuf,
    jobs: Vec<Job>,
}

struct Job {
    id: u32,
    child: std::process::Child,
    command: String,
    status: JobStatus,
}

#[derive(PartialEq, Eq)]
enum JobStatus {
    Running,
    Done,
}

impl JobStatus {
    fn as_padded_str(&self) -> &'static str {
        match self {
            JobStatus::Running => "Running                 ",
            JobStatus::Done => "Done                    ",
        }
    }
}

impl Shell {
    pub fn new(curr_dir: PathBuf) -> Shell {
        Shell {
            curr_dir,
            jobs: Vec::new(),
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
            self.reap_jobs();
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
                "echo" | "type" | "pwd" | "complete" | "jobs" => {
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
                        "complete" => self.complete_cmd(
                            &mut output,
                            &command,
                            editor.helper_mut().expect("No helper set"),
                        ),
                        "jobs" => self.jobs_cmd(&mut output),
                        _ => unreachable!(),
                    }
                }
                cmd_name if let Some(exec_path) = command_path(cmd_name) => {
                    match command.exec_mode {
                        ExecutionMode::Foreground => {
                            execute_ext_command_foreground(exec_path, &command);
                        }
                        ExecutionMode::Background => {
                            self.execute_ext_command_background(exec_path, &command);
                        }
                    }
                }
                _ => println!("{}: command not found", command.name),
            }
        }
    }

    pub fn get_next_id(&self) -> u32 {
        let mut id = 1;
        loop {
            let available = self.jobs.iter().all(|job| job.id != id);
            if available {
                break;
            }
            id += 1;
        }
        id
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

    fn complete_cmd(
        &mut self,
        output: &mut Output,
        command: &ShellCommand,
        helper: &mut ShellHelper,
    ) {
        let Some(flag) = command.args.get(0) else {
            writeln!(output.stdout, "The flag is required").expect("Error writing to stdout");
            return;
        };

        match flag.as_str() {
            "-C" => self.register_completion(output, command, helper),
            "-p" => self.get_completion(output, command, helper),
            "-r" => self.remove_completion(output, command, helper),
            _ => {
                writeln!(output.stdout, "Invalid flag").expect("Error writing to stdout");
            }
        }
    }

    fn register_completion(
        &mut self,
        output: &mut Output,
        command: &ShellCommand,
        helper: &mut ShellHelper,
    ) {
        let path = command.args.get(1);
        let target = command.args.get(2);

        if path.is_none() || target.is_none() {
            writeln!(output.stdout, "usage: complete -C <path> <command>")
                .expect("Error writing to stdout");
            return;
        }

        let path = path.unwrap();
        let target = target.unwrap();

        helper.register_ext_completion(target.to_string(), path.to_string());
    }

    fn get_completion(
        &self,
        output: &mut Output,
        command: &ShellCommand,
        helper: &mut ShellHelper,
    ) {
        let Some(target) = command.args.get(1) else {
            writeln!(output.stdout, "usage: complete -p <command>")
                .expect("Error writing to stdout");
            return;
        };

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

    fn remove_completion(
        &self,
        output: &mut Output,
        command: &ShellCommand,
        helper: &mut ShellHelper,
    ) {
        let Some(target) = command.args.get(1) else {
            writeln!(output.stdout, "usage: complete -p <command>")
                .expect("Error writing to stdout");
            return;
        };

        helper.remove_ext_completion(target);
    }

    fn execute_ext_command_background(&mut self, exec_path: PathBuf, command: &ShellCommand) {
        let mut extern_cmd = build_command(exec_path, command);
        match extern_cmd.spawn() {
            Ok(child) => {
                let id = self.get_next_id();
                println!("[{id}] {}", child.id());
                let cmt_str = command.to_string();
                let job = Job {
                    id,
                    child,
                    command: cmt_str,
                    status: JobStatus::Running,
                };
                self.jobs.push(job);
            }
            Err(e) => eprintln!("Failed to spawn background task: {e}"),
        };
    }

    fn jobs_cmd(&mut self, output: &mut Output) {
        let last_index = self.jobs.len() - 1;

        for (i, job) in self.jobs.iter_mut().enumerate() {
            let marker = match i {
                i if i == last_index => "+",
                i if i == last_index - 1 => "-",
                _ => " ",
            };

            match job.child.try_wait() {
                Ok(Some(_)) => job.status = JobStatus::Done,
                Ok(None) | Err(_) => (),
            }

            writeln!(
                output.stdout,
                "[{}]{marker}  {} {}",
                job.id,
                job.status.as_padded_str(),
                job.command
            )
            .expect("Error writing to stdout");
        }
        self.jobs.retain(|job| job.status != JobStatus::Done);
    }

    fn reap_jobs(&mut self) {
        let last_index = self.jobs.len() - 1;
        for (i, job) in self.jobs.iter_mut().enumerate() {
            match job.child.try_wait() {
                Ok(Some(_)) => {
                    job.status = JobStatus::Done;
                    let marker = match i {
                        i if i == last_index => "+",
                        i if i == last_index - 1 => "-",
                        _ => " ",
                    };
                    println!(
                        "[{}]{marker}  {} {}",
                        job.id,
                        job.status.as_padded_str(),
                        job.command
                    );
                }
                Ok(None) | Err(_) => (),
            }
        }
        self.jobs.retain(|job| job.status != JobStatus::Done);
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
