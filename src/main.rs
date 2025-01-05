#[allow(unused_imports)]
use std::io::{self, Write};
use std::{collections::HashMap, env, os::unix::fs::PermissionsExt, path::Path};

fn main() -> Result<(), String> {
    let valid_bins = get_available_binaries();
    loop {
        let line = readline()?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match respond(line, &valid_bins) {
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

fn get_available_binaries() -> HashMap<String, String> {
    let path = env::var("PATH").unwrap_or("".to_string());
    let mut pathes: Vec<&Path> = path.split(":").map(|p| Path::new(p)).collect();
    pathes.reverse();
    let mut valid_bins: HashMap<String, String> = HashMap::new();
    for p in pathes {
        for f in p.read_dir().unwrap() {
            match f {
                Ok(a) => {
                    let permissions = a.metadata().unwrap().permissions();
                    let is_executable = permissions.mode() & 0o111 != 0;
                    if is_executable {
                        let path = format!("{}", a.path().display());
                        valid_bins.insert(a.file_name().into_string().unwrap(), path);
                    }
                }
                Err(_) => todo!(),
            }
        }
    }
    valid_bins
}

fn respond(line: &str, valid_bins: &HashMap<String, String>) -> Result<bool, String> {
    let mut parsed_args: Vec<&str> = Vec::new();
    let mut command = line;
    if let Some((c, args)) = line.split_once(" ") {
        parsed_args = args.split(' ').collect::<Vec<&str>>();
        command = c;
    }
    match command {
        "echo" => {
            println!("{}", parsed_args.join(" "));
            return Ok(false);
        }
        "type" => match parsed_args[0] {
            "echo" | "type" | "exit" => {
                println!("{} is a shell builtin", parsed_args[0]);
                return Ok(false);
            }
            _ => {
                if let Some(p) = valid_bins.get(&parsed_args[0].to_string()) {
                    println!("{} is {}", parsed_args[0], p);
                    return Ok(false);
                }
                println!("{}: not found", parsed_args[0]);
                return Ok(false);
            }
        },
        "exit" => match parsed_args[0] {
            "0" => return Ok(true),
            _ => return Err(format!("{}: command not found\n", parsed_args[0])),
        },
        _ => return Err(format!("{}: command not found\n", command)),
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
