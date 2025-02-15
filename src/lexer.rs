use std::iter::Peekable;
use std::str::Chars;
use crate::{
    types::QuoteType,
    word::{Word, WordPart},
};

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
                    return Some(Token::Word(self.read_word()));
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
        let mut word = Word::new();
        let mut current = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escape_next = false;
        
        while let Some(c) = self.current {
            match (c, escape_next, in_single_quote, in_double_quote) {
                (c, true, _, _) => {
                    current.push(c);
                    escape_next = false;
                    self.advance();
                },
                ('\\', false, false, true) | ('\\', false, false, false) => {
                    escape_next = true;
                    current.push(c);
                    self.advance();
                },
                ('\'', false, false, false) => {
                    if !current.is_empty() {
                        word.add_part(WordPart::Simple(current));
                        current = String::new();
                    }
                    in_single_quote = true;
                    self.advance();
                },
                ('\'', false, true, false) => {
                    word.add_part(WordPart::SingleQuoted(current));
                    current = String::new();
                    in_single_quote = false;
                    self.advance();
                },
                ('"', false, false, false) => {
                    if !current.is_empty() {
                        word.add_part(WordPart::Simple(current));
                        current = String::new();
                    }
                    in_double_quote = true;
                    self.advance();
                },
                ('"', false, false, true) => {
                    word.add_part(WordPart::DoubleQuoted(current));
                    current = String::new();
                    in_double_quote = false;
                    self.advance();
                },
                (' ' | '\t' | '\n' | '|' | '&' | '>' | '<' | ';', false, false, false) => break,
                (c, _, _, _) => {
                    current.push(c);
                    self.advance();
                }
            }
        }

        if !current.is_empty() {
            match (in_single_quote, in_double_quote) {
                (true, _) => word.add_part(WordPart::SingleQuoted(current)),
                (_, true) => word.add_part(WordPart::DoubleQuoted(current)),
                _ => word.add_part(WordPart::Simple(current)),
            }
        }

        word.to_string()
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
                                    let mut word = n.to_string();
                                    word.push_str(">>");
                                    Token::Word(word)
                                }
                            }
                        }
                        _ => match num {
                            2 => Token::Operator(Operator::RedirectError),
                            _ => Token::Operator(Operator::RedirectOut) // Treat n> as >
                        }
                    }
                } else {
                    Token::Word(num.to_string())
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

}

pub fn lex(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next_token() {
        tokens.push(token);
    }
    tokens
}
