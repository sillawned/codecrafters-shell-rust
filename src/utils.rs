use std::path::Path;
use std::os::unix::fs::PermissionsExt;

pub fn search_cmd(cmd: &str, paths: &str) -> Option<String> {
    let cmd = if cmd.starts_with('"') || cmd.starts_with('\'') {
        // Strip quotes from command name
        let len = cmd.len();
        &cmd[1..len-1]
    } else {
        cmd
    }.trim();
    
    // If command contains a slash, use it directly without PATH search
    if cmd.contains('/') {
        let path = Path::new(cmd);
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