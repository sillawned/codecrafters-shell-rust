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
                    self.consume_whitespace();
                    return Some(Token::Space);
                }
                '\n' => {
                    self.advance();
                    return Some(Token::NewLine);
                }
                '|' => {
                    self.advance();
                    return Some(match self.peek() {
                        Some('|') => {
                            self.advance();
                            Token::Operator(Operator::Or)
                        }
                        Some('&') => {
                            self.advance();
                            Token::Operator(Operator::PipeAnd)
                        }
                        _ => Token::Operator(Operator::Pipe)
                    });
                }
                '&' => {
                    self.advance();
                    return Some(match self.peek() {
                        Some('&') => {
                            self.advance();
                            Token::Operator(Operator::And)
                        }
                        _ => Token::Operator(Operator::Background)
                    });
                }
                '>' => {
                    self.advance();
                    return Some(match self.peek() {
                        Some('>') => {
                            self.advance();
                            Token::Operator(Operator::RedirectAppend)
                        }
                        _ => Token::Operator(Operator::RedirectOut)
                    });
                }
                '<' => {
                    self.advance();
                    return Some(Token::Operator(Operator::RedirectIn));
                }
                ';' => {
                    self.advance();
                    return Some(Token::Operator(Operator::Semicolon));
                }
                '\'' => {
                    self.advance();
                    return Some(Token::Quote(QuoteType::Single));
                }
                '"' => {
                    self.advance();
                    return Some(Token::Quote(QuoteType::Double));
                }
                '\\' => {
                    self.advance();
                    return Some(Token::Quote(QuoteType::Escaped));
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
        while let Some(c) = self.current {
            match c {
                ' ' | '\t' | '\n' | '|' | '&' | '>' | '<' | ';' | '\'' | '"' | '\\' => break,
                _ => {
                    word.push(c);
                    self.advance();
                }
            }
        }
        word
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
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
