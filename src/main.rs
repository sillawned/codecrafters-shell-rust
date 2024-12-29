#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;
use std::string::String;

const BUILTINS: [&str; 3] = ["exit", "echo", "type"];

fn search_cmd(cmd: &str, paths: &str) -> Option<String> {
    for path in paths.split(":") {
        let cmd_path = format!("{}/{}", path, cmd);
        if std::path::Path::new(&cmd_path).exists() {
            return Some(cmd_path);
        }
    }
    None
}

//https://dustinknopoff.dev/articles/minishell/
fn tokenize(input: &str) -> Vec<String> {
    let args: Vec<_> = input.split_whitespace().map(|s| s.to_string()).collect();
    args
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

        //println!("{:?}", tokens);

        match &*tokens[0] {
            "exit" => {
                exit(tokens[1].parse().unwrap());
            }
            "echo" => {
                println!("{}", &tokens[1..].join(" "));
            }
            "type" => {
                if BUILTINS.contains(&&*tokens[1]) {
                    println!("{} is a shell builtin", &tokens[1]);
                } else {
                    if let Some(cmd_path) = search_cmd(&*tokens[0], &paths) {
                        println!("{} is {}", &tokens[1], cmd_path);
                    } else {
                        println!("{}: not found", &*tokens[1]);
                    }
                }
            }
            _ => {
                if let Some(cmd_path) = search_cmd(&*tokens[0], &paths) {
                    let mut cmd = std::process::Command::new(cmd_path);
                    let _ = cmd.args(&tokens[1..]).status();
                } else {
                    print!("{}: command not found\n", &*tokens[0]);
                }
            }
        }
        input.clear();
    }
}
