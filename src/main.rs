use std::io::Write;
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

static BUILTIN: &[&str] = &["pwd", "type", "echo", "exit", "cd"];

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

fn pwd() -> Result<bool, String> {
    if let Ok(p) = env::current_dir() {
        println!("{}", p.display());
        return Ok(false);
    }
    Err("error in pwd".to_string())
}

fn echo(args: &[&str]) -> Result<bool, String> {
    println!("{}", args.join(" "));
    Ok(false)
}

fn type_cmd(args: &[&str]) -> Result<bool, String> {
    if args.is_empty() {
        return Err("type requires an argument\n".to_string());
    }

    if BUILTIN.contains(&args[0]) {
        println!("{} is a shell builtin", args[0]);
        return Ok(false);
    }

    if let Some(p) = find_command_exe(args[0]) {
        println!("{} is {}", args[0], p.display());
        return Ok(false);
    }
    println!("{}: not found", args[0]);
    Ok(false)
}

fn exit(args: &[&str]) -> Result<bool, String> {
    if args.len() != 1 {
        return Err("exit requires a single argument\n".to_string());
    }
    if args[0] == "0" {
        return Ok(true);
    }
    Err("Unknown exit code received in arg".to_string())
}

fn cd(args: &[&str]) -> Result<bool, String> {
    if args.is_empty() {
        return Ok(false);
    }
    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => "unset".to_string(),
    };
    let p = match args[0] {
        "~" => {
            if home == "unset" {
                return Err("$HOME is not set".to_string());
            }
            Path::new(&home)
        }
        _ => Path::new(args[0]),
    };
    match env::set_current_dir(p) {
        Ok(_) => Ok(false),
        Err(_) => {
            println!("cd: {}: No such file or directory", args[0]);
            Ok(false)
        }
    }
}

fn respond(line: &str) -> Result<bool, String> {
    let cmds: Vec<_> = line.split_whitespace().collect();
    let command = cmds[0];
    let args = &cmds[1..];
    if BUILTIN.contains(&command) {
        match command {
            "echo" => return echo(args),
            "type" => return type_cmd(args),
            "exit" => return exit(args),
            "pwd" => return pwd(),
            "cd" => return cd(args),
            _ => unreachable!(),
        }
    }

    if let Some(p) = find_command_exe(command) {
        Command::new(p)
            .args(args)
            .status()
            .expect("failed to execute command");
        return Ok(false);
    }
    Err(format!("{}: command not found\n", command))
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
