use tempfile::NamedTempFile;
use std::fs;
use crate::{
    executor::Executor,
    parser::parse,
    lexer::lex,
};

#[test]
fn test_backtick_command_substitution() -> Result<(), String> {
    let mut executor = Executor::new();
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Basic backtick command substitution
    let cmd = format!("echo `echo hello` > {}", path);
    let _ = executor.execute(&parse(&lex(&cmd))?);
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "hello");

    Ok(())
}

// TODO: Add more tests for command substitution:
// - $(command) syntax
// - Nested command substitution
// - Command substitution with pipes and redirections
// - Error handling for invalid commands within substitution
