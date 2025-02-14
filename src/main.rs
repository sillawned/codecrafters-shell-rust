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
pub mod lexer;

fn main() {
    // Set up signal handlers
    // ctrlc::set_handler(move || {
    //     println!();
    //     io::stdout().flush().unwrap();
    // }).expect("Error setting Ctrl-C handler");

    let stdin = io::stdin();
    let mut input = String::new();
    let mut last_status = ExitStatus::from_raw(0);

    loop {
        // Print prompt and ensure it's displayed immediately
        print!("$ ");
        io::stdout().flush().unwrap();

        input.clear();  // Clear before reading new input
        match stdin.read_line(&mut input) {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!();
                break;
            }
            Ok(_) => {
                // Skip empty lines but still show prompt
                if input.trim().is_empty() {
                    continue;
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        }

        let tokens = lexer::lex(&input);
        #[cfg(debug_assertions)]
        println!("Lexer tokens: {:?}", tokens);

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

        env::set_var("?", last_status.code().unwrap_or(0).to_string());

        input.clear();
    }
}
