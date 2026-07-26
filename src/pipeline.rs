use crate::command::ShellCommand;

#[derive(Debug)]
pub enum ExecutionMode {
    Foreground,
    Background,
}

#[derive(Debug)]
pub struct Pipeline {
    pub commands: Vec<ShellCommand>,
    pub exec_mode: ExecutionMode,
}

impl Pipeline {
    pub fn new() -> Pipeline {
        Pipeline {
            commands: vec![],
            exec_mode: ExecutionMode::Foreground,
        }
    }

    pub fn add_command(&mut self, cmd: ShellCommand) {
        self.commands.push(cmd);
    }
}
