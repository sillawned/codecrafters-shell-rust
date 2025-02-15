use std::env;
use std::path::Path;
use crate::utils::search_cmd;

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

pub fn execute_builtin(name: &str, args: &[String]) -> Result<(), String> {
    match name {
        "exit" => {
            let code = args.get(0)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            std::process::exit(code & 255); // POSIX requires 8-bit exit codes
        },
        "echo" => {
            if args.is_empty() {
                println!();
            } else {
                println!("{}", args.join(" "));
            }
            Ok(())
        }
        "pwd" => {
            println!("{}", env::current_dir().unwrap().display());
            Ok(())
        }
        "cd" => {
            let raw_path = if args.is_empty() {
                "~".to_string()
            } else if args[0] == "-" {
                // Handle cd - to previous directory
                let prev = env::var("OLDPWD").map_err(|_| "OLDPWD not set")?;
                let curr = env::current_dir().map_err(|e| e.to_string())?;
                env::set_var("OLDPWD", curr.to_string_lossy().to_string());
                env::set_current_dir(&prev).map_err(|e| e.to_string())?;
                println!("{}", prev);
                return Ok(())
            } else {
                args[0].clone()
            };

            let path = expand_tilde(&raw_path)?;
            let curr = env::current_dir().map_err(|e| e.to_string())?;
            env::set_var("OLDPWD", curr.to_string_lossy().to_string());
            env::set_current_dir(Path::new(&path))
                .map_err(|_| format!("cd: {}: No such file or directory", raw_path))
        },
        "type" => {
            if BUILTINS.contains(&args[0].as_str()) {
                println!("{} is a shell builtin", args[0]);
            } else if let Some(cmd_path) = search_cmd(&args[0], &std::env::var("PATH").unwrap()) {
                println!("{} is {}", args[0], cmd_path);
            } else {
                println!("{}: not found", args[0]);
            }
            Ok(())
        }
        "alias" => {
            // Implement alias functionality
            Ok(())
        }
        "unalias" => {
            // Implement unalias functionality
            Ok(())
        }
        "export" => {
            if args.is_empty() {
                // Print all environment variables
                for (key, value) in env::vars() {
                    println!("declare -x {}=\"{}\"", key, value);
                }
                return Ok(());
            }

            for arg in args {
                if let Some(pos) = arg.find('=') {
                    // Handle NAME=value format
                    let (name, value) = arg.split_at(pos);
                    let value = value.trim_start_matches('=');
                    // Remove surrounding quotes if present
                    let value = value.trim_matches(|c| c == '"' || c == '\'');
                    env::set_var(name, value);
                } else {
                    // Handle NAME format (just mark as exported)
                    if let Ok(value) = env::var(arg) {
                        env::set_var(arg, value);
                    } else {
                        env::set_var(arg, "");
                    }
                }
            }
            Ok(())
        }
        "unset" => {
            // Implement unset functionality
            Ok(())
        }
        "history" => {
            // Implement history functionality
            Ok(())
        }
        "jobs" => {
            // Implement jobs functionality
            Ok(())
        }
        "fg" => {
            // Implement fg functionality
            Ok(())
        }
        "bg" => {
            // Implement bg functionality
            Ok(())
        }
        "kill" => {
            // Implement kill functionality
            Ok(())
        }
        "wait" => {
            // Implement wait functionality
            Ok(())
        }
        _ if name.contains('=') => {
            let parts: Vec<&str> = name.splitn(2, '=').collect();
            if parts.len() == 2 {
                env::set_var(parts[0], parts[1]);
                Ok(())
            } else {
                Err(format!("Invalid assignment: {}", name))
            }
        }
        _ => Err(format!("Unknown builtin: {}", name)),
    }
}