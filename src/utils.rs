use std::path::Path;

pub fn search_cmd(cmd: &str, paths: &str) -> Option<String> {
    for path in paths.split(":") {
        let cmd_path = format!("{}/{}", path, cmd);
        if Path::new(&cmd_path).exists() {
            return Some(cmd_path);
        }
    }
    None
}

pub fn is_builtin(cmd: &str) -> bool {
    crate::builtins::BUILTINS.contains(&cmd)
}