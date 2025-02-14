use crate::ast::{ASTNode, RedirectMode};
use crate::lexer::{Token, Operator, QuoteType};

pub fn parse(tokens: &[Token]) -> Result<ASTNode, String> {
    let mut iter = tokens.iter().peekable();
    let mut nodes = Vec::new();
    let mut current_command = Vec::new();
    let mut current_word = String::new();

    while let Some(token) = iter.next() {
        match token {
            Token::Word(word) => {
                current_word.push_str(word);
                if !iter.peek().map_or(false, |t| matches!(t, Token::Quote(_))) {
                    current_command.push(current_word.clone());
                    current_word.clear();
                }
            }
            Token::Quote(quote_type) => {
                match quote_type {
                    QuoteType::Single | QuoteType::Double => {
                        if let Some(Token::Word(content)) = iter.next() {
                            current_word.push_str(content);
                            // Look for closing quote
                            if let Some(Token::Quote(closing)) = iter.next() {
                                if closing == quote_type {
                                    current_command.push(current_word.clone());
                                    current_word.clear();
                                }
                            }
                        }
                    }
                    QuoteType::Escaped => {
                        if let Some(Token::Word(next)) = iter.next() {
                            current_word.push_str(next);
                        }
                    }
                }
            }
            Token::Operator(op) => {
                if !current_command.is_empty() {
                    nodes.push(create_command(&current_command)?);
                    current_command.clear();
                }
                match op {
                    Operator::RedirectOut => {
                        if let Some(Token::Word(file)) = iter.next() {
                            let command = nodes.pop().ok_or("No command before redirection")?;
                            nodes.push(ASTNode::Redirect {
                                command: Box::new(command),
                                fd: 1,
                                file: file.clone(),
                                mode: RedirectMode::Overwrite,
                            });
                        }
                    }
                    Operator::Pipe => {
                        if let Some(right) = parse_command(&mut iter)? {
                            let left = nodes.pop().ok_or("No command before pipe")?;
                            nodes.push(ASTNode::Pipe {
                                left: Box::new(left),
                                right: Box::new(right),
                            });
                        }
                    }
                    Operator::PipeAnd | Operator::And | Operator::Or | Operator::Background | Operator::RedirectIn | Operator::RedirectAppend | Operator::Semicolon | Operator::RedirectError => {
                        return Err(format!("Operator {:?} not implemented", op));
                    }
                }
            }
            Token::Space | Token::NewLine => {
                if !current_word.is_empty() {
                    current_command.push(current_word.clone());
                    current_word.clear();
                }
            }
        }
    }

    if !current_command.is_empty() {
        nodes.push(create_command(&current_command)?);
    }

    Ok(nodes.pop().ok_or("No valid command found")?)
}

fn parse_command<'a, I>(tokens: &mut I) -> Result<Option<ASTNode>, String>
where
    I: Iterator<Item = &'a Token>,
{
    let mut command = Vec::new();
    let mut in_command = true;

    while let Some(token) = tokens.next() {
        match token {
            Token::Word(word) => {
                command.push(word.clone());
            }
            Token::Quote(qt) => match qt {
                QuoteType::Single | QuoteType::Double => {
                    if let Some(Token::Word(word)) = tokens.next() {
                        command.push(word.clone());
                        // Skip closing quote
                        tokens.next();
                    }
                }
                QuoteType::Escaped => {
                    if let Some(Token::Word(word)) = tokens.next() {
                        command.push(word.clone());
                    }
                }
            },
            Token::Operator(op) => match op {
                Operator::Pipe => {
                    let left = create_command(&command)?;
                    if let Some(right) = parse_command(tokens)? {
                        return Ok(Some(ASTNode::Pipe {
                            left: Box::new(left),
                            right: Box::new(right),
                        }));
                    }
                    break;
                }
                Operator::RedirectOut => {
                    let cmd = create_command(&command)?;
                    if let Some(Token::Word(file)) = tokens.next() {
                        return Ok(Some(ASTNode::Redirect {
                            command: Box::new(cmd),
                            fd: 1,
                            file: file.clone(),
                            mode: RedirectMode::Overwrite,
                        }));
                    }
                    break;
                }
                _ => {
                    in_command = false;
                    break;
                }
            },
            Token::Space => continue,
            _ => {
                in_command = false;
                break;
            }
        }
    }

    if in_command && !command.is_empty() {
        Ok(Some(create_command(&command)?))
    } else {
        Ok(None)
    }
}

fn create_command(words: &[String]) -> Result<ASTNode, String> {
    if words.is_empty() {
        return Err("Empty command".to_string());
    }
    
    Ok(ASTNode::Command {
        name: words[0].clone(),
        args: words[1..].to_vec(),
    })
}
