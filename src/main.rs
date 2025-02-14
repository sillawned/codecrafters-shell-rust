use std::string::String;
use std::io::{self, Write};
use std::env;
use std::process::ExitStatus;
use std::os::unix::process::ExitStatusExt;
use libc;

pub mod ast;
pub mod builtins;
pub mod executor;
pub mod parser;
pub mod tokenizer;
pub mod utils;
pub mod processor;
pub mod lexer;

fn main() {
    // Set up signal handlers like bash
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_IGN);
        libc::signal(libc::SIGQUIT, libc::SIG_IGN);
        libc::signal(libc::SIGTSTP, libc::SIG_IGN);
    }

    let stdin = io::stdin();
    let mut input = String::new();
    let mut last_status = ExitStatus::from_raw(0);

    // Set initial environment variables
    if env::var("PWD").is_err() {
        if let Ok(pwd) = std::env::current_dir() {
            env::set_var("PWD", pwd.to_string_lossy().as_ref());
        }
    }

    loop {
        // Update PS1 for prompt
        let prompt = env::var("PS1").unwrap_or_else(|_| "$ ".to_string());
        print!("{}", prompt);
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

        let mut executor = executor::Executor::new();
        match executor.execute(&ast) {
            Ok(status) => last_status = status,
            Err(e) => {
                eprintln!("{}", e);
                last_status = ExitStatus::from_raw(1);
            }
        }

        // Update status variable like bash
        env::set_var("?", last_status.code().unwrap_or(0).to_string());

        input.clear();
    }
}
