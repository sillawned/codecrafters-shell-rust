#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

fn main() {
        
    let mut input = String::new();
    let stdin = io::stdin();
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    while let Ok(_) = stdin.read_line(&mut input) {
        match input.trim() {
            "exit 0" => break,
            _ => {
                print!("{}: command not found\n", input.trim());
            }
            
        }
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
    }

    exit(0);
}
