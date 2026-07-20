use rustyline::Helper;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use std::collections::HashMap;
use std::env;
use std::fs;

use crate::external::{command_path, ext_command_output};

pub struct ShellHelper {
    builtins: &'static [&'static str],
    file_completer: FilenameCompleter,
    prog_completions: HashMap<String, String>,
}

impl ShellHelper {
    pub fn new(builtins: &'static [&'static str]) -> ShellHelper {
        ShellHelper {
            builtins,
            file_completer: FilenameCompleter::new(),
            prog_completions: HashMap::new(),
        }
    }

    fn find_candidates(&self, prefix: &str) -> Vec<Pair> {
        let mut candidates = vec![];

        for &candidate in self.builtins {
            if candidate.starts_with(prefix) {
                candidates.push(Pair {
                    display: candidate.to_string(),
                    replacement: candidate.to_string() + " ",
                });
            }
        }

        let full_path = env::var_os("PATH").unwrap_or_default();
        for dir in env::split_paths(&full_path) {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };

            for entry in entries.flatten() {
                let name = match entry.file_name().into_string() {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                if name.starts_with(prefix) {
                    candidates.push(Pair {
                        display: name.clone(),
                        replacement: name + " ",
                    });
                }
            }
        }

        candidates.sort_by(|a, b| a.display.cmp(&b.display));
        candidates.dedup_by(|a, b| a.display == b.display);
        candidates
    }

    pub fn register_ext_completion(&mut self, target: String, path: String) {
        self.prog_completions.insert(target, path);
    }

    pub fn get_ext_completion(&self, target: &str) -> Option<&str> {
        return self.prog_completions.get(target).map(|s| s.as_str());
    }

    pub fn remove_ext_completion(&mut self, target: &String) {
        self.prog_completions.remove(target);
    }
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let &start = &line[..pos].rfind(' ').map(|i| i + 1).unwrap_or(0);
        let prefix = &line[start..pos];
        let trimmed = &line[..pos].trim_start();
        let is_first_word = trimmed.find(' ').is_none();

        if !is_first_word {
            let cmd_end = trimmed.find(' ').unwrap_or(trimmed.len());
            let cmd_name = &trimmed[..cmd_end];

            if let Some(path) = self.get_ext_completion(cmd_name) {
                if let Some(path_buf) = command_path(path) {
                    // `complete -C` contract: the completer is invoked with
                    // $1 = command name, $2 = word being completed,
                    // $3 = word preceding the word being completed
                    let prev_word = line[..start].split_whitespace().last().unwrap_or("");
                    let args = vec![cmd_name, prefix, prev_word];

                    let comp_point = pos.to_string();
                    let envs = [("COMP_LINE", line), ("COMP_POINT", comp_point.as_str())];

                    let output = ext_command_output(path_buf, &args, &envs)?;
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    let candidates = stdout
                        .lines()
                        .map(|line| Pair {
                            display: line.to_string(),
                            replacement: line.to_string() + " ",
                        })
                        .collect();

                    return Ok((start, candidates));
                }
            };

            let (start, candidates) = match self.file_completer.complete(line, pos, ctx) {
                Ok(candidates) => candidates,
                err @ Err(_) => return err,
            };

            let candidates = candidates
                .into_iter()
                .map(|pair| {
                    let sufix = if pair.replacement.ends_with("/") {
                        ""
                    } else {
                        " "
                    };

                    Pair {
                        // this workaround is required because the returned display string doesn't
                        // contain the slash when is a directory
                        display: pair.replacement.clone(),
                        replacement: pair.replacement + sufix,
                    }
                })
                .collect();

            return Ok((start, candidates));
        }

        let candidates = self.find_candidates(prefix);
        Ok((start, candidates))
    }
}

impl Highlighter for ShellHelper {}
impl Validator for ShellHelper {}
impl Hinter for ShellHelper {
    type Hint = String;
}
impl Helper for ShellHelper {}
