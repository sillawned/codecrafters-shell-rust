use std::string::String;
use std::io::{self, Write};
use std::env;
use std::process::ExitStatus;
use std::os::unix::process::ExitStatusExt;

pub mod ast;
pub mod builtins;
pub mod executor;
pub mod parser;
pub mod tokenizer;
pub mod utils;
pub mod processor;

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();

    // Set up signal handlers for SIGINT, SIGTSTP, etc.
    let mut last_status = ExitStatus::from_raw(0);

    // Wait for user input
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        if stdin.read_line(&mut input).is_err() {
            eprintln!("Error reading input");
            continue;
        }

        let tokens = tokenizer::tokenize(&input);
        #[cfg(debug_assertions)]
        println!("Tokens: {:?}", tokens);

        let ast = match parser::parse(&tokens) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                last_status = ExitStatus::from_raw(2);
                input.clear();
                continue;
            }
        };

        match executor::execute(&ast) {
            Ok(status) => last_status = status,
            Err(e) => {
                eprintln!("{}", e);
                last_status = ExitStatus::from_raw(1);
            }
        }

        // Make last exit status available to scripts
        env::set_var("?", last_status.code().unwrap_or(0).to_string());

        input.clear();
    }
}
