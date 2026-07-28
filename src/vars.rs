use crate::{Shell, builtin::Output, command::ShellCommand};

impl Shell {
    pub(crate) fn declare_cmd(&mut self, command: &ShellCommand, output: &mut Output) {
        let first_arg = command.args.get(0);
        match first_arg.map(|arg| arg.as_str()) {
            Some("-p") => {
                let var_name = command.args.get(1).map(|s| s.as_str());
                match var_name {
                    Some(name) => self.display_var(name, output),
                    None => writeln!(output.stdout, "Usage: declare [-p] [name]")
                        .expect("Error writing to stdout"),
                };
            }
            _ => writeln!(output.stdout, "Usage: declare [-p] [name]")
                .expect("Error writing to stdout"),
        };
    }

    fn display_var(&self, name: &str, output: &mut Output) {
        let value = self.vars.get(name);
        match value {
            Some(value) => {
                writeln!(output.stdout, "{name}={value}").expect("Error writing to stdout")
            }
            None => writeln!(output.stdout, "declare: {name}: not found")
                .expect("Error writing to stdout"),
        };
    }
}
