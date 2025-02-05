use std::iter::Peekable;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Word(String),              // Command or argument
    RedirectionOperator(String), // >, >>, <, etc.
    FileDescriptor(i32),       // File descriptor (1, 2, etc.)
    Pipe,                      // Pipe symbol |
    Background,                // Background job symbol &
    LeftParen,                 // Opening parenthesis for subshells
    RightParen,                // Closing parenthesis for subshells
    Semicolon,                 // Command separator ;
    LogicalAnd,                // Logical AND (&&)
    LogicalOr,                 // Logical OR (||)
    DollarVar(String),         // Variable expansion, e.g., $HOME
    CommandSubstitution(String), // Command substitution $(command)
    Quote(char),               // Single or double quotes
    Escape,                    // Escape character like \
    Newline,                   // Newline character
}

pub fn tokenize(input: &str) -> Vec<TokenType> {
    let input = input.trim();
    let mut tokens = Vec::new();
    let mut token = String::new();

    let mut in_quote = false;
    let mut quote_char = '\0';
    let mut escape_next = false;

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if escape_next {
            token.push(c);
            escape_next = false;
            continue;
        }

        match c {
            '\\' => {
                escape_next = true;
            }
            '\'' | '"' => {
                if in_quote {
                    if c == quote_char {
                        in_quote = false;
                        tokens.push(TokenType::Word(token.clone()));
                        token.clear();
                        tokens.push(TokenType::Quote(c));
                    } else {
                        token.push(c);
                    }
                } else {
                    in_quote = true;
                    quote_char = c;
                    tokens.push(TokenType::Quote(c));
                }
            }
            ' ' | '\t' if !in_quote => {
                if !token.is_empty() {
                    tokens.push(TokenType::Word(token.clone()));
                    token.clear();
                }
            }
            '|' if !in_quote => {
                if !token.is_empty() {
                    tokens.push(TokenType::Word(token.clone()));
                    token.clear();
                }
                if let Some(&next_c) = chars.peek() {
                    if next_c == '|' {
                        chars.next(); // Consume the next same character
                        tokens.push(TokenType::LogicalOr);
                    } else {
                        tokens.push(TokenType::Pipe);
                    }
                } else {
                    tokens.push(TokenType::Pipe);
                }
            }
            '&' if !in_quote => {
                if !token.is_empty() {
                    tokens.push(TokenType::Word(token.clone()));
                    token.clear();
                }
                if let Some(&next_c) = chars.peek() {
                    if next_c == '&' {
                        chars.next(); // Consume the next same character
                        tokens.push(TokenType::LogicalAnd);
                    } else {
                        tokens.push(TokenType::Background);
                    }
                } else {
                    tokens.push(TokenType::Background);
                }
            }
            ';' if !in_quote => {
                if !token.is_empty() {
                    tokens.push(TokenType::Word(token.clone()));
                    token.clear();
                }
                tokens.push(TokenType::Semicolon);
            }
            '>' | '<' if !in_quote => {
                if !token.is_empty() {
                    if let Ok(fd) = token.parse::<i32>() {
                        tokens.push(TokenType::FileDescriptor(fd));
                        token.clear();
                    } else {
                        tokens.push(TokenType::Word(token.clone()));
                        token.clear();
                    }
                }
                let operator = if c == '>' && chars.peek() == Some(&'>') {
                    chars.next(); // Consume the next '>'
                    ">>".to_string()
                } else {
                    c.to_string()
                };
                tokens.push(TokenType::RedirectionOperator(operator));
            }
            '(' if !in_quote => {
                if !token.is_empty() {
                    tokens.push(TokenType::Word(token.clone()));
                    token.clear();
                }
                tokens.push(TokenType::LeftParen);
            }
            ')' if !in_quote => {
                if !token.is_empty() {
                    tokens.push(TokenType::Word(token.clone()));
                    token.clear();
                }
                tokens.push(TokenType::RightParen);
            }
            '$' if !in_quote => {
                if !token.is_empty() {
                    tokens.push(TokenType::Word(token.clone()));
                    token.clear();
                }
                if chars.peek() == Some(&'(') {
                    chars.next();
                    let mut cmd = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == ')' {
                            break;
                        }
                        cmd.push(c);
                        chars.next();
                    }
                    chars.next(); // Consume the closing ')'
                    tokens.push(TokenType::CommandSubstitution(cmd));
                } else {
                    let mut var = String::new();
                    while let Some(&c) = chars.peek() {
                        if !c.is_alphanumeric() && c != '_' {
                            break;
                        }
                        var.push(c);
                        chars.next();
                    }
                    tokens.push(TokenType::DollarVar(var));
                }
            }
            '\n' => {
                if !token.is_empty() {
                    tokens.push(TokenType::Word(token.clone()));
                    token.clear();
                }
                tokens.push(TokenType::Newline);
            }
            _ => {
                token.push(c);
            }
        }
    }

    if !token.is_empty() {
        tokens.push(TokenType::Word(token));
    }

    tokens
}
