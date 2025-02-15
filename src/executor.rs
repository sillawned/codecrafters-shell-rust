use std::io::Write;
use std::process::ExitStatus;
use std::os::unix::process::{ExitStatusExt, CommandExt};
use crate::{
    ast::{ASTNode, RedirectMode, RedirectTarget},
    builtins,
    utils::{self, search_cmd},
};
use tempfile;

pub struct Executor {
    last_status: i32,
    environment: std::collections::HashMap<String, String>,
    current_dir: std::path::PathBuf,
}

fn execute_command(cmd: &mut std::process::Command) -> Result<ExitStatus, String> {
    // Set up process group
    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }

    match cmd.status() {
        Ok(status) => Ok(status),
        Err(e) => Err(format!("Failed to execute command: {}", e))
    }
}

impl Executor {
    pub fn new() -> Self {
        Self {
            last_status: 0,
            environment: std::env::vars().collect(),
            current_dir: std::env::current_dir().unwrap(),
        }
    }

    pub fn execute(&mut self, node: &ASTNode) -> Result<ExitStatus, String> {
        match node {
            ASTNode::Command { name, args } => {
                let processed_name = name.to_string();

                let expanded_args: Vec<String> = args.iter()
                    .map(|arg| self.expand_variables(arg))
                    .collect::<Result<_, _>>()?;
                #[cfg(debug_assertions)]
                println!("Expanded args: {:?}", expanded_args);

                if utils::is_builtin(&processed_name) {
                    match builtins::execute_builtin(&processed_name, &expanded_args) {
                        Ok(()) => Ok(ExitStatus::from_raw(0)),
                        Err(e) => {
                            eprintln!("{}", e);
                            Ok(ExitStatus::from_raw(1))
                        }
                    }
                } else if let Some(cmd_path) = search_cmd(&processed_name, &std::env::var("PATH").unwrap_or_default()) {
                    let mut cmd = std::process::Command::new(cmd_path);
                    cmd.arg0(&processed_name);
                    cmd.args(&expanded_args);
                    // Handle output redirection for external commands
                    cmd.stdout(std::process::Stdio::inherit());
                    execute_command(&mut cmd)
                } else {
                    eprintln!("{}: command not found", processed_name);
                    Ok(ExitStatus::from_raw(127))
                }
            }
            ASTNode::Redirect { command, fd, target, mode } => {
                match target {
                    RedirectTarget::File(path) => {
                        // Create parent directories if needed
                        if let Some(parent) = std::path::Path::new(path).parent() {
                            if !parent.as_os_str().is_empty() {
                                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                            }
                        }

                        // Set up the redirection file with append mode
                        let file = std::fs::OpenOptions::new()
                            .write(true)
                            .create(true)
                            .append(matches!(mode, RedirectMode::Append))
                            .truncate(matches!(mode, RedirectMode::Overwrite))
                            .open(path)
                            .map_err(|e| format!("Failed to open {}: {}", path, e))?;

                        self.execute_with_redirection(command, *fd, file, mode)
                    }
                    RedirectTarget::Descriptor(target_fd) => {
                        self.execute_with_fd_duplication(command, *fd, *target_fd)
                    }
                    RedirectTarget::HereDoc(content) => {
                        // Create temporary file with heredoc content
                        let mut temp_file = tempfile::NamedTempFile::new()
                            .map_err(|e| format!("Failed to create temporary file: {}", e))?;
                        std::io::Write::write_all(&mut temp_file, content.as_bytes())
                            .map_err(|e| format!("Failed to write heredoc: {}", e))?;
                        self.execute_with_redirection(command, *fd, temp_file.into_file(), mode)
                    }
                    RedirectTarget::HereString(content) => {
                        // Similar to heredoc but with single string
                        let mut temp_file = tempfile::NamedTempFile::new()
                            .map_err(|e| format!("Failed to create temporary file: {}", e))?;
                        writeln!(temp_file, "{}", content)
                            .map_err(|e| format!("Failed to write here-string: {}", e))?;
                        self.execute_with_redirection(command, *fd, temp_file.into_file(), mode)
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

                execute_command(&mut right_cmd)
            }
            ASTNode::Background { command } => {
                let mut cmd = build_command(command)?;
                cmd.spawn().map_err(|e| e.to_string())?;
                Ok(ExitStatus::from_raw(0))
            }
            ASTNode::LogicalAnd { left, right } => {
                if self.execute(left)?.success() {
                    self.execute(right)
                } else {
                    Ok(ExitStatus::from_raw(0))
                }
            }
            ASTNode::LogicalOr { left, right } => {
                if !self.execute(left)?.success() {
                    self.execute(right)
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
                self.execute(left)?;
                self.execute(right)
            }
            #[allow(unreachable_patterns)]
            _ => {
                Err("Unsupported ASTNode".to_string())
            }
        }
    }

    fn read_variable_name(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, String> {
        let mut var_name = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                var_name.push(c);
                chars.next();
            } else {
                break;
            }
        }
        Ok(var_name)
    }

    fn expand_variables(&self, arg: &str) -> Result<String, String> {
        let mut result = String::new();
        let mut chars = arg.chars().peekable();
        
        while let Some(c) = chars.next() {
            match c {
                // ONLY handle variable expansion, no escape processing
                '$' => {
                    let var_name = self.read_variable_name(&mut chars)?;
                    let value = self.environment.get(&var_name)
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    result.push_str(value);
                }
                // Pass through all other characters unchanged
                c => result.push(c),
            }
        }
        Ok(result)
    }

    fn handle_redirection(&self, fd: i32, file: std::fs::File) -> Result<i32, String> {
        use std::os::fd::IntoRawFd;
        // Save the original file descriptor
        let old_fd = unsafe { libc::dup(fd) };
        if old_fd == -1 {
            return Err("Failed to duplicate file descriptor".to_string());
        }

        // Perform the redirection
        unsafe {
            let file_fd = file.into_raw_fd();
            if libc::dup2(file_fd, fd) == -1 {
                libc::close(old_fd);
                libc::close(file_fd);
                return Err(format!("Failed to redirect file descriptor {}", fd));
            }
            libc::close(file_fd);
        }

        Ok(old_fd)
    }

    fn restore_fd(&self, fd: i32, old_fd: i32) -> Result<(), String> {
        unsafe {
            if libc::dup2(old_fd, fd) == -1 {
                libc::close(old_fd);
                return Err("Failed to restore file descriptor".to_string());
            }
            libc::close(old_fd);
        }
        Ok(())
    }

    fn execute_with_redirection(&mut self, command: &ASTNode, fd: i32, file: std::fs::File, _mode: &RedirectMode) -> Result<ExitStatus, String> {
        // Save original fd
        let old_fd = self.handle_redirection(fd, file)?;

        // Execute command (builtin or external)
        let result = self.execute(command);

        // Restore original fd
        self.restore_fd(fd, old_fd)?;

        result
    }

    fn execute_with_fd_duplication(&mut self, command: &ASTNode, fd: i32, target_fd: i32) -> Result<ExitStatus, String> {
        // Save the original file descriptor
        let old_fd = unsafe { libc::dup(fd) };
        if old_fd == -1 {
            return Err("Failed to duplicate original file descriptor".to_string());
        }

        // Perform the duplication
        unsafe {
            if libc::dup2(target_fd, fd) == -1 {
                libc::close(old_fd);
                return Err("Failed to duplicate target file descriptor".to_string());
            }
        }

        // Execute the command
        let result = self.execute(command);

        // Restore the original file descriptor
        unsafe {
            if libc::dup2(old_fd, fd) == -1 {
                libc::close(old_fd);
                return Err("Failed to restore original file descriptor".to_string());
            }
            libc::close(old_fd);
        }

        result
    }
}

fn build_command(node: &ASTNode) -> Result<std::process::Command, String> {
    match node {
        ASTNode::Command { name, args } => {
            let paths = std::env::var("PATH").unwrap_or_default();
            if let Some(cmd_path) = search_cmd(name, &paths) {
                let mut cmd = std::process::Command::new(&cmd_path);
                // Set program name to be the command as typed by user
                cmd.arg0(name);
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