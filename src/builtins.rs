use std::{
    env,
    path::{Path, PathBuf},
};

pub static BUILTIN: &[&str] = &["pwd", "type", "echo", "exit", "cd"];

pub fn pwd() -> Result<bool, String> {
    if let Ok(p) = env::current_dir() {
        println!("{}", p.display());
        return Ok(false);
    }
    Err("error in pwd".to_string())
}

pub fn type_cmd(args: &[String]) -> Result<bool, String> {
    if args.is_empty() {
        return Err("type requires an argument\n".to_string());
    }

    if BUILTIN.contains(&&args[0].as_ref()) {
        println!("{} is a shell builtin", args[0]);
        return Ok(false);
    }

    if let Some(p) = find_command_exe(&args[0]) {
        println!("{} is {}", args[0], p.display());
        return Ok(false);
    }
    println!("{}: not found", args[0]);
    Ok(false)
}

pub fn echo(args: &[String]) -> Result<bool, String> {
    println!("{}", args.join(" "));
    Ok(false)
}

pub fn exit(args: &[String]) -> Result<bool, String> {
    if args.len() != 1 {
        return Err("exit requires a single argument\n".to_string());
    }
    if args[0] == "0" {
        return Ok(true);
    }
    Err("Unknown exit code received in arg\n".to_string())
}

pub fn cd(args: &[String]) -> Result<bool, String> {
    if args.is_empty() {
        return Ok(false);
    }
    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => "unset".to_string(),
    };
    let p = match args[0].as_ref() {
        "~" => {
            if home == "unset" {
                return Err("$HOME is not set".to_string());
            }
            Path::new(&home)
        }
        _ => Path::new(&args[0]),
    };
    match env::set_current_dir(p) {
        Ok(_) => Ok(false),
        Err(_) => {
            println!("cd: {}: No such file or directory", args[0]);
            Ok(false)
        }
    }
}

pub fn find_command_exe(name: &String) -> Option<PathBuf> {
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
