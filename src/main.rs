use std::process::Command;
use std::io::Result;

struct Shell;
impl Shell {
    fn read_input() -> Result<String> {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    fn get_command(input: &str) -> (&str, Vec<&str>) {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        let command = parts[0];
        let args = parts[1..].to_vec();
        (command, args)
    }
}

fn main() -> Result<()> {
    loop {
        print!("rshell> ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let input = Shell::read_input()?;
        match input.as_str() {
            input if input.is_empty() => continue,
            "exit" => break Ok(()),
            ".." => {
                if let Err(e) = std::env::set_current_dir("..") {
                    eprintln!("cd: ..: {}", e);
                }
            }
            "~" => {
                if let Some(home) = std::env::var_os("HOME") {
                    if let Err(e) = std::env::set_current_dir(home) {
                        eprintln!("cd: ~: {}", e);
                    }
                } else {
                    eprintln!("cd: HOME directory not set");
                }
            }
            _ => {
                let (command, args) = Shell::get_command(&input);
                match command {
                    "cd" => {
                        let dir = Shell::get_command(&input).1[0];
                        if let Err(e) = std::env::set_current_dir(dir) {
                            eprintln!("cd: {}: {}", dir, e);
                        }
                    }
                    _ => match Command::new(command).args(args).status() {
                        Ok(status) => {
                            if !status.success() {
                                eprintln!("Command exited with status: {}", status);
                            }
                        }
                        Err(e) => eprintln!("Error executing command: {}", e),
                    },
                }
            }
        }
    }
}
