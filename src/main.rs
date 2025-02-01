use std::io::Write;
use std::process::Command;

use rsh::builtins::{cd, echo, exit, find_command_exe, pwd, type_cmd, BUILTIN};
use rsh::lex;

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

fn respond(line: &str) -> Result<bool, String> {
    let mut lexed_args = lex(line)?;
    let command = lexed_args[0].clone();
    let args = lexed_args.split_off(1);
    if BUILTIN.contains(&command.as_ref()) {
        match command.as_ref() {
            "echo" => return Ok(echo(&args)),
            "type" => return type_cmd(&args),
            "exit" => return exit(&args),
            "pwd" => return pwd(),
            "cd" => return cd(&args),
            _ => unreachable!(),
        }
    }

    if let Some(p) = find_command_exe(&command) {
        if let Some(name) = p.file_name() {
            Command::new(name)
                .args(&args)
                .status()
                .expect("failed to execute command");
        }
        return Ok(false);
    }
    Err(format!("{command}: command not found\n"))
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
