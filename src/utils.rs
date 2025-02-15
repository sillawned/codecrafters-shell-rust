use std::path::Path;
use std::os::unix::fs::PermissionsExt;

use crate::processor::{process_text, ProcessingMode};

pub fn search_cmd(cmd: &str, paths: &str) -> Option<String> {
    let cmd = if cmd.starts_with('"') || cmd.starts_with('\'') {
        // Process the command name to handle escaped quotes
        let processed = process_text(cmd, ProcessingMode::Command);
        // Strip outer quotes
        let len = processed.len();
        if len >= 2 {
            processed[1..len-1].to_string()
        } else {
            processed
        }
    } else {
        cmd.to_string()
    }.trim().to_string();
    
    // If command contains a slash, use it directly without PATH search
    if cmd.contains('/') {
        let path = Path::new(&cmd);
        if path.exists() && is_executable(path) {
            return Some(cmd.to_string());
        }
        return None;
    }

    // Search in PATH
    for path in paths.split(':') {
        if path.is_empty() {
            continue;
        }
        let cmd_path = format!("{}/{}", path, cmd);
        let cmd_path = Path::new(&cmd_path);
        if cmd_path.exists() && is_executable(cmd_path) {
            return Some(cmd_path.to_string_lossy().into_owned());
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    if let Ok(metadata) = path.metadata() {
        let mode = metadata.permissions().mode();
        return mode & 0o111 != 0; // Check for execute permission
    }
    false
}

pub fn is_builtin(cmd: &str) -> bool {
    crate::builtins::BUILTINS.contains(&cmd)
}