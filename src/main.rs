use std::{
    env,
    process::ExitStatus,
    os::unix::process::ExitStatusExt,
};

use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::config::Config;

pub mod ast;
pub mod builtins;
pub mod executor;
pub mod parser;
pub mod utils;
pub mod processor;
pub mod lexer;
pub mod types;
pub mod word;
pub mod completion;

fn main() {
    // Set up signal handlers like bash
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_IGN);
        libc::signal(libc::SIGQUIT, libc::SIG_IGN);
        libc::signal(libc::SIGTSTP, libc::SIG_IGN);
    }

    let completer = completion::Completer::new();
    let config = Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .history_ignore_space(true)
        .history_ignore_dups(true).unwrap()
        .build();
    let mut rl = Editor::with_config(config).unwrap();
    rl.set_helper(Some(completer));
    
    #[allow(unused_assignments)]
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
        
        match rl.readline(&prompt) {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                if line.trim().is_empty() {
                    continue;
                }
                
                let tokens = lexer::lex(&line);
                #[cfg(debug_assertions)]
                println!("Lexer tokens: {:?}", tokens);

                #[allow(unused_assignments)]
                let ast = match parser::parse(&tokens) {
                    Ok(ast) => ast,
                    Err(e) => {
                        eprintln!("Parse error: {}", e);
                        last_status = ExitStatus::from_raw(2);
                        continue;
                    }
                };
                #[cfg(debug_assertions)]
                println!("AST: {:?}", &ast);

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
            },
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests;
