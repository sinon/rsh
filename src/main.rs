#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, path::PathBuf, process::Command};

fn main() -> Result<(), String> {
    loop {
        let line = readline()?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match respond(line) {
            Ok(quit) => {
                if quit {
                    std::process::exit(0);
                }
            }
            Err(err) => {
                write!(std::io::stdout(), "{err}").map_err(|e| e.to_string())?;
                std::io::stdout().flush().map_err(|e| e.to_string())?;
            }
        }
    }
}

fn find_command_exe(name: &str) -> Option<PathBuf> {
    if let Ok(paths) = env::var("PATH") {
        for path in env::split_paths(&paths) {
            let command_path = path.join(name);

            if command_path.is_file() {
                return Some(command_path);
            }
        }
    }
    None
}

fn respond(line: &str) -> Result<bool, String> {
    let cmds: Vec<_> = line.split_whitespace().collect();
    let command = cmds[0];
    let args = &cmds[1..];
    match command {
        "echo" => {
            println!("{}", args.join(" "));
            return Ok(false);
        }
        "type" => match args[0] {
            "echo" | "type" | "exit" => {
                println!("{} is a shell builtin", args[0]);
                return Ok(false);
            }
            _ => {
                if let Some(p) = find_command_exe(args[0]) {
                    println!("{} is {}", args[0], p.display());
                    return Ok(false);
                }
                println!("{}: not found", args[0]);
                return Ok(false);
            }
        },
        "exit" => match args[0] {
            "0" => return Ok(true),
            _ => return Err(format!("{}: command not found\n", args[0])),
        },
        _ => {
            if let Some(p) = find_command_exe(command) {
                Command::new(p)
                    .args(args)
                    .status()
                    .expect("failed to execute command");
                return Ok(false);
            }
            return Err(format!("{}: command not found\n", command));
        }
    }
}

fn readline() -> Result<String, String> {
    write!(std::io::stdout(), "$ ").map_err(|e| e.to_string())?;
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| e.to_string())?;
    Ok(buffer)
}
