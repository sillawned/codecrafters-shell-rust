use std::process::{ExitStatus, Stdio};
use std::os::unix::process::{ExitStatusExt, CommandExt};
use nix::unistd::{fork, pipe, dup2, close, ForkResult, read as nix_read};
use nix::sys::wait::{waitpid, WaitStatus};
use std::os::unix::io::{AsRawFd, FromRawFd};
use crate::ast::{ASTNode, RedirectMode, RedirectTarget};
use crate::lexer::lex;
use crate::parser::parse;
use crate::types::ShellError;
use crate::word::{Word, WordPart};

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write; // Added Write trait for file operations
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct Executor {
    last_status: i32,
    environment: HashMap<String, String>,
    current_dir: PathBuf,
    aliases: HashMap<String, String>,
    functions: HashMap<String, ASTNode>,
    history_file: Option<PathBuf>,
}

fn execute_command_process(cmd: &mut std::process::Command) -> Result<ExitStatus, ShellError> {
    // Set up process group
    unsafe {
        cmd.pre_exec(|| {
            Ok(())
        });
    }

    match cmd.status() {
        Ok(status) => Ok(status),
        Err(e) => Err(ShellError::IoError(e))
    }
}

impl Executor {
    pub fn new() -> Self {
        Self {
            last_status: 0,
            environment: std::env::vars().collect(),
            current_dir: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            aliases: HashMap::new(),
            functions: HashMap::new(),
            history_file: None,
        }
    }
    
    fn new_for_command_substitution(&self) -> Self {
        Self {
            last_status: 0,
            environment: self.environment.clone(),
            current_dir: self.current_dir.clone(),
            aliases: self.aliases.clone(),
            functions: self.functions.clone(),
            history_file: self.history_file.clone(),
        }
    }

    pub fn execute(&mut self, node: &ASTNode) -> Result<ExitStatus, ShellError> {
        self.execute_internal(node, None, None)
    }

