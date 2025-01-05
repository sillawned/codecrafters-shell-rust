#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;
use std::string::String;
use std::path::Path;

const BUILTINS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];
//const CONTROL_OPERATORS: [&str; 12] = ["\n", "&&", "||", "&", ";", ";;", ";&", ";;&", "|", "|&", "(", ")"];
//const META_CHARACTERS: [&str; 10] = [" ", "\t", "\n", "|", "&", ";", "(", ")", "<", ">"];

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
    let input = input.trim().to_string();

    // Input can contain double and single quotes
    // We need to handle them from left to right
    // For example, if the input is echo "single 'quote' inside double quote", we should get ["echo", "single 'quote' inside double quote"] as tokens.
    // And if the input is echo 'double "quote" inside single quote', we should get ["echo", "double "quote" inside single quote"] as tokens.
    let tokens = unquote(input);

    tokens
}

fn unquote(input: String) -> Vec<String> {
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut was_escaped = false;
    let mut token = String::new();
    let mut tokens = Vec::new();

    for c in input.chars() {
        match c {
            ' ' => {
                if was_escaped{
                    was_escaped = !was_escaped;
                    continue;
                } else if in_double_quote || in_single_quote {
                    token.push(c);
                } else {
                    if !token.is_empty() {
                        tokens.push(token.clone());
                        token.clear();
                    }
                }
            }
            '\\' => {
                // Enclosed by double quotes, the backslash retains its special meaning when followed by "$", "`", """, "\", or newline
                if in_double_quote {
                    if let Some(next_char) = input.chars().nth(input.chars().position(|x| x == c).unwrap() + 1) {
                        if next_char == '$' || next_char == '`' || next_char == '"' || next_char == '\\' || next_char == '\n' {
                            token.push(next_char);
                        } else {
                            token.push(c);
                        }
                    }
                } else if in_single_quote {
                    token.push(c);
                } else {
                    if let Some(next_char) = input.chars().nth(input.chars().position(|x| x == c).unwrap() + 1) {
                        if next_char == '\n' {
                            continue;
                        } else {
                            token.push(next_char);
                            was_escaped = true;
                        }
                    }
                }
            }
            '"' => {
                if in_single_quote {
                    token.push(c);
                } else {
                    in_double_quote = !in_double_quote;
                }
            }
            '\'' => {
                if in_double_quote {
                    token.push(c);
                } else {
                    in_single_quote = !in_single_quote;
                }
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
