use super::EDITOR;
use rustyline::error::ReadlineError;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use termion::{color, terminal_size};

pub fn read_input() -> Result<String, ReadlineError> {
    let current_path = env::current_dir().unwrap_or_default();
    let shortened_path = shorten_path(current_path.to_str().unwrap_or(""));
    let (width, _) = terminal_size().unwrap_or((0, 0));

    let prompt = format!(
        "{}╭─{}{}{}@{}{}{}:{}\n{}╰─{}{} ",
        color::Fg(color::LightBlue),
        color::Fg(color::Green),
        whoami::username(),
        color::Fg(color::Black),
        color::Fg(color::Yellow),
        whoami::hostname(),
        color::Fg(color::LightBlack),
        if current_path.to_str().unwrap_or("").len() < (width / 2).into() {
            current_path.to_str().unwrap_or("")
        } else {
            shortened_path.as_str()
        },
        color::Fg(color::LightBlue),
        color::Fg(color::Reset),
        if whoami::username() == "root" {
            "#"
        } else {
            "$"
        }
    );

    EDITOR.lock().unwrap().readline(&prompt)
}

fn shorten_path(path: &str) -> String {
    let components: Vec<&str> = path.split('/').collect();
    components
        .iter()
        .enumerate()
        .map(|(i, &part)| {
            if i == 0 || i == components.len() - 1 {
                part.to_string()
            } else {
                part.chars().next().unwrap_or('?').to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("/")
}

pub fn handle_command(line: String) {
    let (command, args) = parse_command(line.trim());
    match command.as_str() {
        "cd" => change_directory(&args),
        _ => execute_command(&command, &args),
    }
}

fn parse_command(line: &str) -> (String, Vec<String>) {
    let mut parts = line.split_whitespace().map(String::from);
    let command = parts.next().unwrap_or_default();
    let args = parts.collect::<Vec<String>>();
    (command, args)
}

fn change_directory(args: &[String]) {
    let target_dir = if let Some(dir) = args.first() {
        PathBuf::from(dir)
    } else {
        env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
    };

    if let Err(e) = env::set_current_dir(&target_dir) {
        eprintln!("Error: {}", e);
    }
}

fn execute_command(command: &str, args: &[String]) {
    let child = Command::new(command).args(args).spawn();

    match child {
        Ok(mut child) => {
            let _ = child.wait();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub fn handle_error(err: ReadlineError) {
    match err {
        ReadlineError::Eof => {
            std::process::exit(0);
        }
        ReadlineError::Interrupted => (),
        _ => {
            eprintln!("Error: {:?}", err);
            std::process::exit(1);
        }
    }
}
