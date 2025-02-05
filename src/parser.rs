use crate::ast::{ASTNode, RedirectMode};
use crate::tokenizer::TokenType;

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
                tokens.next(); // Consume the file descriptor
            }
            TokenType::RedirectionOperator(op) => {
                tokens.next(); // Consume the redirection operator
                if let Some(TokenType::Word(file)) = tokens.next() {
                    command = ASTNode::Redirect {
                        command: Box::new(command),
                        file: file.clone(),
                        fd,
                        mode: match op.as_str() {
                            ">" => RedirectMode::Overwrite,
                            ">>" => RedirectMode::Append,
                            "<" => RedirectMode::Input,
                            _ => return Err(format!("Unknown redirection operator: {}", op)),
                        },
                    };
                } else {
                    return Err("Expected file after redirection operator".to_string());
                }
            }
            TokenType::Background => {
                tokens.next(); // Consume the "&"
                command = ASTNode::Background {
                    command: Box::new(command),
                };
            }
            TokenType::LeftParen => {
                tokens.next(); // Consume the "("
                let subshell_command = parse_sequence(tokens)?;
                if let Some(TokenType::RightParen) = tokens.next() {
                    command = ASTNode::Subshell {
                        command: Box::new(subshell_command),
                    };
                } else {
                    return Err("Expected closing parenthesis for subshell".to_string());
                }
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
                tokens.next(); // Consume the word
            }
            TokenType::DollarVar(var) => {
                args.push(format!("${}", var));
                tokens.next(); // Consume the variable
            }
            TokenType::CommandSubstitution(cmd) => {
                args.push(format!("$({})", cmd));
                tokens.next(); // Consume the command substitution
            }
            TokenType::Quote(quote) => {
                args.push(quote.to_string());
                tokens.next(); // Consume the quote
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
