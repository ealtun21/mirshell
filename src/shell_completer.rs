use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult};
use rustyline::{validate, Helper};

pub struct ShellCompleter {
    pub filename_completer: FilenameCompleter,
}

impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.filename_completer.complete(line, pos, ctx)
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
