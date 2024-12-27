#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
        
    // Uncomment this block to pass the first stage
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    let stdin = io::stdin();

    // Wait for user input
    while let Ok(_) = stdin.read_line(&mut input) {
        print!("{}: command not found\n", input.trim());
        print!("$ ");
        io::stdout().flush().unwrap();
    }
}
