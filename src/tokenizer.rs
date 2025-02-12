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
    Comment(String),            // Comment starting with #
    Assignment(String, String), // Variable assignment, e.g., VAR=value
    Newline,                   // Newline character
    SingleQuotedString(String), // Single-quoted strings
    DoubleQuotedString(String), // Double-quoted strings
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum QuoteState {
    None,
    Single,
    Double,
}

pub fn tokenize(input: &str) -> Vec<TokenType> {
    let input = input.trim();
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '\\' => {
                chars.next();
                if let Some(escaped_char) = chars.next() {
                    tokens.push(TokenType::Word(escaped_char.to_string()));
                }
            }
            '\'' | '"' => {
                tokens.push(tokenize_quoted_string(&mut chars));
            }
            ' ' | '\t' => {
                chars.next();
            }
            '|' => {
                tokens.push(tokenize_pipe(&mut chars));
            }
            '&' => {
                tokens.push(tokenize_background(&mut chars));
            }
            ';' => {
                chars.next();
                tokens.push(TokenType::Semicolon);
            }
            '>' | '<' => {
                tokens.push(tokenize_redirection(&mut chars));
            }
            '(' => {
                chars.next();
                tokens.push(TokenType::LeftParen);
            }
            ')' => {
                chars.next();
                tokens.push(TokenType::RightParen);
            }
            '$' => {
                tokens.push(tokenize_dollar(&mut chars));
            }
            '#' => {
                tokens.push(tokenize_comment(&mut chars));
                break;
            }
            '=' => {
                tokens.push(tokenize_assignment(&mut chars));
            }
            '\n' => {
                chars.next();
                tokens.push(TokenType::Newline);
            }
            _ => {
                if c.is_digit(10) {
                    tokens.push(tokenize_file_descriptor(&mut chars));
                } else {
                    tokens.push(tokenize_word(&mut chars));
                }
            }
        }
    }

    tokens
}

fn tokenize_quoted_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> TokenType {
    let mut quoted_string = String::new();
    let quote_char = chars.next().unwrap();
    let quote_state = if quote_char == '\'' { QuoteState::Single } else { QuoteState::Double };

    while let Some(&c) = chars.peek() {
        match (quote_state, c) {
            (QuoteState::Single, '\'') => {
                chars.next(); // Consumte the closing single quote
                break;
            }
            (QuoteState::Single, _) => {
                quoted_string.push(c);
                chars.next(); // Consume the character inside the single quotes
            }
            (QuoteState::Double, '"') => {
                chars.next(); // Consumte the closing double quote
                break;
            }
            (QuoteState::Double, '\\') => {
                chars.next(); // Consume the backslash
                if let Some(&escaped_char) = chars.peek() {
                    match escaped_char {
                        '\\' | '"' | '$' | '`' | '\n' => {
                            quoted_string.push(escaped_char);
                            chars.next(); // Consume the escaped character
                        }
                        _ => {
                            quoted_string.push('\\');
                            quoted_string.push(escaped_char);
                            chars.next(); // Consume the escaped character
                        }
                    }
                }
            }
            (QuoteState::Double, _) => {
                quoted_string.push(c);
                chars.next();
            }
            _ => unreachable!()
        }
    }

    if quote_state == QuoteState::Single {
        TokenType::SingleQuotedString(quoted_string)
    } else {
        TokenType::DoubleQuotedString(quoted_string)
    }
}

fn tokenize_pipe(chars: &mut std::iter::Peekable<std::str::Chars>) -> TokenType {
    chars.next(); // Consume the "|"
    if let Some(&next_c) = chars.peek() {
        if next_c == '|' {
            chars.next(); // Consume the next "|"
            TokenType::LogicalOr
        } else {
            TokenType::Pipe
        }
    } else {
        TokenType::Pipe
    }
}

fn tokenize_background(chars: &mut std::iter::Peekable<std::str::Chars>) -> TokenType {
    chars.next(); // Consume the "&"
    if let Some(&next_c) = chars.peek() {
        if next_c == '&' {
            chars.next(); // Consume the next "&"
            TokenType::LogicalAnd
        } else {
            TokenType::Background
        }
    } else {
        TokenType::Background
    }
}

