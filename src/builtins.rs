use std::env;
use crate::utils::search_cmd;

pub const BUILTINS: [&str; 15] = [
    "exit", "echo", "type", "pwd", "cd", "alias", "unalias", "export", "unset", "history", "jobs", "fg", "bg", "kill", "wait"
];

pub fn execute_builtin(name: &str, args: &[String]) -> Result<(), String> {
    match name {
        "exit" => std::process::exit(args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0)),
        "echo" => {
            let mut output = String::new();
            for arg in args {
                let mut chars = arg.chars().peekable();
                while let Some(c) = chars.next() {
                    if c == '\\' {
                        if let Some(&escaped_char) = chars.peek() {
                            match escaped_char {
                                'n' => output.push('\n'),
                                't' => output.push('\t'),
                                '\\' => output.push('\\'),
                                '"' => output.push('"'),
                                '\'' => output.push('\''),
                                _ => {
                                    output.push(c);
                                    output.push(escaped_char);
                                }
                            }
                            chars.next(); // Consume the escaped character
                        } else {
                            output.push(c);
                        }
                    } else {
                        output.push(c);
                    }
                }
                output.push(' ');
            }
            println!("{}", output.trim_end());
            Ok(())
        }
        "pwd" => {
            println!("{}", env::current_dir().unwrap().display());
            Ok(())
        }
        "cd" => {
            let path = args.get(0).map_or(env::var("HOME").unwrap(), |s| s.clone());
            env::set_current_dir(&path).map_err(|e| e.to_string())
        }
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