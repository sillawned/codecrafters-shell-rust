use crate::ast::{ASTNode, RedirectMode};
use crate::builtins::{self, BUILTINS};
use crate::utils::search_cmd;

pub fn execute(node: &ASTNode) -> Result<(), String> {
    #[cfg(debug_assertions)]
    println!("Executing: {:?}", node);

    match node {
        ASTNode::Command { name, args } => {
            if BUILTINS.contains(&name.as_str()) {
                builtins::execute_builtin(name, args)
            } else {
                let paths = std::env::var("PATH").unwrap();
                if let Some(cmd_path) = search_cmd(name, &paths) {
                    let mut cmd = std::process::Command::new(cmd_path);
                    cmd.args(args);
                    let status = cmd.status().map_err(|e| e.to_string())?;
                    if !status.success() {
                        // return Err(format!("Command failed with status: {}", status));
                    }
                    Ok(())
                } else {
                    Err(format!("{}: command not found", name))
                }
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
        ASTNode::Redirect { command, fd, file, mode } => {
            let mut cmd = build_command(command)?;
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(matches!(mode, RedirectMode::Append))
                .truncate(!matches!(mode, RedirectMode::Append))
                .open(file)
                .map_err(|e| e.to_string())?;

            match fd {
                1 => cmd.stdout(file),
                2 => cmd.stderr(file),
                _ => return Err(format!("Unsupported file descriptor: {}", fd)),
            };

            let status = cmd.status().map_err(|e| e.to_string())?;
            if !status.success() {
                //return Err(format!("Redirection failed with status: {}", status));
            }
            Ok(())
        }
        ASTNode::Background { command } => {
            let mut cmd = build_command(command)?;
            cmd.spawn().map_err(|e| e.to_string())?;
            Ok(())
        }
        ASTNode::LogicalAnd { left, right } => {
            if execute(left).is_ok() {
                execute(right)
            } else {
                Ok(())
            }
        }
        ASTNode::LogicalOr { left, right } => {
            if execute(left).is_err() {
                execute(right)
            } else {
                Ok(())
            }
        }
        ASTNode::Subshell { command } => {
            let mut cmd = build_command(command)?;
            let status = cmd.status().map_err(|e| e.to_string())?;
            if !status.success() {
                return Err(format!("Subshell failed with status: {}", status));
            }
            Ok(())
        }
        ASTNode::Semicolon { left, right } => {
            execute(left)?;
            execute(right)
        }
        #[allow(unreachable_patterns)]
        _ => {
            Err("Unsupported ASTNode".to_string())
        }
    }
}

fn build_command(node: &ASTNode) -> Result<std::process::Command, String> {
    match node {
        ASTNode::Command { name, args } => {
            let paths = std::env::var("PATH").unwrap();
            if let Some(cmd_path) = search_cmd(name, &paths) {
                let mut cmd = std::process::Command::new(cmd_path);
                cmd.args(args);
                Ok(cmd)
            } else {
                Err(format!("{}: command not found", name))
            }
        }
        _ => Err("Unsupported ASTNode for command building".to_string()),
    }
}