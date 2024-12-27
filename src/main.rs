#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

fn main() {
        
    let mut input = String::new();
    let stdin = io::stdin();
    let mut result: i32 = 0;
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    while let Ok(_) = stdin.read_line(&mut input) {
        let cmd_line: Vec<_> = input.trim().split(" ").collect();
        match cmd_line[0] {
            "exit" => {
                result = cmd_line[1].parse().unwrap();
                break;
            },
            "echo" => {
                print!("{}\n", &cmd_line[1..].join(" "));
            }
            _ => {
                print!("{}: command not found\n", &cmd_line[0]);
            }
            
        }
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
    }

    exit(result);
}
