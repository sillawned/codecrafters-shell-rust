use crate::ast::{ASTNode, RedirectMode};
use crate::builtins;
use crate::utils::search_cmd;

pub fn execute(node: &ASTNode) -> Result<(), String> {
  match node {
      ASTNode::Command { name, args } => {
          let paths = std::env::var("PATH").unwrap();
          if let Some(cmd_path) = search_cmd(name, &paths) {
              let mut cmd = std::process::Command::new(cmd_path);
              cmd.args(args);
              let status = cmd.status().map_err(|e| e.to_string())?;
              if !status.success() {
                  return Err(format!("Command failed with status: {}", status));
              }
              Ok(())
          } else {
              Err(format!("{}: command not found", name))
          }
      }
      ASTNode::Pipe { left, right } => {
          let mut left_cmd = build_command(left)?;
          let mut right_cmd = build_command(right)?;

          let left_output = left_cmd.stdout(std::process::Stdio::piped()).spawn().map_err(|e| e.to_string())?;
          let right_input = left_output.stdout.ok_or("Failed to capture left command output")?;
          right_cmd.stdin(std::process::Stdio::from(right_input));

          let status = right_cmd.status().map_err(|e| e.to_string())?;
          if !status.success() {
              return Err(format!("Pipe failed with status: {}", status));
          }
          Ok(())
      }
      ASTNode::Redirect { command, file, mode } => {
          let mut cmd = build_command(command)?;
          let file = std::fs::OpenOptions::new()
              .write(true)
              .create(true)
              .append(matches!(mode, RedirectMode::Append))
              .truncate(!matches!(mode, RedirectMode::Append))
              .open(file)
              .map_err(|e| e.to_string())?;
          cmd.stdout(file);
          let status = cmd.status().map_err(|e| e.to_string())?;
          if !status.success() {
              return Err(format!("Redirect failed with status: {}", status));
          }
          Ok(())
      }
      ASTNode::Builtin { name, args } => {
          builtins::execute_builtin(name, args)
      }
  }
}

fn build_command(node: &ASTNode) -> Result<std::process::Command, String> {
  match node {
      ASTNode::Command { name, args } => {
          let mut cmd = std::process::Command::new(name);
          cmd.args(args);
          Ok(cmd)
      }
      _ => Err("Expected a command node".to_string()),
  }
}