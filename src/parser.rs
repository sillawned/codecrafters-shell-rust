use crate::ast::{ASTNode, RedirectMode};
use crate::lexer::{Token, Operator, QuoteType};

pub fn parse(tokens: &[Token]) -> Result<ASTNode, String> {
    let mut parser = Parser::new(tokens);
    parser.parse_command()
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current_token(&self) -> Option<&'a Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    fn parse_command(&mut self) -> Result<ASTNode, String> {
        let mut words = Vec::new();
        let mut redirects = Vec::new();
        
        while let Some(token) = self.current_token() {
            match token {
                Token::Word(word) => {
                    words.push(word.clone());
                    self.advance();
                },
                Token::Quote(quote_type) => {
                    match quote_type {
                        QuoteType::Single | QuoteType::Double => {
                            if let Some(Token::Word(content)) = self.peek_next() {
                                words.push(content.clone());
                                self.advance(); // consume word
                                self.advance(); // consume closing quote
                            }
                        }
                        QuoteType::Escaped => {
                            if let Some(Token::Word(next)) = self.peek_next() {
                                words.push(next.clone());
                                self.advance();
                            }
                        }
                    }
                    self.advance();
                },
                Token::Operator(Operator::RedirectOut) => {
                    self.advance();
                    if let Some(Token::Word(file)) = self.current_token() {
                        redirects.push((1, RedirectMode::Overwrite, file.clone()));
                        self.advance();
                    } else {
                        return Err("Expected filename after redirection".to_string());
                    }
                },
                Token::Operator(Operator::Pipe) => {
                    self.advance();
                    let right = self.parse_command()?;
                    if words.is_empty() {
                        return Err("No command before pipe".to_string());
                    }
                    return Ok(ASTNode::Pipe {
                        left: Box::new(ASTNode::Command {
                            name: words.remove(0),
                            args: words,
                        }),
                        right: Box::new(right),
                    });
                },
                _ => break,
            }
        }

        if words.is_empty() {
            return Err("Empty command".to_string());
        }

        let mut command = ASTNode::Command {
            name: words.remove(0),
            args: words,
        };

        // Apply redirections in order
        for (fd, mode, file) in redirects {
            command = ASTNode::Redirect {
                command: Box::new(command),
                fd,
                file,
                mode,
            };
        }

        Ok(command)
    }
}
