use rustyline::Helper;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use std::env;
use std::fs;

pub struct ShellHelper {
    builtins: &'static [&'static str],
    file_completer: FilenameCompleter,
}

impl ShellHelper {
    pub fn new(builtins: &'static [&'static str]) -> ShellHelper {
        ShellHelper {
            builtins,
            file_completer: FilenameCompleter::new(),
        }
    }

    fn find_candidates(&self, prefix: &str) -> Vec<Pair> {
        let mut candidates = vec![];

        for &candidate in self.builtins {
            if candidate.starts_with(prefix) {
                candidates.push(Pair {
                    display: candidate.to_string() + " ",
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
                        display: name.clone() + " ",
                        replacement: name + " ",
                    });
                }
            }
        }

        candidates.sort_by(|a, b| a.display.cmp(&b.display));
        candidates.dedup_by(|a, b| a.display == b.display);
        candidates
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
        let prefix = &line[..pos];

        let is_first_word = prefix.trim_start().find(' ').is_none();

        if !is_first_word {
            let (pos, pairs) = match self.file_completer.complete(line, pos, ctx) {
                Ok(candidates) => candidates,
                err @ Err(_) => return err,
            };

            let pairs = pairs
                .into_iter()
                .map(|pair| Pair {
                    display: pair.display + " ",
                    replacement: pair.replacement + " ",
                })
                .collect();

            return Ok((pos, pairs));
        }

        let candidates = self.find_candidates(prefix);
        Ok((0, candidates))
    }
}

impl Highlighter for ShellHelper {}
impl Validator for ShellHelper {}
impl Hinter for ShellHelper {
    type Hint = String;
}
impl Helper for ShellHelper {}
