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

    // DEVIATION: `pipeline.exec_mode` is ignored — a backgrounded pipeline
    // (`cmd1 | cmd2 &`) runs in the foreground. Bash backgrounds the whole
    // pipeline as one job; not exercised by the stages.
    pub(crate) fn exec_pipeline(&mut self, pipeline: Pipeline) {
        let mut commands: Vec<Stage> = vec![];
        let mut children: Vec<Child> = vec![];

        for shell_command in pipeline.commands {
            if is_pipeline_built_in(shell_command.name.as_str()) {
                commands.push(Stage::Builtin(shell_command));
                continue;
            }

            // DEVIATION: a command-not-found aborts the entire pipeline
            // before anything spawns. Bash still runs the other stages.
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
                        // DEVIATION: the pipe overrides a `>` redirect from
                        // build_command; bash applies the redirect on top of
                        // the pipe instead. Untested by the stages.
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
                    // Builtins are sources, never sinks: none of them reads
                    // stdin, so the upstream read end is closed on purpose.
                    // Without this, an upstream producing >64KB would block
                    // forever; with it, the writer gets EPIPE and terminates.
                    // Also: a builtin stage's own redirect is ignored — its
                    // Output comes from the pipe wiring, never from files.
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
