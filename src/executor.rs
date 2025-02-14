use crate::ast::{ASTNode, RedirectMode};
use crate::builtins;
use crate::utils::{self, search_cmd};
use std::process::ExitStatus;
use std::os::unix::process::ExitStatusExt;
use crate::processor::{process_text, ProcessingMode};

fn process_argument(arg: &str) -> String {
    let mut result = String::new();
    let mut chars = arg.chars().peekable();
    let mut in_quotes = false;
    
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                in_quotes = !in_quotes;
                result.push(c);
            }
            '\\' => {
                if in_quotes {
                    // Inside quotes, preserve backslash and next character literally
                    result.push('\\');
                    if let Some(next) = chars.next() {
                        result.push(next);
                    }
                } else {
                    // Outside quotes, handle escaped characters
                    if let Some(next) = chars.next() {
                        match next {
                            ' ' | '\'' | '"' => result.push(next),
                            '\\' => result.push('\\'),
                            'n' => result.push('\n'),
                            't' => result.push('\t'),
                            _ => {
                                result.push('\\');
                                result.push(next);
                            }
                        }
                    }
                }
            }
            _ => result.push(c),
        }
    }
    result
}

pub fn execute(node: &ASTNode) -> Result<ExitStatus, String> {
    #[cfg(debug_assertions)]
    println!("Executing: {:?}", node);

    match node {
        ASTNode::Command { name, args } => {
            if utils::is_builtin(name) {
                // Convert builtin result to ExitStatus
                match builtins::execute_builtin(name, args) {
                    Ok(()) => Ok(ExitStatus::from_raw(0)),
                    Err(_) => Ok(ExitStatus::from_raw(1))
                }
            } else {
                let paths = std::env::var("PATH").unwrap_or_default();
                if let Some(cmd_path) = search_cmd(name, &paths) {
                    let mut cmd = std::process::Command::new(cmd_path);
                    cmd.args(args.iter().map(|arg| process_argument(arg))); // Handle escaped sequences
                    Ok(cmd.status().map_err(|e| e.to_string())?)
                } else {
                    eprintln!("{}: command not found", name);
                    Ok(ExitStatus::from_raw(127)) // 127 is standard for command not found
                }
            }
        }
        ASTNode::Pipe { left, right } => {
            // Preserve exit status of the rightmost command in a pipeline
            let mut left_cmd = build_command(left)?;
            let mut right_cmd = build_command(right)?;

            let left_output = left_cmd.stdout(std::process::Stdio::piped()).spawn().map_err(|e| e.to_string())?;
            let right_input = left_output.stdout.ok_or("Failed to capture left command output")?;
            right_cmd.stdin(std::process::Stdio::from(right_input));

            Ok(right_cmd.status().map_err(|e| e.to_string())?)
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
            Ok(status)
        }
        ASTNode::Background { command } => {
            let mut cmd = build_command(command)?;
            cmd.spawn().map_err(|e| e.to_string())?;
            Ok(ExitStatus::from_raw(0))
        }
        ASTNode::LogicalAnd { left, right } => {
            if execute(left)?.success() {
                execute(right)
            } else {
                Ok(ExitStatus::from_raw(0))
            }
        }
        ASTNode::LogicalOr { left, right } => {
            if !execute(left)?.success() {
                execute(right)
            } else {
                Ok(ExitStatus::from_raw(0))
            }
        }
        ASTNode::Subshell { command } => {
            let mut cmd = build_command(command)?;
            let status = cmd.status().map_err(|e| e.to_string())?;
            if !status.success() {
                return Err(format!("Subshell failed with status: {}", status));
            }
            Ok(status)
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
            let paths = std::env::var("PATH").unwrap_or_default();
            let cmd_parts: Vec<&str> = name.split_whitespace().collect();
            
            if cmd_parts.is_empty() {
                return Err("Empty command".to_string());
            }

            // Try to find the command
            if let Some(cmd_path) = search_cmd(cmd_parts[0], &paths) {
                let mut cmd = std::process::Command::new(cmd_path);
                // Add any command flags from the name
                if cmd_parts.len() > 1 {
                    cmd.args(&cmd_parts[1..]);
                }
                // Add explicit arguments
                cmd.args(args);
                Ok(cmd)
            } else {
                Err(format!("{}: command not found", cmd_parts[0]))
            }
        }
        _ => Err("Unsupported ASTNode for command building".to_string()),
    }
}