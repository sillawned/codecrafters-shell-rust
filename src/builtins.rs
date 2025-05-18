use std::env;
use std::path::Path;

pub const BUILTINS: [&str; 15] = [
    "exit", "echo", "type", "pwd", "cd", "alias", "unalias", "export", "unset", "history", "jobs", "fg", "bg", "kill", "wait"
];

pub fn expand_tilde(path: &str) -> Result<String, String> {
    if path == "~" {
        env::var("HOME").map_err(|_| "HOME not set".to_string())
    } else if path.starts_with("~/") {
        env::var("HOME")
            .map(|home| format!("{}{}", home, &path[1..]))
            .map_err(|_| "HOME not set".to_string())
    } else {
        Ok(path.to_string())
    }
}

// Updated signature to include environment and current_dir
pub fn execute_builtin(
    name: &str, 
    args: &[String], 
    environment: &mut std::collections::HashMap<String, String>, 
    current_dir: &mut std::path::PathBuf
) -> Result<(), String> {
    match name {
        "exit" => {
            let code = args.get(0)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            std::process::exit(code & 255); // POSIX requires 8-bit exit codes
        },
        "echo" => {
            use std::io::{stdout, Write}; 
            let mut out_handle = stdout(); 
            if args.is_empty() {
                if let Err(e) = writeln!(out_handle) {
                    return Err(format!("echo: write error: {}", e));
                }
            } else {
                let output = args.join(" ");
                if let Err(e) = writeln!(out_handle, "{}", output) {
                    return Err(format!("echo: write error: {}", e));
                }
            }
            if let Err(e) = out_handle.flush() {
                return Err(format!("echo: flush error: {}", e));
            }
            Ok(())
        }
        "pwd" => {
            use std::io::{stdout, Write};
            let mut out_handle = stdout();
            if let Err(e) = writeln!(out_handle, "{}", current_dir.display()) { // Use current_dir from params
                return Err(format!("pwd: write error: {}", e));
            }
            if let Err(e) = out_handle.flush() {
                return Err(format!("pwd: flush error: {}", e));
            }
            Ok(())
        }
        "cd" => {
            let raw_path = if args.is_empty() {
                "~".to_string()
            } else if args[0] == "-" {
                let prev = environment.get("OLDPWD").cloned().ok_or("OLDPWD not set")?;
                let curr_display = current_dir.display().to_string(); // current_dir is PathBuf
                environment.insert("OLDPWD".to_string(), curr_display);
                
                let new_path = Path::new(&prev);
                env::set_current_dir(new_path).map_err(|e| format!("cd: {}: {}", prev, e))?;
                *current_dir = new_path.to_path_buf(); // Update mutable current_dir

                use std::io::{stdout, Write};
                let mut out_handle = stdout();
                if let Err(e) = writeln!(out_handle, "{}", prev) {
                    return Err(format!("cd: write error: {}", e));
                }
                if let Err(e) = out_handle.flush() {
                    return Err(format!("cd: flush error: {}", e));
                }
                return Ok(())
            } else {
                args[0].clone()
            };

            let path_str = expand_tilde(&raw_path)?;
            let new_path = Path::new(&path_str);
            
            // Attempt to canonicalize the path *before* changing the directory.
            // This path should be relative to the current_dir *before* the cd operation.
            let canonical_path = if new_path.is_absolute() {
                new_path.canonicalize().map_err(|e| format!("cd: {}: {}", path_str, e))?
            } else {
                current_dir.join(new_path).canonicalize().map_err(|e| format!("cd: {}: {}....", path_str, e))?
            };

            let curr_display = current_dir.display().to_string(); // current_dir is PathBuf
            environment.insert("OLDPWD".to_string(), curr_display);
            
            env::set_current_dir(&canonical_path) // Use the canonical_path
                .map_err(|e| format!("cd: {}: {}", canonical_path.display(), e))?; // Error with canonical_path
            *current_dir = canonical_path; // Update mutable current_dir with the already canonicalized path
            Ok(())
        },
        "type" => {
            use std::io::{stdout, Write};
            let mut out_handle = stdout();

            if args.is_empty() {
                // According to POSIX, `type` without arguments is unspecified.
                // Bash and zsh print usage. Let's return an error/usage.
                return Err("type: usage: type name [name ...]".to_string());
            }

            let mut all_found = true; // Track if all names are found

            for name_to_check in args {
                let mut current_name_found = false;
                // Check if it's a builtin
                if BUILTINS.contains(&name_to_check.as_str()) {
                    if let Err(e) = writeln!(out_handle, "{} is a shell builtin", name_to_check) {
                        return Err(format!("type: write error: {}", e));
                    }
                    current_name_found = true;
                }

                // Check if it's a command on PATH
                // The `search_cmd` utility needs the PATH string from the environment.
                let path_env_var = environment.get("PATH").map_or_else(|| "".to_string(), |s| s.clone());
                if let Some(path_str) = crate::utils::search_cmd(name_to_check, &path_env_var) {
                    if let Err(e) = writeln!(out_handle, "{} is {}", name_to_check, path_str) {
                        return Err(format!("type: write error: {}", e));
                    }
                    current_name_found = true;
                }
                
                if !current_name_found {
                    if let Err(e) = writeln!(out_handle, "type: {}: not found", name_to_check) {
                        return Err(format!("type: write error: {}", e));
                    }
                    all_found = false; // If any name is not found, set all_found to false
                }
            }

            if let Err(e) = out_handle.flush() {
                return Err(format!("type: flush error: {}", e));
            }

            if all_found {
                Ok(())
            } else {
                // POSIX specifies exit status > 0 if any name is not found.
                // Common practice is 1.
                Err("type: one or more arguments not found".to_string()) // This error message won't be printed by shell, exit status matters.
                                                                      // The executor will handle mapping this Err to an ExitStatus::Failure(1)
            }
        },
        "export" => {
            for arg in args {
                if let Some((var, val)) = arg.split_once('=') {
                    environment.insert(var.to_string(), val.to_string());
                } else {
                    // If just `export VAR`, mark VAR for export if shell differentiates
                    // between shell variables and environment variables. Here, just ensure it exists.
                    if environment.get(arg).is_none() {
                        // Behavior for `export VAR` without `=` can vary.
                        // Some shells export it with an empty value if not set,
                        // some require it to be set first, some do nothing.
                        // For now, let's insert with empty string if not present.
                        // environment.insert(arg.to_string(), "".to_string());
                    } 
                    // If it exists, it's already in `environment` and thus "exported"
                    // in the context of this shell and its children.
                }
            }
            Ok(())
        },
        "unset" => {
            for arg in args {
                environment.remove(arg);
            }
            Ok(())
        },
        // TODO: Implement other builtins: alias, unalias, history, jobs, fg, bg, kill, wait
        _ => Err(format!("{}: builtin not implemented", name))
    }
}