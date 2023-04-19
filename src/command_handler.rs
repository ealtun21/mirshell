use super::EDITOR;
use rustyline::error::ReadlineError;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::process::Command;
use std::{env, process::Stdio};
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
    let (commands, redirects) = parse_command(line.trim());
    match commands[0].0.as_str() {
        "cd" => change_directory(&commands[0].1),
        _ => execute_command(commands, redirects),
    }
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

fn parse_command(line: &str) -> (Vec<(String, Vec<String>)>, Vec<(String, String)>) {
    let mut tokens = line
        .split_whitespace()
        .map(String::from)
        .collect::<Vec<String>>();
    let mut command_parts = vec![];
    let mut redirects = vec![];

    let mut current_command = vec![];

    while !tokens.is_empty() {
        let token = tokens.remove(0);
        match token.as_str() {
            "|" => {
                if !current_command.is_empty() {
                    command_parts.push(current_command);
                    current_command = vec![];
                }
            }
            ">" | ">>" | "<" => {
                if let Some(target) = tokens.get(0).cloned() {
                    redirects.push((token.clone(), target.clone()));
                    tokens.remove(0);
                }
            }
            _ => current_command.push(token),
        }
    }

    if !current_command.is_empty() {
        command_parts.push(current_command);
    }

    (
        command_parts
            .into_iter()
            .map(|parts| {
                (
                    parts.first().cloned().unwrap_or_else(|| String::from("")),
                    parts[1..].to_vec(),
                )
            })
            .collect(),
        redirects,
    )
}

fn execute_command(commands: Vec<(String, Vec<String>)>, redirects: Vec<(String, String)>) {
    let mut prev_stdout = None;
    for (i, (command, args)) in commands.iter().enumerate() {
        let mut cmd = Command::new(&command);
        cmd.args(args);

        if let Some(stdout) = prev_stdout.take() {
            cmd.stdin(Stdio::from(stdout));
        }

        if i != commands.len() - 1 {
            cmd.stdout(Stdio::piped());
        }

        for (op, target) in &redirects {
            match op.as_str() {
                ">" => {
                    let file = File::create(target).expect("Failed to create file");
                    cmd.stdout(Stdio::from(file));
                }
                ">>" => {
                    let file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(target)
                        .expect("Failed to open file");
                    cmd.stdout(Stdio::from(file));
                }
                "<" => {
                    let file = File::open(target).expect("Failed to open file");
                    cmd.stdin(Stdio::from(file));
                }
                _ => {}
            }
        }

        let child = cmd.spawn();

        match child {
            Ok(mut child) => {
                if i != commands.len() - 1 {
                    prev_stdout = child.stdout.take();
                }
                let _ = child.wait();
            }
            Err(e) => eprintln!("Error: {}", e),
        }
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
