use tempfile::NamedTempFile;
use std::fs;
use crate::{
    executor::Executor,
    parser::parse,
    lexer::lex,
};

#[test]
fn test_pipeline_features() -> Result<(), String> {
    let mut executor = Executor::new();
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Basic pipeline
    let cmd = format!("echo hello | grep o > {}", path);
    let _ = executor.execute(&parse(&lex(&cmd))?);
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "hello");

    // Multiple pipes
    let cmd = format!("echo hello | tr 'a-z' 'A-Z' | grep O > {}", path);
    let _ = executor.execute(&parse(&lex(&cmd))?);
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "HELLO");

    // Pipeline with redirection
    let temp_file2 = NamedTempFile::new().unwrap();
    let path2 = temp_file2.path().to_str().unwrap();
    let cmd = format!("echo hello | tee {} | grep o > {}", path, path2);
    let _ = executor.execute(&parse(&lex(&cmd))?);
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "hello");
    assert_eq!(fs::read_to_string(path2).unwrap().trim(), "hello");

    Ok(())
}

#[test]
fn test_pipeline_errors() -> Result<(), String> {
    let mut executor = Executor::new();

    // Missing command after pipe - should be a parsing error
    let result = parse(&lex("echo hello |"));
    assert_eq!(result.is_err(), true, "Expected parsing error for 'echo hello |'");

    // Missing command before pipe - should be a parsing error
    let result = parse(&lex("| echo hello"));
    assert_eq!(result.is_err(), true, "Expected parsing error for '| echo hello'");

    // Command not found in pipeline - parsing should succeed, execution should fail
    let parsed_commands = parse(&lex("echo hello | nonexistentcmd"))?;
    let result = executor.execute(&parsed_commands);
    assert_eq!(result.is_err(), true, "Expected execution error for 'echo hello | nonexistentcmd'");

    Ok(())
}
