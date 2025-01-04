#[allow(unused_imports)]
use std::io::{self, Write};

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
    if line.starts_with("echo ") {
        let (_, output) = line
            .split_once("echo ")
            .expect("Already performed a starts_with check");
        println!("{}", output);
        return Ok(false);
    }
    match line {
        "exit 0" => Ok(true),
        _ => Err(format!("{}: command not found\n", line)),
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