    fn execute_internal(&mut self, node: &ASTNode, piped_stdin_fd: Option<i32>, _piped_stdout_fd: Option<i32>) -> Result<ExitStatus, ShellError> {
        match node {
            ASTNode::Command { name, args } => {
                let processed_name_str = self.expand_word_to_string(name)?;
                let expanded_args_str: Vec<String> = args.iter()
                    .map(|arg_word| self.expand_word_to_string(arg_word))
                    .collect::<Result<_, _>>()?;
                
                #[cfg(debug_assertions)]
                eprintln!("[Executor] Command: {}, Args: {:?}", processed_name_str, expanded_args_str);

                // Determine if it's a builtin or external command and execute
                let execution_result: Result<ExitStatus, ShellError> = if crate::utils::is_builtin(&processed_name_str) {
                    match crate::builtins::execute_builtin(&processed_name_str, &expanded_args_str, &mut self.environment, &mut self.current_dir) {
                        Ok(()) => Ok(ExitStatus::from_raw(0)),
                        Err(e) => {
                            eprintln!("{}", e); // Builtin errors go to stderr
                            Ok(ExitStatus::from_raw(1)) // Default error code for builtins
                        }
                    }
                } else if let Some(cmd_path) = crate::utils::search_cmd(&processed_name_str, &self.environment.get("PATH").cloned().unwrap_or_default()) {
                    let mut cmd = std::process::Command::new(cmd_path);
                    cmd.arg0(&processed_name_str);
                    cmd.args(&expanded_args_str);
                    cmd.current_dir(&self.current_dir);
                    cmd.envs(&self.environment);

                    if let Some(stdin_fd) = piped_stdin_fd {
                        // Safety: FromRawFd is unsafe. Ensure stdin_fd is a valid, open file descriptor.
                        cmd.stdin(unsafe { Stdio::from_raw_fd(stdin_fd) });
                    }
                    // Stdout/Stderr are inherited by default unless redirected by ASTNode::Redirect or ASTNode::Pipe
                    execute_command_process(&mut cmd)
                } else {
                    eprintln!("{}: command not found", processed_name_str);
                    Err(ShellError::CommandNotFound(processed_name_str))
                };

                // Update last_status based on the execution result
                match execution_result {
                    Ok(status) => {
                        self.last_status = status.code().unwrap_or(1);
                        Ok(status)
                    }
                    Err(err) => {
                        self.last_status = match err {
                            ShellError::CommandNotFound(_) => 127,
                            _ => 1, // General error status
                        };
                        Err(err)
                    }
                }
            }
            ASTNode::Redirect { command, fd, target, mode } => {
                let target_path_str: String;
                let mut temp_heredoc_file: Option<tempfile::NamedTempFile> = None;

                let target_file_object: File = match target {
                    RedirectTarget::File(target_word) => {
                        target_path_str = self.expand_word_to_string(target_word)?;
                        if let Some(parent) = Path::new(&target_path_str).parent() {
                            if !parent.as_os_str().is_empty() {
                                std::fs::create_dir_all(parent).map_err(ShellError::IoError)?;
                            }
                        }
                        OpenOptions::new()
                            .write(true)
                            .create(true)
                            .append(matches!(mode, RedirectMode::Append))
                            .truncate(matches!(mode, RedirectMode::Overwrite))
                            .open(&target_path_str)
                            .map_err(ShellError::IoError)?
                    }
                    RedirectTarget::Descriptor(target_fd_num) => {
                        return self.execute_with_fd_duplication(command, *fd, *target_fd_num, piped_stdin_fd);
                    }
                    RedirectTarget::HereDoc(content) => {
                        let mut file = tempfile::NamedTempFile::new().map_err(ShellError::IoError)?;
                        file.write_all(content.as_bytes()).map_err(ShellError::IoError)?;
                        file.flush().map_err(ShellError::IoError)?;
                        let reopened_file = file.reopen().map_err(ShellError::IoError)?;
                        temp_heredoc_file = Some(file);
                        reopened_file
                    }
                    RedirectTarget::HereString(content) => {
                        let mut file = tempfile::NamedTempFile::new().map_err(ShellError::IoError)?;
                        writeln!(file, "{}", content).map_err(ShellError::IoError)?;
                        file.flush().map_err(ShellError::IoError)?;
                        let reopened_file = file.reopen().map_err(ShellError::IoError)?;
                        temp_heredoc_file = Some(file);
                        reopened_file
                    }
                };

                let result = self.execute_with_redirection_to_file(command, *fd, target_file_object, piped_stdin_fd);
                
                drop(temp_heredoc_file);
                result
            }
            ASTNode::Pipe { left, right } => {
                let (pipe_read_fd, pipe_write_fd) = pipe().map_err(|e| ShellError::NixError(e, "pipe creation failed".to_string()))?;

                match unsafe { fork() } {
                    Ok(ForkResult::Parent { child, .. }) => {
                        //close(pipe_write_fd).map_err(|e| ShellError::NixError(e, "pipe_write_fd close failed in parent".to_string()))?;
                        
                        let right_status_res = self.execute_internal(right, Some(pipe_read_fd), None);
                        
                        // Update last_status based on the rightmost command's result, which defines pipeline status
                        match &right_status_res {
                            Ok(status) => self.last_status = status.code().unwrap_or(1),
                            Err(e) => self.last_status = match e {
                                ShellError::CommandNotFound(_) => 127,
                                _ => 1,
                            },
                        }
                        
                        
                        // close(pipe_read_fd).map_err(|e| ShellError::NixError(e, "pipe_read_fd close failed in parent after right exec".to_string()))?;
                        
                        match waitpid(Some(child), None) {
                            Ok(WaitStatus::Exited(_status_left, _)) => {
                                // self.last_status is already set by the rightmost command.
                                // _status_left could be used for pipefail logic.
                            }
                            Ok(WaitStatus::Signaled(_, signal, _)) => {
                                eprintln!("Left pipe command killed by signal: {:?}", signal);
                                // If pipefail, this might alter self.last_status.
                            }
                            Ok(WaitStatus::Stopped(_, _)) => { /* Handle stopped */ }
                            Ok(WaitStatus::Continued(_)) => { /* Handle continued */ }
                            Ok(WaitStatus::PtraceEvent(_, _, _)) => { /* Handle ptrace event */ }
                            Ok(WaitStatus::PtraceSyscall(_)) => { /* Handle ptrace syscall */ }
                            Ok(WaitStatus::StillAlive) => { /* Handle still alive */ }
                            Err(e) => {
                                eprintln!("Error waiting for left pipe command: {:?}", e);
                            }
                        }
                        right_status_res
                    }
                    Ok(ForkResult::Child) => {
                        //close(pipe_read_fd).map_err(|e| ShellError::NixError(e, "pipe_read_fd close failed in left child".to_string()))?;
                        dup2(pipe_write_fd, libc::STDOUT_FILENO).map_err(|e| ShellError::NixError(e, "dup2 stdout to pipe_write_fd failed in left child".to_string()))?;
                        //close(pipe_write_fd).map_err(|e| ShellError::NixError(e, "pipe_write_fd close failed in left child after dup2".to_string()))?;
                        
                        if let Some(initial_stdin) = piped_stdin_fd {
                            dup2(initial_stdin, libc::STDIN_FILENO).map_err(|e| ShellError::NixError(e, "dup2 initial_stdin to stdin failed in left child".to_string()))?;
                            //close(initial_stdin).map_err(|e| ShellError::NixError(e, "initial_stdin close failed in left child after dup2".to_string()))?;
                        }

                        match self.execute_internal(left, None, None) {
                            Ok(status) => std::process::exit(status.code().unwrap_or(0)),
                            Err(e) => {
                                eprintln!("Shell error in left pipe command: {:?}", e);
                                std::process::exit(127);
                            }
                        }
                    }
                    Err(e) => return Err(ShellError::NixError(e, "Fork failed for pipeline".to_string())),
                }
            }
            ASTNode::Background { command } => {
                match unsafe { fork() } {
                    Ok(ForkResult::Parent { child: _ }) => {
                        self.last_status = 0; // Successfully launched background job
                        Ok(ExitStatus::from_raw(0))
                    }
                    Ok(ForkResult::Child) => {
                        if let Some(bg_stdin_fd) = piped_stdin_fd {
                            dup2(bg_stdin_fd, libc::STDIN_FILENO).map_err(|e| {
                                eprintln!("Error redirecting stdin for background process: {:?}", e);
                                ShellError::NixError(e, "dup2 stdin for background process failed".to_string())
                            })?;
                            close(bg_stdin_fd).ok();
                        } else {
                            let dev_null = File::open("/dev/null").map_err(ShellError::IoError)?;
                            dup2(dev_null.as_raw_fd(), libc::STDIN_FILENO).map_err(|e| {
                                eprintln!("Error redirecting stdin to /dev/null for background process: {:?}", e);
                                ShellError::NixError(e, "dup2 stdin to /dev/null for background process failed".to_string())
                            })?;
                        }

                        match self.execute_internal(command, None, None) {
                            Ok(status) => std::process::exit(status.code().unwrap_or(0)),
                            Err(e) => {
                                eprintln!("Error in background command: {:?}", e);
                                std::process::exit(127);
                            }
                        }
                    }
                    Err(e) => {
                        self.last_status = 1; // Or a specific error code for fork failure
                        return Err(ShellError::NixError(e, "Fork failed for background command".to_string()));
                    }
                }
            }
            ASTNode::LogicalAnd { left, right } => {
                let left_res = self.execute_internal(left, piped_stdin_fd, None);
                match left_res {
                    Ok(status) => {
                        self.last_status = status.code().unwrap_or(1);
                        if status.success() {
                            // Left succeeded, execute right
                            let right_res = self.execute_internal(right, None, None);
                            match &right_res {
                                Ok(r_status) => self.last_status = r_status.code().unwrap_or(1),
                                Err(r_err) => self.last_status = match r_err {
                                    ShellError::CommandNotFound(_) => 127,
                                    _ => 1,
                                },
                            }
                            right_res
                        } else {
                            Ok(status) // Left failed, short-circuit
                        }
                    }
                    Err(e) => { // Error executing left command itself
                        self.last_status = match e {
                            ShellError::CommandNotFound(_) => 127,
                            _ => 1,
                        };
                        Err(e)
                    }
                }
            }
            ASTNode::LogicalOr { left, right } => {
                let left_res = self.execute_internal(left, piped_stdin_fd, None);
                match left_res {
                    Ok(status) => {
                        self.last_status = status.code().unwrap_or(1);
                        if !status.success() {
                            // Left failed, execute right
                            let right_res = self.execute_internal(right, None, None);
                            match &right_res {
                                Ok(r_status) => self.last_status = r_status.code().unwrap_or(1),
                                Err(r_err) => self.last_status = match r_err {
                                    ShellError::CommandNotFound(_) => 127,
                                    _ => 1,
                                },
                            }
                            right_res
                        } else {
                            Ok(status) // Left succeeded, short-circuit
                        }
                    }
                    Err(e) => { // Error executing left command itself
                        self.last_status = match e {
                            ShellError::CommandNotFound(_) => 127,
                            _ => 1,
                        };
                        Err(e)
                    }
                }
            }
            ASTNode::Subshell { command } => {
                match unsafe { fork() } {
                    Ok(ForkResult::Parent { child, .. }) => {
                        match waitpid(child, None) {
                            Ok(WaitStatus::Exited(_, status_code)) => {
                                self.last_status = status_code;
                                Ok(ExitStatus::from_raw(status_code))
                            }
                            Ok(WaitStatus::Signaled(_, signal, _)) => {
                                let exit_code = 128 + signal as i32;
                                self.last_status = exit_code;
                                eprintln!("Subshell killed by signal: {:?}", signal);
                                Ok(ExitStatus::from_raw(exit_code))
                            }
                            Ok(WaitStatus::Stopped(_, _)) => { self.last_status = 1; Ok(ExitStatus::from_raw(1)) } // Or specific mapping
                            Ok(WaitStatus::Continued(_)) => { self.last_status = 0; Ok(ExitStatus::from_raw(0)) } // Or last status before stop
                            Ok(WaitStatus::PtraceEvent(_, _, _)) => { self.last_status = 1; Ok(ExitStatus::from_raw(1)) }
                            Ok(WaitStatus::PtraceSyscall(_)) => { self.last_status = 1; Ok(ExitStatus::from_raw(1)) }
                            Ok(WaitStatus::StillAlive) => { self.last_status = 1; Ok(ExitStatus::from_raw(1)) } // Should not happen with None options
                            Err(e) => {
                                self.last_status = 1; // Or a specific error code for waitpid failure
                                Err(ShellError::NixError(e, "waitpid failed for subshell".to_string()))
                            }
                        }
                    }
                    Ok(ForkResult::Child) => {
                        if let Some(initial_stdin) = piped_stdin_fd {
                            dup2(initial_stdin, libc::STDIN_FILENO).map_err(|e| {
                                let err_msg = format!("dup2 initial_stdin to stdin failed in subshell: {:?}", e);
                                eprintln!("{}", err_msg);
                                ShellError::NixError(e, "dup2 initial_stdin to stdin failed in subshell".to_string())
                            })?;
                            close(initial_stdin).ok();
                        }

                        match self.execute_internal(command, None, None) {
                            Ok(status) => std::process::exit(status.code().unwrap_or(0)),
                            Err(e) => {
                                eprintln!("Shell error in subshell command: {:?}", e);
                                std::process::exit(127);
                            }
                        }
                    }
                    Err(e) => return Err(ShellError::NixError(e, "Fork failed for subshell".to_string())),
                }
            }
            ASTNode::Semicolon { left, right } => {
                // Execute left command; its status updates self.last_status
                let left_result = self.execute_internal(left, piped_stdin_fd, None);
                match left_result {
                    Ok(s) => self.last_status = s.code().unwrap_or(1),
                    Err(ref e) => {
                        // Optionally print error from left side, e.g., if not handled by main loop
                        // eprintln!("Error in left part of semicolon sequence: {:?}", e);
                        self.last_status = match e {
                            ShellError::CommandNotFound(_) => 127,
                            _ => 1,
                        };
                        // If the error from left_result should halt execution (e.g. critical fork failure),
                        // it should have been propagated by `?` if used. Here we continue to `right`.
                    }
                };

                // Execute right command; its status is the result of the semicolon sequence and updates self.last_status
                let right_res = self.execute_internal(right, None, None);
                match &right_res {
                    Ok(s) => self.last_status = s.code().unwrap_or(1),
                    Err(e) => self.last_status = match e {
                        ShellError::CommandNotFound(_) => 127,
                        _ => 1,
                    },
                }
                right_res
            }
            ASTNode::Assignment { var, val } => {
                let value_str = self.expand_word_to_string(val)?; // If this fails, Err is propagated, last_status not set to 0 here.
                self.environment.insert(var.clone(), value_str);
                self.last_status = 0; // Successful assignment
                Ok(ExitStatus::from_raw(0))
            }
            ASTNode::CommandSubstitution { .. } => Err(ShellError::InternalError("CommandSubstitution node should not be directly executed.".to_string())),
        }
    }

