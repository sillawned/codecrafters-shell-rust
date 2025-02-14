use crate::ast::{ASTNode, RedirectMode};
use crate::tokenizer::TokenType;
use crate::processor::{process_text, ProcessingMode};

pub fn parse(tokens: &Vec<TokenType>) -> Result<ASTNode, String> {
    let mut iter = tokens.iter().peekable();
    let ast = parse_sequence(&mut iter)?;

    #[cfg(debug_assertions)]
    println!("AST: {:?}", ast);

    Ok(ast)
}

fn parse_sequence<'a, I>(tokens: &mut std::iter::Peekable<I>) -> Result<ASTNode, String>
where
    I: Iterator<Item = &'a TokenType>,
{
    let mut left = parse_logical(tokens)?;
    while let Some(token) = tokens.peek() {
        if let TokenType::Semicolon = token {
            tokens.next(); // Consume the ";"
            let right = parse_logical(tokens)?;
            left = ASTNode::Semicolon {
                left: Box::new(left),
                right: Box::new(right),
            };
        } else {
            break;
        }
    }
    Ok(left)
}

fn parse_logical<'a, I>(tokens: &mut std::iter::Peekable<I>) -> Result<ASTNode, String>
where
    I: Iterator<Item = &'a TokenType>,
{
    let mut left = parse_pipeline(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            TokenType::LogicalAnd => {
                tokens.next(); // Consume the "&&"
                let right = parse_pipeline(tokens)?;
                left = ASTNode::LogicalAnd {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            }
            TokenType::LogicalOr => {
                tokens.next(); // Consume the "||"
                let right = parse_pipeline(tokens)?;
                left = ASTNode::LogicalOr {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }
    Ok(left)
}

fn parse_pipeline<'a, I>(tokens: &mut std::iter::Peekable<I>) -> Result<ASTNode, String>
where
    I: Iterator<Item = &'a TokenType>,
{
    let mut left = parse_command_with_redirects(tokens)?;
    while let Some(token) = tokens.peek() {
        if let TokenType::Pipe = token {
            tokens.next(); // Consume the "|"
            let right = parse_command_with_redirects(tokens)?;
            left = ASTNode::Pipe {
                left: Box::new(left),
                right: Box::new(right),
            };
        } else {
            break;
        }
    }
    Ok(left)
}

fn parse_command_with_redirects<'a, I>(tokens: &mut std::iter::Peekable<I>) -> Result<ASTNode, String>
where
    I: Iterator<Item = &'a TokenType>,
{
    let mut command = parse_command(tokens)?;
    let mut fd = -1; // Default is no file descriptor

    while let Some(token) = tokens.peek() {
        match token {
            TokenType::FileDescriptor(num) => {
                fd = *num;
                tokens.next();
            }
            TokenType::RedirectionOperator(op) => {
                tokens.next(); // Consume the redirection operator
                if fd == -1 {
                    fd = if op == "<" { 0 } else { 1 };
                }
                
                // Skip spaces until we find a word or quoted string
                while let Some(TokenType::Space) = tokens.peek() {
                    tokens.next();
                }
                
                // Collect the filename, which might span multiple tokens
                let mut filename = String::new();
                while let Some(token) = tokens.peek() {
                    match token {
                        TokenType::Word(word) => {
                            filename.push_str(word);
                            tokens.next();
                        }
                        TokenType::SingleQuotedString(word) |
                        TokenType::DoubleQuotedString(word) => {
                            filename.push_str(word);
                            tokens.next();
                        }
                        TokenType::Space => {
                            // Stop at space unless next token is part of path
                            let mut lookahead = tokens.peekable();
                            lookahead.next(); // Skip current space
                            if let Some(TokenType::Word(w)) = lookahead.peek() {
                                if w.starts_with('/') {
                                    filename.push('/');
                                    tokens.next(); // consume space
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }

                if filename.is_empty() {
                    return Err("Expected file after redirection operator".to_string());
                }

                let filename = process_text(&filename, ProcessingMode::Path);
                command = ASTNode::Redirect {
                    command: Box::new(command),
                    file: filename,
                    fd,
                    mode: match op.as_str() {
                        ">" => RedirectMode::Overwrite,
                        ">>" => RedirectMode::Append,
                        "<" => RedirectMode::Input,
                        _ => return Err(format!("Unknown redirection operator: {}", op)),
                    },
                };
                fd = -1;
            }
            TokenType::Space => {
                tokens.next();
            }
            _ => break,
        }
    }
    Ok(command)
}

fn parse_command<'a, I>(tokens: &mut std::iter::Peekable<I>) -> Result<ASTNode, String>
where
    I: Iterator<Item = &'a TokenType>,
{
    let mut args = Vec::new();
    let mut name = String::new();

    while let Some(token) = tokens.peek() {
        match token {
            TokenType::Word(word) => {
                if name.is_empty() {
                    name = word.clone();
                } else {
                    args.push(word.clone());
                }
                tokens.next();
            }
            TokenType::SingleQuotedString(word) => {
                let value = word.clone(); // Preserve literal value
                if name.is_empty() {
                    name = value;
                } else {
                    args.push(value);
                }
                tokens.next();
            }
            TokenType::DoubleQuotedString(word) => {
                let value = word.clone(); // May contain escaped characters
                if name.is_empty() {
                    name = value;
                } else {
                    args.push(value);
                }
                tokens.next();
            }
            TokenType::DollarVar(var) => {
                args.push(format!("${}", var));
                tokens.next();
            }
            TokenType::CommandSubstitution(cmd) => {
                args.push(format!("$({})", cmd));
                tokens.next();
            }
            TokenType::Assignment(var, val) => {
                args.push(format!("{}={}", var, val));
                tokens.next();
            }
            TokenType::Comment(_) => {
                tokens.next();
                break;
            }
            TokenType::Space => {
                tokens.next(); // Consume the space
            }
            _ => break,
        }
    }

    if name.is_empty() {
        Err("Expected command".to_string())
    } else {
        Ok(ASTNode::Command { name, args })
    }
}
