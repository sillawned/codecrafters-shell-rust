#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

fn main() {
        
    let stdin = io::stdin();
    let paths = std::env::var( "PATH").unwrap();

    let mut input = String::new();

    // Wait for user input
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
            
        let _ = stdin.read_line(&mut input);
        let cmd_line: Vec<_> = input.trim().split(" ").collect();

        match cmd_line[0] {
            "exit" => {
                exit(cmd_line[1].parse().unwrap());
            }
            "echo" => {
                print!("{}\n", &cmd_line[1..].join(" "));
            }
            "type" => {
                match cmd_line[1] {
                    "echo" => {
                        print!("echo is a shell builtin\n");
                    }
                    "type" => {
                        print!("type is a shell builtin\n");
                    }
                    "exit" => {
                        print!("exit is a shell builtin\n");
                    }
                    _ => {
                        let mut found = false;
                        for path in paths.split(":") {
                            let cmd_path = format!("{}/{}", path, cmd_line[1]);
                            if std::path::Path::new(&cmd_path).exists() {
                                print!("{} is {}\n", &cmd_line[1], &cmd_path);
                                found = true;
                                break;
                            } 
                        }
                        if !found {
                            println!("{}: not found", &cmd_line[1]);
                        }
                    }
                }
            }
            _ => {
                print!("{}: command not found\n", &cmd_line[0]);
            }
        }
        input.clear();
    }
}
