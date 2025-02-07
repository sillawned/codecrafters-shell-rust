use std::string::String;
use std::io::{self, Write};

pub mod ast;
pub mod builtins;
pub mod executor;
pub mod parser;
pub mod tokenizer;
pub mod utils;

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();

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
                input.clear();
                continue;
            }
        };

        if let Err(e) = executor::execute(&ast) {
            eprintln!("{}", e);
        }

        input.clear();
    }
}
