use std::iter::Peekable;
use std::str::Chars;
use crate::{
    types::QuoteType,
    word::{Word, WordPart},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(Word), // Changed from String to Word
    Operator(Operator),
    Quote(QuoteType),
    Space,
    NewLine,
    CommandSubst(String), // Added for command substitution
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Pipe,           // |
    PipeAnd,        // |&
    And,            // &&
    Or,            // ||
    Background,     // &
    Semicolon,      // ;
    RedirectOut,    // >
    RedirectIn,     // <
    RedirectAppend, // >>
    RedirectError,  // 2>
    RedirectHereDoc, // <<
    RedirectHereStr, // <<<
    RedirectDup,     // >&
    RedirectErrorAppend, // 2>>
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    current: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars().peekable();
        let current = chars.next();
        Self { 
            input: chars,
            current,
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.consume_whitespace(); // Skip leading whitespace

        while let Some(c) = self.current {
            match c {
                '`' => { // Handle backtick for command substitution
                    self.advance(); // Consume the opening backtick
                    let command_string = self.read_until_backtick();
                    let mut word = Word::new();
                    // Create a WordPart::Simple for the command substitution string `...`
                    // The executor will handle the expansion of this.
                    word.add_part(WordPart::Simple(format!("`{}`", command_string)));
                    return Some(Token::Word(word)); 
                }
                ' ' | '\t' => {
                    self.advance();
                    if let Some(next_c) = self.current {
                        if !matches!(next_c, ' ' | '\t' | '\n') {
                            return Some(Token::Space);
                        }
                    }
                }
                '\n' => {
                    self.advance();
                    return Some(Token::NewLine);
                }
                '0'..='9' => {
                    // Peek ahead to see if this is a redirection
                    if let Some(&'>') = self.input.peek() {
                        return Some(self.read_operator());
                    }
                    return Some(Token::Word(self.read_word())); // read_word now returns Word
                }
                '|' | '&' | '>' | '<' | ';' => {
                    return Some(self.read_operator());
                }
                _ => {
                    return Some(Token::Word(self.read_word())); // read_word now returns Word
                }
            }
        }
        None
    }

    fn read_word(&mut self) -> Word { // Changed return type to Word
        let mut word = Word::new();
        let mut current_segment = String::new(); // Stores current unquoted, single-quoted, or double-quoted segment
        let mut current_quote_type = QuoteType::None;
        let mut escape_next = false;
        
        while let Some(c) = self.current {
            match current_quote_type {
                QuoteType::None => { // Currently not inside any quotes
                    match c {
                        '\\' if !escape_next => {
                            escape_next = true;
                            // For unquoted simple parts, backslash + char will be processed by Word::to_string or executor
                            // Store the backslash and the char it escapes
                            current_segment.push('\\'); 
                            self.advance(); // consume backslash, next char will be pushed if it exists
                            if let Some(escaped_char) = self.current { // Ensure there is a char to escape
                                current_segment.push(escaped_char);
                                self.advance(); // consume the escaped character
                                escape_next = false; // Reset escape_next as it has been processed
                            } else { // Trailing backslash
                                // escape_next remains true, current_segment has trailing \\
                                // This will be added as is to WordPart::Simple
                            }
                        }
                        '\'' => {
                            if !current_segment.is_empty() { word.add_part(WordPart::Simple(current_segment)); current_segment = String::new(); }
                            current_quote_type = QuoteType::Single;
                            self.advance(); // Consume opening single quote
                        }
                        '\"' => {
                            if !current_segment.is_empty() { word.add_part(WordPart::Simple(current_segment)); current_segment = String::new(); }
                            current_quote_type = QuoteType::Double;
                            self.advance(); // Consume opening double quote
                        }
                        ' ' | '\t' | '\n' | '|' | '&' | '>' | '<' | ';' | '`' => { // Added ` to break word on command sub start
                            break; // End of word
                        }
                        _ => {
                            current_segment.push(c);
                            self.advance();
                            escape_next = false; // Reset if it was true from a previous incomplete escape
                        }
                    }
                }
                QuoteType::Single => { // Inside single quotes
                    match c {
                        '\'' => { // End of single quotes
                            word.add_part(WordPart::SingleQuoted(current_segment));
                            current_segment = String::new();
                            current_quote_type = QuoteType::None; self.advance(); // Consume closing single quote
                        }
                        _ => {
                            current_segment.push(c);
                            self.advance();
                        }
                    }
                }
                QuoteType::Double => { // Inside double quotes
                    match c {
                        '\\' if !escape_next => { // Potential escape within double quotes
                            escape_next = true;
                            current_segment.push('\\'); // Preserve backslash for now, Word::to_string or executor will handle it
                            self.advance(); // Consume backslash, next char will be processed
                        }
                        '\"' if !escape_next => { // End of double quotes
                            word.add_part(WordPart::DoubleQuoted(current_segment));
                            current_segment = String::new();
                            current_quote_type = QuoteType::None; self.advance(); // Consume closing double quote
                        }
                        _ => {
                            if escape_next { // Character is escaped within double quotes (e.g., \n, \", \$)
                                // current_segment already has the backslash
                                current_segment.push(c); // Add the character being escaped
                                escape_next = false; // Reset escape status
                            } else { // Regular character inside double quotes
                                current_segment.push(c);
                            }
                            self.advance();
                        }
                    }
                }
                QuoteType::Escaped => unreachable!(), // Not used in this revised logic
            }
        }

        // Add any remaining segment
        if !current_segment.is_empty() {
            match current_quote_type {
                QuoteType::None => word.add_part(WordPart::Simple(current_segment)),
                QuoteType::Single => word.add_part(WordPart::SingleQuoted(current_segment)), // Unterminated single quote
                QuoteType::Double => word.add_part(WordPart::DoubleQuoted(current_segment)), // Unterminated double quote
                QuoteType::Escaped => unreachable!(),
            }
        }
        word
    }

    fn read_operator(&mut self) -> Token {
        match self.current {
            Some(c) if c.is_ascii_digit() => {
                let num = c.to_digit(10).unwrap() as i32;
                self.advance();
                
                if self.current == Some('>') {
                    self.advance();
                    match self.current {
                        Some('>') => {
                            self.advance();
                            match num {
                                1 => Token::Operator(Operator::RedirectAppend),
                                2 => Token::Operator(Operator::RedirectErrorAppend),
                                n => {
                                    // First put back the number and >> as a Word
                                    let mut word_val = Word::new(); // Create Word
                                    word_val.add_part(WordPart::Simple(n.to_string() + ">>")); // Add as Simple part
                                    Token::Word(word_val)
                                }
                            }
                        }
                        _ => match num {
                            2 => Token::Operator(Operator::RedirectError),
                            _ => Token::Operator(Operator::RedirectOut) // Treat n> as >
                        }
                    }
                } else {
                    let mut word_val = Word::new(); // Create Word
                    word_val.add_part(WordPart::Simple(num.to_string())); // Add as Simple part
                    Token::Word(word_val)
                }
            },
            Some('>') => {
                self.advance();
                match self.current {
                    Some('>') => {
                        self.advance();
                        Token::Operator(Operator::RedirectAppend)
                    }
                    Some('&') => {
                        self.advance();
                        Token::Operator(Operator::RedirectDup)
                    }
                    _ => Token::Operator(Operator::RedirectOut)
                }
            }
            Some('<') => {
                self.advance();
                match self.current {
                    Some('<') => {
                        self.advance();
                        if self.current == Some('<') {
                            self.advance();
                            Token::Operator(Operator::RedirectHereStr)
                        } else {
                            Token::Operator(Operator::RedirectHereDoc)
                        }
                    }
                    Some('&') => {
                        self.advance();
                        Token::Operator(Operator::RedirectDup)
                    }
                    _ => Token::Operator(Operator::RedirectIn)
                }
            }
            Some('|') => {
                self.advance();
                if self.current == Some('|') {
                    self.advance();
                    Token::Operator(Operator::Or)
                } else {
                    Token::Operator(Operator::Pipe)
                }
            }
            Some('&') => {
                self.advance();
                if self.current == Some('&') {
                    self.advance();
                    Token::Operator(Operator::And)
                } else {
                    Token::Operator(Operator::Background)
                }
            }
            Some(';') => {
                self.advance();
                Token::Operator(Operator::Semicolon)
            }
            _ => unreachable!(),
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.current {
            if !matches!(c, ' ' | '\t') {
                break;
            }
            self.advance();
        }
    }

    fn advance(&mut self) {
        self.current = self.input.next();
    }

    // New method to read content within backticks
    fn read_until_backtick(&mut self) -> String {
        let mut content = String::new();
        let mut escape_next = false;
        while let Some(c) = self.current {
            if escape_next {
                // Inside backticks, \\`, \\\\, \\$ are passed literally (e.g. \\` becomes `)
                // Other escapes like \\n are literal \\ then n.
                match c {
                    '`' | '\\' | '$' => content.push(c), // These are escaped to be literal ` or \\ or $
                    _ => { // Other escaped chars: literal backslash then char, e.g. \\n -> \\n
                        content.push('\\');
                        content.push(c);
                    }
                }
                escape_next = false;
                self.advance();
                continue;
            }
            match c {
                '\\' => { // Start of an escape sequence
                    escape_next = true;
                    self.advance(); // Consume the backslash, next char will be handled by `if escape_next` block
                }
                '`' => { // Closing backtick
                    self.advance(); // Consume the closing backtick
                    break;
                }
                _ => {
                    content.push(c);
                    self.advance();
                }
            }
        }
        content
    }
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next_token() {
        tokens.push(token);
    }
    tokens
}
