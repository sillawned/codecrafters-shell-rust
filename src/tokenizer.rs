// https://dustinknopoff.dev/articles/minishell/
pub fn tokenize(input: &str) -> Vec<String> {
    let input = input.trim().to_string();
    unquote(input)
}

fn unquote(input: String) -> Vec<String> {
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut was_escaped = false;

    let mut token = String::new();
    let mut tokens = Vec::new();

    for (position, c) in input.chars().enumerate() {
        match c {
            ' ' => {
                if was_escaped {
                    was_escaped = false;
                    continue;
                }
                if in_double_quote || in_single_quote {
                    token.push(c);
                } else {
                    if !token.is_empty() {
                        tokens.push(token.clone());
                        token.clear();
                    }
                }
            }
            '\\' => {
                // Enclosed by double quotes, the backslash retains its special meaning when followed by "$", "`", """, "\", or newline
                if was_escaped {
                    was_escaped = false;
                    continue;
                }
                if in_double_quote {
                    if let Some(next_char) = input.chars().nth(position + 1) {
                        if next_char == '$'
                            || next_char == '`'
                            || next_char == '"'
                            || next_char == '\\'
                            || next_char == '\n'
                        {
                            token.push(next_char);
                            was_escaped = true;
                        } else {
                            token.push(c);
                        }
                    }
                } else if in_single_quote {
                    token.push(c);
                } else {
                    if let Some(next_char) = input.chars().nth(position + 1) {
                        if next_char == '\n' {
                            continue;
                        } else {
                            token.push(next_char);
                            was_escaped = true;
                        }
                    }
                }
            }
            '"' => {
                if was_escaped {
                    was_escaped = !was_escaped;
                    continue;
                }
                if in_single_quote {
                    token.push(c);
                } else {
                    in_double_quote = !in_double_quote;
                }
            }
            '\'' => {
                if was_escaped {
                    was_escaped = !was_escaped;
                    continue;
                }
                if in_double_quote {
                    token.push(c);
                } else {
                    in_single_quote = !in_single_quote;
                }
            }
            _ => {
                if was_escaped {
                    was_escaped = false;
                    continue;
                }
                token.push(c);
            }
        }
    }
    // The last word may not be followed by a space.
    if !token.is_empty() {
        tokens.push(token.clone());
        token.clear();
    }

    tokens
}

