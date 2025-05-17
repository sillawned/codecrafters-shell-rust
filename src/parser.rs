use crate::ast::{ASTNode, RedirectMode, RedirectTarget};
use crate::lexer::{Token, Operator};
use crate::word::Word; // Import Word

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
        // Skip consecutive Space tokens to simplify logic elsewhere
        self.pos += 1;
        while let Some(Token::Space) = self.tokens.get(self.pos) {
            self.pos += 1;
        }
    }

    fn parse_command(&mut self) -> Result<ASTNode, String> {
        let mut words: Vec<Word> = Vec::new(); // Changed to Vec<Word>
        
        // First, consume leading spaces if any
        if let Some(Token::Space) = self.current_token() {
            self.advance();
        }

        while let Some(token) = self.current_token() {
            match token {
                Token::Word(word) => {
                    words.push(word.clone()); // word is already a Word object
                    self.advance();
                },
                Token::Operator(op @ (Operator::RedirectOut | Operator::RedirectAppend | Operator::RedirectError | Operator::RedirectErrorAppend)) => {
                    let (fd, mode) = match op {
                        Operator::RedirectOut => (1, RedirectMode::Overwrite),
                        Operator::RedirectAppend => (1, RedirectMode::Append),
                        Operator::RedirectError => (2, RedirectMode::Overwrite),
                        Operator::RedirectErrorAppend => (2, RedirectMode::Append),
                        _ => unreachable!(),
                    };

                    self.advance(); // Consume the redirection operator
                    
                    let target_token = self.current_token().ok_or_else(|| "Expected filename or descriptor after redirection operator".to_string())?;
                    // target_str is now a Word, not a String. Executor will handle its expansion.
                    let target_word = match target_token {
                        Token::Word(w) => w.clone(),
                        _ => return Err("Expected filename or descriptor after redirection operator".to_string()),
                    };
                    self.advance(); // Consume the target token

                    if words.is_empty() {
                        return Err("No command before redirection".to_string());
                    }
                    let command = ASTNode::Command {
                        name: words.remove(0), // name is Word
                        args: words,           // args is Vec<Word>
                    };
                    
                    // For RedirectTarget::File, we now pass the Word. Expansion happens in executor.
                    // For RedirectTarget::Descriptor, we need to try to parse the Word as an i32.
                    // This is a simplification; a Word could be complex. For now, assume simple number for descriptor.
                    let target_str_for_parse = target_word.to_string(); // Convert Word to String for parsing attempt
                    let target = if let Ok(target_fd) = target_str_for_parse.parse::<i32>() {
                        // If it parses to int, assume it *could* be a descriptor.
                        // A more robust check might involve ensuring the Word was simple and numeric.
                        RedirectTarget::Descriptor(target_fd)
                    } else {
                        RedirectTarget::File(target_word) // Pass the Word object
                    };

                    return Ok(ASTNode::Redirect {
                        command: Box::new(command),
                        fd,
                        target,
                        mode,
                    });
                },
                Token::Operator(Operator::Pipe) => {
                    self.advance(); // Consume the pipe operator
                    let right = self.parse_command()?; // Recursively parse the right-hand side
                    if words.is_empty() {
                        return Err("No command before pipe".to_string());
                    }
                    let left_command = ASTNode::Command {
                        name: words.remove(0), // name is Word
                        args: words,           // args is Vec<Word>
                    };
                    return Ok(ASTNode::Pipe {
                        left: Box::new(left_command),
                        right: Box::new(right),
                    });
                },
                Token::Space => { // Should be consumed by advance() or handled at start of loop
                    self.advance();
                },
                _ => break, // End of command or unhandled token for this level
            }
        }

        if words.is_empty() {
            // Check if we consumed any tokens at all. If not, it might be an empty input.
            if self.pos == 0 || (self.pos == 1 && matches!(self.tokens.get(0), Some(Token::Space) | Some(Token::NewLine))) {
                 return Err("Empty command".to_string());
            }
            // If words is empty but tokens were consumed (e.g. only operators not forming a full command here)
            // this might indicate an incomplete command, or a command handled fully by an operator branch.
            // This state should ideally be caught by operator logic if it expects a preceding command.
            // If we reach here with empty words after processing some tokens, it implies a syntax issue.
            return Err("Command expected but not found or incomplete command structure".to_string());
        }

        Ok(ASTNode::Command {
            name: words.remove(0), // name is Word
            args: words,           // args is Vec<Word>
        })
    }
}
