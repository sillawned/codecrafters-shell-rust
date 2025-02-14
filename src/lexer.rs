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

        while let Some(c) = self.current {
            match (c, in_quotes) {
                ('"' | '\'', false) => {
                    in_quotes = true;
                    quote_char = Some(c);
                    word.push(c);  // Keep the quotes
                    self.advance();
                }
                (c, true) if Some(c) == quote_char => {
                    word.push(c);  // Keep the quotes
                    self.advance();
                    in_quotes = false;
                    quote_char = None;
                }
                ('\\', _) => {
                    word.push(c);
                    self.advance();
                    if let Some(next) = self.current {
                        word.push(next);
                        self.advance();
                    }
                }
                (' ' | '\t' | '\n' | '|' | '&' | '>' | '<' | ';', false) => break,
                (_, _) => {
                    word.push(c);
                    self.advance();
                }
            }
        }
        word
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
