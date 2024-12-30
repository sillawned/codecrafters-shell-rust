#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;
use std::string::String;
use std::path::Path;

const BUILTINS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];

fn search_cmd(cmd: &str, paths: &str) -> Option<String> {
    for path in paths.split(":") {
        let cmd_path = format!("{}/{}", path, cmd);
        if Path::new(&cmd_path).exists() {
            return Some(cmd_path);
        }
    }
    None
}

// https://dustinknopoff.dev/articles/minishell/
fn tokenize(input: &str) -> Vec<String> {
    // Split the input into tokens

    // Before we split the input into tokens, we need to handle single quotes.
    // Single quotes are used to prevent the shell from interpreting special characters.
    // We need to remove the single quotes and treat the content inside as a single token.
    // For example, if the input is echo 'Hello, World!', we should get ["echo", "Hello, World!"] as tokens.
    let input = input.trim().to_string();
    let mut tokens = Vec::new();
    let mut in_single_quote = false;
    let mut token = String::new();
    for c in input.chars() {
        match c {
            ' ' => {
                if in_single_quote {
                    token.push(c);
                } else {
                    if !token.is_empty() {
                        tokens.push(token.clone());
                        token.clear();
                    }
                }
            }
            '\'' => {
                // A single quote cannot be enclosed in another single quote.
                // So we can safely toggle the in_single_quote flag.
                in_single_quote = !in_single_quote;
            }
            _ => {
                token.push(c);
            }
        }
    }
    // The last word may not be followed by a space.
    if !token.is_empty() {
       tokens.push(token.clone());
       token.clear();
    }

    tokens
}

fn main() {
        
    let stdin = io::stdin();
    let paths = std::env::var( "PATH").unwrap();

    let mut input = String::new();

    // Wait for user input
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
            
        let _ = stdin.read_line(&mut input);
        let tokens: Vec<String> = tokenize(&input);

        #[cfg(debug_assertions)]
        println!("{:?}", tokens);

        match &*tokens[0] {
            "type" => {
                if BUILTINS.contains(&&*tokens[1]) {
                    println!("{} is a shell builtin", &tokens[1]);
                } else {
                    if let Some(cmd_path) = search_cmd(&*tokens[1], &paths) {
                        println!("{} is {}", &tokens[1], cmd_path);
                    } else {
                        eprintln!("{}: not found", &*tokens[1]);
                    }
                }
            }
            "exit" => {
                exit(tokens[1].parse().unwrap());
            }
            "echo" => {
                println!("{}", &tokens[1..].join(" "));
            }
            "pwd" => {
                let path = std::env::current_dir().unwrap();
                println!("{}", path.display());
            }
            "cd" => {
                let path = if vec!["", "~"].contains(&&*tokens[1]) {
                    std::env::var("HOME").unwrap()
                } else {
                    tokens[1].clone()
                };
                let _ = std::env::set_current_dir(&path).unwrap_or_else(|error| {
                    if error.kind() == io::ErrorKind::NotFound {
                        eprintln!("cd: {}: No such file or directory", path);
                    } else {
                        eprintln!("cd: {}: {}", path, error);
                    }
                });
            }
            _ => {
                if let Some(cmd_path) = search_cmd(&*tokens[0], &paths) {
                    let mut cmd = std::process::Command::new(cmd_path);
                    let _ = cmd.args(&tokens[1..]).status();
                } else {
                    eprint!("{}: command not found\n", &*tokens[0]);
                }
            }
        }
        input.clear();
    }
}
