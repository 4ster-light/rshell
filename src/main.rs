use colored::Colorize;
use std::env;
use std::io::{self, Result, Write};
use std::path::PathBuf;
use std::process::Command;

struct Shell;

impl Shell {
    fn read_input() -> Result<String> {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => Ok(input.trim().to_string()),
            Err(e) => Err(e),
        }
    }

    fn parse_input(input: &str) -> Option<(&str, Vec<&str>)> {
        match input.split_whitespace().collect::<Vec<&str>>().as_slice() {
            [] => None,
            [command, args @ ..] => Some((command, args.to_vec())),
        }
    }

    fn print_prompt() -> Result<()> {
        let current_dir = env::current_dir()?;
        let home_dir = env::var_os("HOME").map(PathBuf::from);
        let user = env::var_os("USER")
            .as_ref()
            .map(|u| u.to_string_lossy().to_string());

        if user.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "USER environment variable not set",
            ));
        }

        if home_dir.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "HOME environment variable not set",
            ));
        }

        let display_dir = current_dir
            .strip_prefix(home_dir.unwrap())
            .map(|p| {
                if p.as_os_str().is_empty() {
                    "~".to_string()
                } else {
                    format!("~/{}", p.to_string_lossy())
                }
            })
            .unwrap_or(current_dir.to_string_lossy().to_string());

        let prompt = format!(
            "rshell @ {}:{}> ",
            user.unwrap().yellow(),
            display_dir.blue()
        )
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
        "cd" => match args.first().copied() {
            Some(dir) => {
                handle_cd(dir);
                Ok(true)
            }
            None => {
                eprintln!("{}", "cd: missing directory argument".red());
                Ok(true)
            }
        },
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

fn main() {
    loop {
        if let Err(e) = Shell::print_prompt() {
            eprintln!("{}", format!("Error printing prompt: {}", e).red());
            continue;
        }

        let input = match Shell::read_input() {
            Ok(line) => line,
            Err(e) => {
                eprintln!("{}", format!("Error reading input: {}", e).red());
                continue;
            }
        };

        let (command, args) = match Shell::parse_input(&input) {
            Some(data) => data,
            None => continue,
        };

        match execute_command(command, &args) {
            Ok(should_continue) => {
                if !should_continue {
                    break;
                }
            }
            Err(e) => eprintln!("{}", format!("Error executing command: {}", e).red()),
        }
    }
}
