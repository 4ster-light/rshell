use colored::Colorize;
use std::env;
use std::io::{self, Result, Write};
use std::path::PathBuf;
use std::process::Command;

struct Shell;

impl Shell {
    fn read_input() -> Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    fn parse_input(input: &str) -> Option<(&str, Vec<&str>)> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            None
        } else {
            let command = parts[0];
            let args = parts[1..].to_vec();
            Some((command, args))
        }
    }

    fn print_prompt() -> Result<()> {
        let user_os = env::var_os("USER");
        let user_part = user_os
            .as_ref()
            .map(|u| u.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let current_dir = env::current_dir()?;
        let home_dir = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();

        let display_dir = current_dir
            .strip_prefix(&home_dir)
            .map(|p| {
                if p.as_os_str().is_empty() {
                    "~".into()
                } else {
                    format!("~/{}", p.to_string_lossy())
                }
            })
            .unwrap_or_else(|_| current_dir.to_string_lossy().to_string());

        let prompt = format!("rshell @ {}:{}> ", user_part.yellow(), display_dir.blue())
            .bold()
            .green();

        print!("{}", prompt);
        io::stdout().flush()?;
        Ok(())
    }
}

fn handle_cd(dir: &str) {
    let path = match dir {
        "~" => env::var_os("HOME").map(PathBuf::from),
        ".." => Some(PathBuf::from("..")),
        _ => Some(PathBuf::from(dir)),
    };

    match path {
        Some(p) => {
            if let Err(e) = env::set_current_dir(&p) {
                if dir == "~" && env::var_os("HOME").is_none() {
                    eprintln!("cd: HOME directory not set");
                } else {
                    eprintln!("cd: {}: {}", dir, e);
                }
            }
        }
        None => eprintln!("cd: HOME directory not set"),
    }
}

fn execute_command(command: &str, args: &[&str]) -> Result<bool> {
    match command {
        "" => Ok(true),
        "exit" => Ok(false),
        "cd" => {
            let dir = args.get(0).copied().unwrap_or_else(|| "~");
            handle_cd(dir);
            Ok(true)
        }
        ".." | "~" => {
            handle_cd(command);
            Ok(true)
        }
        _ => {
            match Command::new(command).args(args).status() {
                Ok(status) => {
                    if !status.success() {
                        eprintln!(
                            "{}",
                            format!("Command exited with status: {}\n", status).yellow()
                        );
                    }
                }
                Err(e) => eprintln!("{}", format!("Error executing command: {}", e).red()),
            }
            Ok(true)
        }
    }
}

fn main() -> Result<()> {
    loop {
        if let Err(e) = Shell::print_prompt() {
            eprintln!("Error printing prompt: {}", e);
            continue;
        }

        let input = Shell::read_input()?;
        if input.is_empty() {
            continue;
        }

        let (command, args) = match Shell::parse_input(&input) {
            Some(data) => data,
            None => continue,
        };

        match execute_command(command, &args) {
            Ok(should_continue) => {
                if !should_continue {
                    break Ok(());
                }
            }
            Err(e) => {
                eprintln!("Shell execution error: {}", e);
            }
        }
    }
}
