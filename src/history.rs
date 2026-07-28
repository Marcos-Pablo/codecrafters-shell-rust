use std::{fs, io::Write, path::PathBuf};

use rustyline::history::History;

use crate::{Shell, builtin::Output, command::ShellCommand};

impl Shell {
    pub(crate) fn history_cmd(&mut self, command: &ShellCommand, output: &mut Output) {
        let first_arg = command.args.get(0);
        match first_arg.map(|s| s.as_str()) {
            Some("-r") => {
                let Some(path) = command.args.get(1).map(|s| s.as_str()) else {
                    writeln!(output.stdout, "usage: history -r <file>")
                        .expect("Error writing to stdout");
                    return;
                };
                self.load_history(path)
            }
            Some("-w") => {
                let Some(path) = command.args.get(1).map(|s| s.as_str()) else {
                    writeln!(output.stdout, "usage: history -w <file>")
                        .expect("Error writing to stdout");
                    return;
                };
                self.write_history(path);
            }
            Some("-a") => {
                let Some(path) = command.args.get(1).map(|s| s.as_str()) else {
                    writeln!(output.stdout, "usage: history -w <file>")
                        .expect("Error writing to stdout");
                    return;
                };
                self.append_history(path);
            }
            _ => self.list_history(command, output),
        }
    }

    fn list_history(&mut self, command: &ShellCommand, output: &mut Output) {
        let history = self.editor.history();

        let num_entries = command
            .args
            .get(0)
            .and_then(|arg| arg.parse::<usize>().ok())
            .unwrap_or(history.len());

        let offset = history.len() - num_entries;

        for (idx, entry) in history.iter().skip(offset).take(num_entries).enumerate() {
            let display_idx = idx + offset + 1;
            writeln!(output.stdout, "{:>5}  {entry}", display_idx)
                .expect("Error writing to stdout");
        }
    }

    pub(crate) fn load_history(&mut self, path: &str) {
        let history_path = PathBuf::from(path);
        history_path.exists().then(|| {
            if let Err(e) = self.editor.load_history(&history_path) {
                eprintln!(
                    "Failed to load history from {}: {e}",
                    history_path.display()
                );
            }
            self.history_baseline += self.editor.history().len() - self.history_baseline;
        });
    }

    pub(crate) fn write_history(&mut self, path: &str) {
        let history = self
            .editor
            .history()
            .iter()
            .map(|entry| entry.to_string() + "\n")
            .collect::<String>();

        if let Err(err) = fs::write(path, history) {
            eprintln!("Error writing history to file: {err}");
        }
        self.history_baseline += self.editor.history().len() - self.history_baseline;
    }

    // Appends only entries at or past `history_baseline` (everything before it
    // is already persisted by definition). The baseline advances after every
    // -a, -w, and -r: without that, a later -a would duplicate entries that a
    // previous -a wrote, or that -r loaded from a file.
    fn append_history(&mut self, path: &str) {
        let history = self
            .editor
            .history()
            .iter()
            .skip(self.history_baseline)
            .map(|entry| entry.to_string() + "\n")
            .collect::<String>();

        if let Err(err) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut file| file.write_all(history.as_bytes()))
        {
            eprintln!("Error appending history to file: {err}");
        }

        self.history_baseline += self.editor.history().len() - self.history_baseline;
    }
}
