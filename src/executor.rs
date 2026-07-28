use std::{
    io::{self, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use crate::{
    Shell,
    builtin::{Output, is_pipeline_built_in},
    command::ShellCommand,
    external::{build_command, command_path},
    pipeline::Pipeline,
};

pub(crate) enum Stage {
    Builtin(ShellCommand),
    External(Command),
}

impl Shell {
    pub(crate) fn execute_ext_command_background(
        &mut self,
        exec_path: PathBuf,
        command: &ShellCommand,
    ) {
        let mut extern_cmd = build_command(exec_path, command);
        match extern_cmd.spawn() {
            Ok(child) => {
                let cmt_str = command.to_string();
                let pid = child.id();
                let id = self.jobs.add(child, cmt_str);
                println!("[{id}] {pid}");
            }
            Err(e) => eprintln!("Failed to spawn background task: {e}"),
        };
    }

    pub(crate) fn exec_pipeline(&mut self, pipeline: Pipeline) {
        let mut commands: Vec<Stage> = vec![];
        let mut children: Vec<Child> = vec![];

        for shell_command in pipeline.commands {
            if is_pipeline_built_in(shell_command.name.as_str()) {
                commands.push(Stage::Builtin(shell_command));
                continue;
            }

            let Some(exec_path) = command_path(&shell_command.name) else {
                eprintln!("{}: command not found", shell_command.name);
                return;
            };

            let extern_cmd = build_command(exec_path, &shell_command);
            commands.push(Stage::External(extern_cmd));
        }

        let len = commands.len();
        let mut prev_reader: Option<io::PipeReader> = None;

        for (i, stage) in commands.into_iter().enumerate() {
            let is_last = i + 1 == len;

            let pipe = if is_last {
                None
            } else {
                match io::pipe() {
                    Ok(pipe) => Some(pipe),
                    Err(e) => {
                        eprintln!("Failed to create pipe: {e}");
                        break;
                    }
                }
            };

            match stage {
                Stage::External(mut cmd) => {
                    if let Some(reader) = prev_reader.take() {
                        cmd.stdin(Stdio::from(reader));
                    }

                    if let Some((reader, writer)) = pipe {
                        cmd.stdout(Stdio::from(writer));
                        prev_reader = Some(reader);
                    }

                    let Ok(child) = cmd.spawn() else {
                        eprintln!(
                            "Failed to spawn command: {}",
                            cmd.get_program().to_string_lossy()
                        );
                        break;
                    };
                    children.push(child);
                }
                Stage::Builtin(cmd) => {
                    drop(prev_reader.take());
                    match pipe {
                        Some((reader, writer)) => {
                            {
                                let stdout: Box<dyn Write> = Box::new(writer);
                                let stderr: Box<dyn Write> = Box::new(io::stderr());
                                let mut output = Output::new(stdout, stderr);
                                self.run_built_in(&cmd, &mut output);
                            }

                            prev_reader = Some(reader);
                        }
                        None => {
                            let stdout: Box<dyn Write> = Box::new(io::stdout());
                            let stderr: Box<dyn Write> = Box::new(io::stderr());
                            let mut output = Output::new(stdout, stderr);
                            self.run_built_in(&cmd, &mut output);
                        }
                    }
                }
            }
        }

        for child in &mut children {
            match child.wait() {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to wait for command: {e}"),
            }
        }
    }
}
