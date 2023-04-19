mod command_handler;
mod shell_completer;

use command_handler::{handle_command, handle_error, read_input};
use lazy_static::lazy_static;
use rustyline::completion::FilenameCompleter;
use rustyline::{CompletionType, Config, EditMode, Editor};
use shell_completer::ShellCompleter;
use std::sync::Mutex;

lazy_static! {
    static ref EDITOR: Mutex<Editor<ShellCompleter>> = {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();
        let completer = ShellCompleter {
            filename_completer: FilenameCompleter::new(),
        };
        let mut editor = Editor::<ShellCompleter>::with_config(config);
        editor.set_helper(Some(completer));
        Mutex::new(editor)
    };
}

fn main() {
    loop {
        let readline = read_input();
        match readline {
            Ok(line) => {
                if !line.trim().is_empty() {
                    handle_command(line);
                }
            }
            Err(err) => handle_error(err),
        }
    }
}
