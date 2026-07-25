#[derive(Debug)]
pub enum Redirect {
    Stdout(String),
    Stderr(String),
    AppendStdout(String),
    AppendStderr(String),
}

#[derive(Debug)]
pub enum ExecutionMode {
    Foreground,
    Background,
}

#[derive(Debug)]
pub struct ShellCommand {
    pub name: String,
    pub args: Vec<String>,
    pub redirect: Option<Redirect>,
    pub exec_mode: ExecutionMode,
}

impl ShellCommand {
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        result.push_str(&self.name);
        for arg in &self.args {
            result.push(' ');
            result.push_str(arg);
        }
        if let Some(ref redirect) = self.redirect {
            result.push(' ');
            match redirect {
                Redirect::Stdout(file) => {
                    result.push_str(">");
                    result.push(' ');
                    result.push_str(file);
                }
                Redirect::Stderr(file) => {
                    result.push_str("2>");
                    result.push(' ');
                    result.push_str(file);
                }
                Redirect::AppendStdout(file) => {
                    result.push_str(">>");
                    result.push(' ');
                    result.push_str(file);
                }
                Redirect::AppendStderr(file) => {
                    result.push_str("2>>");
                    result.push(' ');
                    result.push_str(file);
                }
            }
        }

        result
    }
}
