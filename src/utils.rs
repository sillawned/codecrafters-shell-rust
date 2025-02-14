use std::path::Path;

pub fn search_cmd(cmd: &str, paths: &str) -> Option<String> {
    let cmd = cmd.trim();
    
    // If command contains a slash, try it as an absolute path
    if cmd.contains('/') {
        let path = Path::new(cmd);
        if path.exists() {
            return Some(cmd.to_string());
        }
        return None;
    }

    // Split command name from any arguments
    let cmd_name = cmd.split_whitespace().next()?;

    // Search in PATH
    for path in paths.split(':') {
        if path.is_empty() {
            continue;
        }
        let cmd_path = format!("{}/{}", path, cmd_name);
        if Path::new(&cmd_path).exists() {
            return Some(cmd_path);
        }
    }
    None
}

pub fn is_builtin(cmd: &str) -> bool {
    crate::builtins::BUILTINS.contains(&cmd)
}