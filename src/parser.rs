use crate::ast::{ASTNode, RedirectMode};
use crate::builtins::BUILTINS;

pub fn parse(tokens: &[String]) -> Result<ASTNode, String> {
  let mut iter = tokens.iter().peekable();
  parse_pipe(&mut iter)
}

fn parse_pipe<'a, I>(tokens: &mut std::iter::Peekable<I>) -> Result<ASTNode, String>
where
  I: Iterator<Item = &'a String>,
{
  let mut left = parse_redirect(tokens)?;
  while let Some(token) = tokens.peek() {
      if token.as_str() == "|" {
          tokens.next(); // Consume the "|"
          let right = parse_redirect(tokens)?;
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

fn parse_redirect<'a, I>(tokens: &mut std::iter::Peekable<I>) -> Result<ASTNode, String>
where
  I: Iterator<Item = &'a String>,
{
  let mut command = parse_command(tokens)?;
  while let Some(token) = tokens.peek() {
      match token.as_str() {
          ">" | ">>" | "<" => {
              let mode = match token.as_str() {
                  ">" => RedirectMode::Overwrite,
                  ">>" => RedirectMode::Append,
                  "<" => RedirectMode::Input,
                  _ => unreachable!(),
              };
              tokens.next(); // Consume the redirection token
              let file = tokens.next().ok_or("Expected file after redirection")?.clone();
              command = ASTNode::Redirect {
                  command: Box::new(command),
                  file,
                  mode,
              };
          }
          _ => break,
      }
  }
  Ok(command)
}

fn parse_command<'a, I>(tokens: &mut std::iter::Peekable<I>) -> Result<ASTNode, String>
where
  I: Iterator<Item = &'a String>,
{
  let name = tokens.next().ok_or("Expected command name")?.clone();
  let mut args = Vec::new();
  while let Some(token) = tokens.peek() {
      if token.as_str() == "|" || token.as_str() == ">" || token.as_str() == ">>" || token.as_str() == "<" {
          break;
      }
      args.push(tokens.next().unwrap().clone());
  }
  if BUILTINS.contains(&&*name) {
      Ok(ASTNode::Builtin { name, args })
  } else {
      Ok(ASTNode::Command { name, args })
  }
}