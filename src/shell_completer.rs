use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult};
use rustyline::{validate, Helper};

use std::{env, fs};

pub struct ShellCompleter {
    pub filename_completer: FilenameCompleter,
    pub binary_completer: BinaryCompleter,
}

impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let (start, mut filename_candidates) = self.filename_completer.complete(line, pos, ctx)?;
        let (_start, mut binary_candidates) = self.binary_completer.complete(line, pos, ctx)?;

        filename_candidates.append(&mut binary_candidates);
        Ok((start, filename_candidates))
    }
}

impl Helper for ShellCompleter {}
impl Highlighter for ShellCompleter {}

impl Hinter for ShellCompleter {
    type Hint = String;
}

impl validate::Validator for ShellCompleter {
    fn validate(&self, _ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        Ok(ValidationResult::Valid(None))
    }
}

pub struct BinaryCompleter;

impl Completer for BinaryCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        complete_binaries(line).map(|binaries| (0, binaries))
    }
}

fn complete_binaries(line: &str) -> Result<Vec<Pair>, ReadlineError> {
    let mut result = vec![];

    let path_var = env::var("PATH").unwrap_or_else(|_| String::from(""));
    let paths: Vec<&str> = path_var.split(':').collect();

    for path in &paths {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error reading directory {}: {:?}", path, e);
                continue;
            }
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                if file_name.starts_with(line) {
                    result.push(Pair {
                        display: file_name.to_string(),
                        replacement: file_name.to_string(),
                    });
                }
            }
        }
    }

    Ok(result)
}