    fn read_variable_name(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, ShellError> {
        let mut var_name = String::new();
        if chars.peek() == Some(&'{') {
            chars.next();
            while let Some(&c) = chars.peek() {
                if c == '}' {
                    chars.next();
                    break;
                }
                if c.is_alphanumeric() || c == '_' {
                    var_name.push(c);
                    chars.next();
                } else {
                    return Err(ShellError::InvalidSyntax("Invalid character in variable name within ${}".to_string()));
                }
            }
            if var_name.is_empty() && chars.peek() != Some(&'}') {
                return Err(ShellError::InvalidSyntax("Empty or malformed variable name in ${...}".to_string()));
            }
        } else {
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    var_name.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
        }
        if var_name.is_empty() && chars.peek().map_or(false, |c| !c.is_alphanumeric() && *c != '_') {
        }
        Ok(var_name)
    }

    fn expand_word_to_string(&mut self, word: &Word) -> Result<String, ShellError> {
        let mut result = String::new();
        for part in &word.parts {
            match part {
                WordPart::Simple(s) => {
                    if s.starts_with('`') && s.ends_with('`') && s.len() >= 2 {
                        let command_str = &s[1..s.len()-1];
                        if command_str.is_empty() {
                        } else {
                            let mut sub_executor = self.new_for_command_substitution();
                            
                            let tokens = lex(command_str);
                            if tokens.is_empty() || tokens.iter().all(|t| matches!(t, crate::lexer::Token::Space | crate::lexer::Token::NewLine)) {
                            } else {
                                let ast_node = parse(&tokens).map_err(ShellError::ParseError)?;
                                
                                let (pipe_read_fd, pipe_write_fd) = pipe().map_err(|e| ShellError::NixError(e, "pipe creation for command substitution failed".to_string()))?;

                                match unsafe { fork() } {
                                    Ok(ForkResult::Parent { child, .. }) => {
                                        close(pipe_write_fd).map_err(|e| ShellError::NixError(e, "closing write end of pipe in parent (cmd subst) failed".to_string()))?;
                                        
                                        let mut output_bytes = Vec::new();
                                        let mut read_buffer = [0u8; 1024];
                                        loop {
                                            match nix_read(pipe_read_fd, &mut read_buffer) {
                                                Ok(0) => break,
                                                Ok(n) => output_bytes.extend_from_slice(&read_buffer[..n]),
                                                Err(nix::errno::Errno::EINTR) => continue,
                                                Err(e) => {
                                                    close(pipe_read_fd).ok();
                                                    waitpid(child, None).ok();
                                                    return Err(ShellError::NixError(e, "reading from pipe (cmd subst) failed".to_string()));
                                                }
                                            }
                                        }
                                        close(pipe_read_fd).map_err(|e| ShellError::NixError(e, "closing read end of pipe in parent (cmd subst) failed".to_string()))?;

                                        match waitpid(child, None) {
                                            Ok(WaitStatus::Exited(_, status_code)) => {
                                                if status_code != 0 {
                                                    // Optionally log or handle non-zero exit status of command substitution
                                                }
                                            }
                                            Ok(WaitStatus::Signaled(_, signal, _)) => {
                                                // Optionally log or handle signal termination
                                                eprintln!("[Cmd Subst Parent] Child killed by signal: {:?}", signal);
                                            }
                                            Ok(other_status) => {
                                                 eprintln!("[Cmd Subst Parent] Child exited with other status: {:?}", other_status);
                                            }
                                            Err(e) => {
                                                eprintln!("[Cmd Subst Parent] Error waiting for child: {:?}", e);
                                                // This might warrant returning an error, depending on desired shell behavior
                                            }
                                        }

                                        let mut captured_output = String::from_utf8(output_bytes)
                                            .map_err(|e| ShellError::InternalError(format!("Command substitution output not valid UTF-8: {}", e)))?;
                                        
                                        while captured_output.ends_with('\n') {
                                            captured_output.pop();
                                        }
                                        result.push_str(&captured_output);
                                    }
                                    Ok(ForkResult::Child) => {
                                        close(pipe_read_fd).map_err(|e| ShellError::NixError(e, "closing read end of pipe in child (cmd subst) failed".to_string()))?;
                                        dup2(pipe_write_fd, libc::STDOUT_FILENO).map_err(|e| {
                                            let err_msg = format!("dup2 stdout to pipe in child (cmd subst) failed: {:?}", e);
                                            eprintln!("{}", err_msg);
                                            ShellError::NixError(e, "dup2 stdout to pipe in child (cmd subst) failed".to_string())
                                        })?;
                                        let dev_null_err = OpenOptions::new().write(true).open("/dev/null").map_err(ShellError::IoError)?;
                                        dup2(dev_null_err.as_raw_fd(), libc::STDERR_FILENO).map_err(|e| {
                                            // Log error, but don't necessarily exit, stderr redirection is best-effort
                                            eprintln!("Failed to redirect stderr for command substitution: {:?}", e);
                                            ShellError::NixError(e, "dup2 stderr for command substitution failed".to_string())
                                        }).ok(); // .ok() to ignore the error if stderr redirection fails

                                        close(pipe_write_fd).map_err(|e| ShellError::NixError(e, "closing write end of pipe in child (cmd subst) after dup2 failed".to_string()))?;
                                        
                                        match sub_executor.execute_internal(&ast_node, None, None) {
                                            Ok(status) => std::process::exit(status.code().unwrap_or(0)),
                                            Err(e) => {
                                                // Error from command execution inside substitution.
                                                // This error won't be directly visible in parent unless stderr is captured.
                                                // The exit code will be caught by parent.
                                                eprintln!("[Cmd Subst Child Err] Shell error: {:?}", e);
                                                std::process::exit(127); // General error for command failure in substitution
                                            }
                                        }
                                    }
                                    Err(e) => return Err(ShellError::NixError(e, "Fork failed for command substitution".to_string())),
                                }
                            }
                        }
                    } else {
                        result.push_str(&self.expand_string_segment(s, false)?);
                    }
                }
                WordPart::SingleQuoted(text) => {
                    result.push_str(text);
                }
                WordPart::DoubleQuoted(text) => {
                    result.push_str(&self.expand_string_segment(text, true)?);
                }
            }
        }
        Ok(result)
    }

    fn expand_string_segment(&mut self, segment: &str, is_double_quoted: bool) -> Result<String, ShellError> {
        let mut expanded_segment = String::new();
        let mut chars = segment.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(next_char) = chars.next() {
                    if is_double_quoted {
                        match next_char {
                            'n' => expanded_segment.push('\n'),
                            't' => expanded_segment.push('\t'),
                            '\\' => expanded_segment.push('\\'),
                            '"' => expanded_segment.push('"'),
                            '$' => expanded_segment.push('$'),
                            '`' => expanded_segment.push('`'),
                            _ => {
                                expanded_segment.push('\\');
                                expanded_segment.push(next_char);
                            }
                        }
                    } else {
                        expanded_segment.push(next_char);
                    }
                } else {
                    expanded_segment.push('\\');
                }
            } else if ch == '$' {
                if chars.peek() == Some(&'?') {
                    chars.next();
                    expanded_segment.push_str(&self.last_status.to_string());
                    continue;
                }
                if chars.peek() == Some(&'$') {
                    chars.next();
                    expanded_segment.push_str(&nix::unistd::getpid().to_string());
                    continue;
                }

                let var_name = self.read_variable_name(&mut chars)?;
                if var_name.is_empty() {
                    expanded_segment.push('$');
                } else {
                    let value = self.environment.get(&var_name).cloned().unwrap_or_default();
                    expanded_segment.push_str(&value);
                }
            } else {
                expanded_segment.push(ch);
            }
        }
        Ok(expanded_segment)
    }

    fn execute_with_redirection_to_file(&mut self, command_node: &ASTNode, fd_to_redirect: i32, target_file: File, piped_stdin_fd: Option<i32>) -> Result<ExitStatus, ShellError> {
        let target_file_raw_fd = target_file.as_raw_fd();
        
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                // Parent closes its copy of target_file_raw_fd implicitly when target_file is dropped.
                match waitpid(child, None) {
                    Ok(WaitStatus::Exited(_, status_code)) => {
                        self.last_status = status_code;
                        Ok(ExitStatus::from_raw(status_code))
                    }
                    Ok(WaitStatus::Signaled(_, signal, _)) => {
                        let exit_code = 128 + signal as i32;
                        self.last_status = exit_code;
                        eprintln!("Redirected command killed by signal: {:?}", signal);
                        Ok(ExitStatus::from_raw(exit_code))
                    }
                    Ok(other_status) => { // Catch-all for other statuses like Stopped, Continued, etc.
                        eprintln!("Redirected command exited with unexpected status: {:?}", other_status);
                        self.last_status = 1; // Generic error
                        Ok(ExitStatus::from_raw(1))
                    }
                    Err(e) => {
                        self.last_status = 1; // Or a specific error code for waitpid failure
                        Err(ShellError::NixError(e, "waitpid failed for redirected command".to_string()))
                    }
                }
            }
            Ok(ForkResult::Child) => {
                dup2(target_file_raw_fd, fd_to_redirect).map_err(|e| {
                    let err_msg = format!("dup2 target_file to fd {} failed in child: {:?}", fd_to_redirect, e);
                    eprintln!("{}", err_msg);
                    ShellError::NixError(e, "dup2 target_file failed".to_string())
                })?;

                let child_stdin_fd = if fd_to_redirect == libc::STDIN_FILENO {
                    None
                } else {
                    piped_stdin_fd
                };
                
                if let Some(stdin_val_fd) = child_stdin_fd {
                     dup2(stdin_val_fd, libc::STDIN_FILENO).map_err(|e| {
                        let err_msg = format!("dup2 piped_stdin_fd to STDIN_FILENO failed in child (redir): {:?}", e);
                        eprintln!("{}", err_msg);
                        ShellError::NixError(e, "dup2 piped_stdin_fd to STDIN_FILENO failed (redir)".to_string())
                     })?;
                     if stdin_val_fd != libc::STDIN_FILENO { close(stdin_val_fd).ok(); }
                }

                match self.execute_internal(command_node, None, None) {
                    Ok(status) => std::process::exit(status.code().unwrap_or(0)),
                    Err(e) => {
                        eprintln!("Shell error in redirected command: {:?}", e);
                        std::process::exit(127);
                    }
                }
            }
            Err(e) => Err(ShellError::NixError(e, "Fork failed for redirection".to_string())),
        }
    }

    fn execute_with_fd_duplication(&mut self, command_node: &ASTNode, fd_to_redirect: i32, target_fd: i32, piped_stdin_fd: Option<i32>) -> Result<ExitStatus, ShellError> {
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                match waitpid(child, None) {
                    Ok(WaitStatus::Exited(_, status_code)) => {
                        self.last_status = status_code;
                        Ok(ExitStatus::from_raw(status_code))
                    }
                    Ok(WaitStatus::Signaled(_, signal, _)) => {
                        let exit_code = 128 + signal as i32;
                        self.last_status = exit_code;
                        eprintln!("FD duplicated command killed by signal: {:?}", signal);
                        Ok(ExitStatus::from_raw(exit_code))
                    }
                    Ok(other_status) => { // Catch-all for other statuses
                        eprintln!("FD duplicated command exited with unexpected status: {:?}", other_status);
                        self.last_status = 1; // Generic error
                        Ok(ExitStatus::from_raw(1))
                    }
                    Err(e) => {
                        self.last_status = 1; // Or a specific error code for waitpid failure
                        Err(ShellError::NixError(e, "waitpid failed for fd duplicated command".to_string()))
                    }
                }
            }
            Ok(ForkResult::Child) => {
                dup2(target_fd, fd_to_redirect).map_err(|e| {
                    let err_msg = format!("dup2 target_fd {} to fd {} failed in child: {:?}", target_fd, fd_to_redirect, e);
                    eprintln!("{}", err_msg);
                    ShellError::NixError(e, "dup2 target_fd to fd_to_redirect failed".to_string())
                })?;

                let child_stdin_fd = if fd_to_redirect == libc::STDIN_FILENO {
                    None
                } else {
                    piped_stdin_fd
                };
                
                if let Some(stdin_val_fd) = child_stdin_fd {
                     dup2(stdin_val_fd, libc::STDIN_FILENO).map_err(|e| {
                        let err_msg = format!("dup2 piped_stdin_fd to STDIN_FILENO failed in child (fd-dup): {:?}", e);
                        eprintln!("{}", err_msg);
                        ShellError::NixError(e, "dup2 piped_stdin_fd to STDIN_FILENO failed (fd-dup)".to_string())
                     })?;
                     if stdin_val_fd != libc::STDIN_FILENO { close(stdin_val_fd).ok(); }
                }

                match self.execute_internal(command_node, None, None) {
                    Ok(status) => std::process::exit(status.code().unwrap_or(0)),
                    Err(e) => {
                        eprintln!("Shell error in fd duplicated command: {:?}", e);
                        std::process::exit(127);
                    }
                }
            }
            Err(e) => Err(ShellError::NixError(e, "Fork failed for fd duplication".to_string())),
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}