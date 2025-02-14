use std::env;
use std::path::Path;
use crate::utils::search_cmd;

pub const BUILTINS: [&str; 15] = [
    "exit", "echo", "type", "pwd", "cd", "alias", "unalias", "export", "unset", "history", "jobs", "fg", "bg", "kill", "wait"
];

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
            let path = if args.is_empty() {
                env::var("HOME").map_err(|_| "HOME not set")?
            } else {
                args[0].clone()
            };
            
            if path == "-" {
                // Handle cd - to previous directory
                let prev = env::var("OLDPWD").map_err(|_| "OLDPWD not set")?;
                let curr = env::current_dir().map_err(|e| e.to_string())?;
                env::set_var("OLDPWD", curr.to_string_lossy().to_string());
                env::set_current_dir(&prev).map_err(|e| e.to_string())?;
                println!("{}", prev);
                Ok(())
            } else {
                let curr = env::current_dir().map_err(|e| e.to_string())?;
                env::set_var("OLDPWD", curr.to_string_lossy().to_string());
                env::set_current_dir(Path::new(&path)).map_err(|e| e.to_string())
            }
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
            // Implement export functionality
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