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
    Space,                     // Space character
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum QuoteState {
    None,
    Single,
    Double,
}

fn tokenize_word(chars: &mut std::iter::Peekable<std::str::Chars>) -> TokenType {
    let mut word = String::new();
    
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '>' | '<' | '|' | '&' | ';' | '"' | '\'' => break,
            '\\' => {
                chars.next(); // consume backslash
                if let Some(&escaped_char) = chars.peek() {
                    word.push(escaped_char);
                    chars.next();
                }
            }
            _ => {
                word.push(c);
                chars.next();
            }
        }
    }
    TokenType::Word(word)
}

pub fn tokenize(input: &str) -> Vec<TokenType> {
    let mut tokens = Vec::new();
    let mut chars = input.trim().chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
                continue;
            }
            '"' | '\'' => {
                tokens.push(tokenize_quoted_string(&mut chars));
            }
            '>' | '<' => {
                let token = tokenize_redirection(&mut chars);
                tokens.push(token);
            }
            '|' => {
                tokens.push(tokenize_pipe(&mut chars));
            }
            '&' => {
                tokens.push(tokenize_background(&mut chars));
            }
            '$' => {
                tokens.push(tokenize_dollar(&mut chars));
            }
            _ => {
                tokens.push(tokenize_word(&mut chars));
            }
        }
    }
    tokens
}

fn tokenize_quoted_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> TokenType {
    let mut quoted_string = String::new();
    let quote_char = chars.next().unwrap();
    
    while let Some(&c) = chars.peek() {
        if c == quote_char {
            chars.next(); // consume closing quote
            break;
        } else if c == '\\' && quote_char == '"' {
            chars.next(); // consume backslash
            if let Some(&escaped_char) = chars.peek() {
                quoted_string.push(escaped_char);
                chars.next();
            }
        } else {
            quoted_string.push(c);
            chars.next();
        }
    }
    
    if quote_char == '\'' {
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
    let op = chars.next().unwrap();
    let mut operator = op.to_string();
    
    if op == '>' && chars.peek() == Some(&'>') {
        chars.next();
        operator.push('>');
    }
    
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