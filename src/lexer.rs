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
        while let Some(c) = self.current {
            match c {
                ' ' | '\t' => {
                    self.advance(); // Just advance past the space
                    // Only return Space token if we haven't reached the end
                    if self.current.is_some() {
                        return Some(Token::Space);
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
        let mut in_quotes = false;
        let mut quote_char = None;
        let mut escaped = false;

        while let Some(c) = self.current {
            match (c, escaped, in_quotes) {
                (c, true, _) => {
                    // Handle escaped characters
                    match c {
                        'n' => word.push('\n'),
                        't' => word.push('\t'),
                        'r' => word.push('\r'),
                        _ => word.push(c),
                    }
                    escaped = false;
                    self.advance();
                },
                ('\\', false, _) => {
                    escaped = true;
                    self.advance();
                },
                ('"' | '\'', false, false) => {
                    in_quotes = true;
                    quote_char = Some(c);
                    self.advance();
                },
                (c, false, true) if Some(c) == quote_char => {
                    in_quotes = false;
                    quote_char = None;
                    self.advance();
                },
                (' ' | '\t' | '\n' | '|' | '&' | '>' | '<' | ';', false, false) => break,
                (_, false, _) => {
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
            Some('>') => {
                self.advance();
                if self.current == Some('>') {
                    self.advance();
                    Token::Operator(Operator::RedirectAppend)
                } else {
                    Token::Operator(Operator::RedirectOut)
                }
            }
            Some('<') => {
                self.advance();
                Token::Operator(Operator::RedirectIn)
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
