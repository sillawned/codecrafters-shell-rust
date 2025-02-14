use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Operator(Operator),
    Quote(QuoteType),
    Space,
    NewLine,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuoteType {
    Single,     // '
    Double,     // "
    Escaped,    // \
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
                '|' | '&' | '>' | '<' | ';' => {
                    return Some(self.read_operator());
                }
                _ => {
                    return Some(Token::Word(self.read_word()));
                }
            }
        }
        None
    }

    fn read_word(&mut self) -> String {
        let mut word = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        
        while let Some(c) = self.current {
            match (c, in_single_quote, in_double_quote) {
                ('\'', false, false) => {
                    in_single_quote = true;
                    // Keep the quote for command names
                    word.push(c);
                    self.advance();
                }
                ('\'', true, false) => {
                    in_single_quote = false;
                    // Keep the quote for command names
                    word.push(c);
                    self.advance();
                }
                ('"', false, false) => {
                    in_double_quote = true;
                    // Keep the quote for command names
                    word.push(c);
                    self.advance();
                }
                ('"', false, true) => {
                    in_double_quote = false;
                    // Keep the quote for command names
                    word.push(c);
                    self.advance();
                }
                ('\\', false, true) => {
                    self.advance(); // consume backslash
                    if let Some(next) = self.current {
                        // Keep escaped quotes in the word
                        if next == '\'' || next == '"' {
                            word.push('\\');
                        }
                        word.push(next);
                        self.advance();
                    }
                }
                ('\\', false, false) => {
                    self.advance(); // consume backslash
                    if let Some(next) = self.current {
                        // Keep escaped quotes in the word
                        if next == '\'' || next == '"' {
                            word.push('\\');
                        }
                        word.push(next);
                        self.advance();
                    }
                }
                (' ' | '\t' | '\n' | '|' | '&' | '>' | '<' | ';', false, false) => break,
                _ => {
                    word.push(c);
                    self.advance();
                }
            }
        }
        word
    }

    // Add new function to handle special parameters
    fn read_special_param(&mut self) -> Option<Token> {
        self.advance(); // consume $
        match self.current {
            Some('?') => {
                self.advance();
                Some(Token::Word(std::env::var("?").unwrap_or_else(|_| "0".to_string())))
            },
            Some('#') => {
                self.advance();
                Some(Token::Word("0".to_string())) // Placeholder for arg count
            },
            _ => None
        }
    }

    fn read_operator(&mut self) -> Token {
        match self.current {
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

    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
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
