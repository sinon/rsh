use std::io::Write;
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use regex::Regex;
use std::sync::LazyLock;

static SINGLE_QUOTE_GROUPS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"'([^']+)'|"([^"]+)"|([\w\d\/~.-]+)"#).unwrap());
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
    Err("Unknown exit code received in arg\n".to_string())
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

fn clean_commas(line: &str) -> String {
    line.replace("''", "").replace("\"\"", "")
}

fn parse_cmd_args(line: &str) -> Result<(&str, Vec<&str>), String> {
    let (command, args) = if let Some((command, rest)) = line.split_once(" ") {
        let split_rest: Vec<&str> = SINGLE_QUOTE_GROUPS
            .captures_iter(rest)
            .filter_map(|captures| {
                captures
                    .get(1)
                    .or_else(|| captures.get(2))
                    .or_else(|| captures.get(3))
                    .map(|m| m.as_str())
            })
            .collect();
        (command, split_rest)
    } else {
        (line, Vec::new())
    };
    Ok((command, args))
}

fn respond(line: &str) -> Result<bool, String> {
    let line = clean_commas(line);
    let (command, args) = parse_cmd_args(&line)?;
    if BUILTIN.contains(&command) {
        match command {
            "echo" => return echo(&args),
            "type" => return type_cmd(&args),
            "exit" => return exit(&args),
            "pwd" => return pwd(),
            "cd" => return cd(&args),
            _ => unreachable!(),
        }
    }

    if let Some(p) = find_command_exe(command) {
        if let Some(name) = p.file_name() {
            Command::new(name)
                .args(args)
                .status()
                .expect("failed to execute command");
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("echo script     shell", ("echo", vec!["script", "shell"]))]
    #[case("echo 'world     hello' 'shell''example'", ("echo", vec!["world     hello", "shellexample"]))]
    #[case("echo \"world     hello\" \"shell\"\"example\"", ("echo", vec!["world     hello", "shellexample"]))]
    #[case("cd /tmp/blueberry/pineapple/apple", ("cd", vec!["/tmp/blueberry/pineapple/apple"]))]
    #[case("pwd", ("pwd", vec![]))]
    #[case("cd ~", ("cd", vec!["~"]))]
    #[case("cd ./raspberry/raspberry", ("cd", vec!["./raspberry/raspberry"]))]
    #[case("cd /non-existing-directory", ("cd", vec!["/non-existing-directory"]))]
    #[case("custom_exe_6510 Alice", ("custom_exe_6510", vec!["Alice"]))]
    fn test_cmd_arg_parsing(
        #[case] input: &str,
        #[case] expected: (&str, Vec<&str>),
    ) -> Result<(), String> {
        let cleaned_input = clean_commas(input);
        let (cmd, args) = parse_cmd_args(&cleaned_input)?;
        assert_eq!(cmd, expected.0);
        assert_eq!(args, expected.1);
        Ok(())
    }

    #[rstest]
    #[case("cat", "/bin/cat")]
    fn test_find_command_exe(
        #[case] input: &str,
        #[case] expected_output: &str,
    ) -> Result<(), String> {
        let p = find_command_exe(input).unwrap();
        assert_eq!(p.display().to_string(), expected_output.to_string());
        Ok(())
    }
}
