pub mod builtins;

enum LexerState {
    Normal,
    InSingleQuote,
    InDoubleQuote,
}

pub fn lex(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut state = LexerState::Normal;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match state {
            LexerState::Normal => {
                match ch {
                    ' ' | '\t' => {
                        if !current_token.is_empty() {
                            tokens.push(current_token.clone());
                            current_token.clear();
                        }
                    }
                    '\'' => state = LexerState::InSingleQuote,
                    '"' => state = LexerState::InDoubleQuote,
                    '\\' => {
                        if let Some(&next) = chars.peek() {
                            // Simple escape: add next character literally.
                            current_token.push(next);
                            chars.next();
                        }
                    }
                    _ => current_token.push(ch),
                }
            }
            LexerState::InSingleQuote => {
                if ch == '\'' {
                    state = LexerState::Normal;
                } else {
                    current_token.push(ch);
                }
            }
            LexerState::InDoubleQuote => {
                match ch {
                    '"' => state = LexerState::Normal,
                    '\\' => {
                        if let Some(&next) = chars.peek() {
                            // In double quotes, only escape certain characters.
                            if "\"$`\\".contains(next) {
                                current_token.push(next);
                                chars.next();
                            } else {
                                current_token.push(ch);
                            }
                        }
                    }
                    _ => current_token.push(ch),
                }
            }
        }
    }

    if !matches!(state, LexerState::Normal) {
        return Err("Unterminated quote detected".into());
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("echo script     shell", vec!["echo", "script", "shell"])]
    #[case("echo 'world     hello' 'shell''example'", vec!["echo", "world     hello", "shellexample"])]
    #[case("echo \"world     hello\" \"shell\"\"example\"", vec!["echo", "world     hello", "shellexample"])]
    #[case("cd /tmp/blueberry/pineapple/apple", vec!["cd", "/tmp/blueberry/pineapple/apple"])]
    #[case("pwd", vec!["pwd"])]
    #[case("cd ~", vec!["cd", "~"])]
    #[case("cd ./raspberry/raspberry", vec!["cd","./raspberry/raspberry"])]
    #[case("cd /non-existing-directory", vec!["cd","/non-existing-directory"])]
    #[case("custom_exe_6510 Alice", vec!["custom_exe_6510","Alice"])]
    #[case(r#"echo "before\   after""#, vec!["echo", r#"before\   after"#])]
    #[case(r#"echo world\ \ \ \ \ \ test"#, vec!["echo", r#"world      test"#])]
    fn test_cmd_arg_parsing(
        #[case] input: &str,
        #[case] expected: Vec<&str>,
    ) -> Result<(), String> {
        let args = lex(input)?;
        assert_eq!(args, expected);
        Ok(())
    }
}