fn tokenize_redirection(chars: &mut std::iter::Peekable<std::str::Chars>) -> TokenType {
    let operator = chars.next().unwrap();
    let operator = if operator == '>' && chars.peek() == Some(&'>') {
        chars.next(); // Consume the next ">"
        ">>".to_string()
    } else {
        operator.to_string()
    };
    TokenType::RedirectionOperator(operator)
}

fn tokenize_dollar(chars: &mut std::iter::Peekable<std::str::Chars>) -> TokenType {
    chars.next(); // Consume the "$"
    if chars.peek() == Some(&'(') {
        chars.next(); // Consume the "("
        let mut cmd = String::new();
        while let Some(&c) = chars.peek() {
            if c == ')' {
                break;
            }
            cmd.push(c);
            chars.next();
        }
        chars.next(); // Consume the closing ")"
        TokenType::CommandSubstitution(cmd)
    } else {
        let mut var = String::new();
        while let Some(&c) = chars.peek() {
            if !c.is_alphanumeric() && c != '_' {
                break;
            }
            var.push(c);
            chars.next();
        }
        TokenType::DollarVar(var)
    }
}

fn tokenize_comment(chars: &mut std::iter::Peekable<std::str::Chars>) -> TokenType {
    chars.next(); // Consume the "#"
    let comment: String = chars.collect();
    TokenType::Comment(comment)
}

fn tokenize_assignment(chars: &mut std::iter::Peekable<std::str::Chars>) -> TokenType {
    let mut var_name = String::new();
    while let Some(&c) = chars.peek() {
        if c == '=' {
            chars.next(); // Consume the "="
            break;
        }
        var_name.push(c);
        chars.next();
    }
    let mut value = String::new();
    while let Some(&c) = chars.peek() {
        if c == ' ' || c == '\t' || c == '\n' {
            break;
        }
        value.push(c);
        chars.next();
    }
    TokenType::Assignment(var_name, value)
}

fn tokenize_file_descriptor(chars: &mut std::iter::Peekable<std::str::Chars>) -> TokenType {
    let mut fd = String::new();
    while let Some(&c) = chars.peek() {
        if !c.is_digit(10) {
            break;
        }
        fd.push(c);
        chars.next();
    }
    TokenType::FileDescriptor(fd.parse().unwrap())
}

fn tokenize_word(chars: &mut std::iter::Peekable<std::str::Chars>) -> TokenType {
    let mut word = String::new();
    let mut quote_state = QuoteState::None;

    while let Some(&c) = chars.peek() {
        match (quote_state, c) {
            (QuoteState::None, '\\') => {
                chars.next();
                if let Some(&escaped_char) = chars.peek() {
                    word.push(escaped_char);
                    chars.next();
                }
            }
            (QuoteState::None, ' ' | '\t' | '\n' | '|' | '&' | ';' | '>' | '<' | '(' | ')' | '$' | '#' | '=') => {
                break;
            }
            (QuoteState::None, '\'') => {
                quote_state = QuoteState::Single;
                chars.next();
            }
            (QuoteState::None, '"') => {
                quote_state = QuoteState::Double;
                chars.next();
            }
            (QuoteState::Single, '\'') => {
                quote_state = QuoteState::None;
                chars.next();
            }
            (QuoteState::Single, _) => {
                word.push(c);
                chars.next();
            }
            (QuoteState::Double, '"') => {
                quote_state = QuoteState::None;
                chars.next();
            }
            (QuoteState::Double, '\\') => {
                chars.next(); // Consume the backslash
                if let Some(&escaped_char) = chars.peek() {
                    match escaped_char {
                        '\\' | '"' | '$' | '`' | '\n' => {
                            word.push(escaped_char);
                            chars.next(); // Consume the escaped character
                        }
                        _ => {
                            word.push('\\');
                            word.push(escaped_char);
                            chars.next(); // Consume the escaped character
                        }
                    }
                }
            }
            (QuoteState::Double, _) => {
                word.push(c);
                chars.next();
            }
            _ => {
                word.push(c);
                chars.next();
            }
        }
    }
    TokenType::Word(word)
}