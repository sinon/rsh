use std::{
    env,
    path::{Path, PathBuf},
};

pub static BUILTIN: &[&str] = &["pwd", "type", "echo", "exit", "cd"];

/// # Errors
///
/// Will return `Err` if fails to retrieve the `current_dir`
pub fn pwd() -> Result<bool, String> {
    if let Ok(p) = env::current_dir() {
        println!("{}", p.display());
        return Ok(false);
    }
    Err("error in pwd".to_string())
}

/// # Errors
///
/// Will return `Err` if:
///     - not enough arguments passed
///     - no matching command is found
pub fn type_cmd(args: &[String]) -> Result<bool, String> {
    if args.is_empty() {
        return Err("type requires an argument\n".to_string());
    }

    if BUILTIN.contains(&args[0].as_ref()) {
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

#[must_use]
pub fn echo(args: &[String]) -> bool {
    println!("{}", args.join(" "));
    false
}

/// # Errors
///
/// Will return `Err` if:
///     - `args` is empty
///     - An supported exit code is passed in `args`
/// permission to read it.
pub fn exit(args: &[String]) -> Result<bool, String> {
    if args.len() != 1 {
        return Err("exit requires a single argument\n".to_string());
    }
    if args[0] == "0" {
        return Ok(true);
    }
    Err("Unknown exit code received in arg\n".to_string())
}

/// # Errors
///
/// Will return `Err` if:
///     - `$HOME` is not set
/// permission to read it.
pub fn cd(args: &[String]) -> Result<bool, String> {
    if args.is_empty() {
        return Ok(false);
    }
    let home = env::var("HOME").unwrap_or_else(|_| "unset".to_string());
    let p = match args[0].as_ref() {
        "~" => {
            if home == "unset" {
                return Err("$HOME is not set".to_string());
            }
            Path::new(&home)
        }
        _ => Path::new(&args[0]),
    };
    if !matches!(env::set_current_dir(p), Ok(())) {
        println!("cd: {}: No such file or directory", args[0]);
    }
    Ok(false)
}

#[must_use]
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
