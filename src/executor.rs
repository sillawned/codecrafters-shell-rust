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
            '"' => {
                in_quotes = !in_quotes;
                continue;  // Skip the quote character
            }
            '\\' if !in_quotes => {
                if let Some(next) = chars.next() {
                    match next {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        _ => result.push(next),
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
            let paths = std::env::var("PATH").unwrap_or_default();
            
            if utils::is_builtin(name) {
                let processed_args: Vec<String> = args.iter()
                    .map(|arg| process_argument(arg))
                    .collect();
                match builtins::execute_builtin(name, &processed_args) {
                    Ok(()) => Ok(ExitStatus::from_raw(0)),
                    Err(e) => {
                        eprintln!("{}", e);
                        Ok(ExitStatus::from_raw(1))
                    }
                }
            } else if let Some(cmd_path) = search_cmd(name, &paths) {
                let mut cmd = std::process::Command::new(cmd_path);
                let processed_args: Vec<String> = args.iter()
                    .map(|arg| process_argument(arg))
                    .collect();
                cmd.args(&processed_args);
                Ok(cmd.status().map_err(|e| e.to_string())?)
            } else {
                eprintln!("{}: command not found", name);
                Ok(ExitStatus::from_raw(127))
            }
        },
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
            let mut cmd = match &**command {
                ASTNode::Command { name, args } => {
                    let paths = std::env::var("PATH").unwrap_or_default();
                    if let Some(cmd_path) = search_cmd(name, &paths) {
                        let mut cmd = std::process::Command::new(cmd_path);
                        cmd.args(args);
                        cmd
                    } else {
                        eprintln!("{}: command not found", name);
                        return Ok(ExitStatus::from_raw(127));
                    }
                },
                _ => return Err("Invalid redirection".to_string()),
            };

            // Bash-like file handling
            let file_result = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(matches!(mode, RedirectMode::Append))
                .truncate(!matches!(mode, RedirectMode::Append))
                .open(file);

            match file_result {
                Ok(file) => {
                    match fd {
                        1 => cmd.stdout(file),
                        2 => cmd.stderr(file),
                        _ => return Err(format!("Bad file descriptor: {}", fd)),
                    };
                    Ok(cmd.status().unwrap_or(ExitStatus::from_raw(1)))
                },
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::NotFound => {
                            eprintln!("No such file or directory");
                            Ok(ExitStatus::from_raw(1))
                        },
                        std::io::ErrorKind::PermissionDenied => {
                            eprintln!("Permission denied");
                            Ok(ExitStatus::from_raw(1))
                        },
                        _ => {
                            eprintln!("{}", e);
                            Ok(ExitStatus::from_raw(1))
                        }
                    }
                }
            }
        },
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
            if let Some(cmd_path) = search_cmd(name, &paths) {
                let mut cmd = std::process::Command::new(cmd_path);
                cmd.args(args);
                Ok(cmd)
            } else {
                Err(format!("{}: command not found", name))
            }
        }
        ASTNode::Pipe { left, right } => {
            let mut left_cmd = build_command(left)?;
            let mut right_cmd = build_command(right)?;
            
            left_cmd.stdout(std::process::Stdio::piped());
            right_cmd.stdin(std::process::Stdio::piped());
            
            Ok(right_cmd)
        }
        _ => Err("Invalid command node".to_string()),
    }
}