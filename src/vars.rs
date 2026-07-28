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
            Some(key_val_pair) => {
                let parts: Vec<&str> = key_val_pair.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let name = parts[0];
                    let value = parts[1];
                    let is_valid_name = self.validate_var_name(name);
                    if !is_valid_name {
                        writeln!(
                            output.stdout,
                            "declare: `{name}={value}': not a valid identifier"
                        )
                        .expect("Error writing to stdout");
                        return;
                    }
                    self.vars.insert(name.to_string(), value.to_string());
                } else {
                    writeln!(output.stdout, "Usage: declare [-p] [name]")
                        .expect("Error writing to stdout");
                }
            }
            _ => writeln!(output.stdout, "Usage: declare [-p] [name]")
                .expect("Error writing to stdout"),
        };
    }

    fn display_var(&self, name: &str, output: &mut Output) {
        let value = self.vars.get(name);
        match value {
            Some(value) => writeln!(output.stdout, "declare -- {name}=\"{value}\"")
                .expect("Error writing to stdout"),
            None => writeln!(output.stdout, "declare: {name}: not found")
                .expect("Error writing to stdout"),
        };
    }

    fn validate_var_name(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let mut chars = name.chars();
        let first_char = chars.next();
        match first_char {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => true,
            _ => return false,
        };

        while let Some(c) = chars.next() {
            if !c.is_ascii_alphanumeric() && c != '_' {
                return false;
            }
        }

        true
    }
}
