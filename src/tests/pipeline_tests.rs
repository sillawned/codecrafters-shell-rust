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
    executor.execute(&parse(&lex(&cmd))?)?;
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "hello");

    // Multiple pipes
    let cmd = format!("echo hello | tr 'a-z' 'A-Z' | grep O > {}", path);
    executor.execute(&parse(&lex(&cmd))?)?;
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "HELLO");

    // Pipeline with builtin commands
    let cmd = format!("echo \"a\\nb\\nc\" | grep b > {}", path);
    executor.execute(&parse(&lex(&cmd))?)?;
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "b");

    // Pipeline with redirection
    let temp_file2 = NamedTempFile::new().unwrap();
    let path2 = temp_file2.path().to_str().unwrap();
    let cmd = format!("echo hello | tee {} | grep o > {}", path, path2);
    executor.execute(&parse(&lex(&cmd))?)?;
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "hello");
    assert_eq!(fs::read_to_string(path2).unwrap().trim(), "hello");

    Ok(())
}

#[test]
fn test_pipeline_errors() -> Result<(), String> {
    let mut executor = Executor::new();

    // Missing command after pipe
    let result = executor.execute(&parse(&lex("echo hello |"))?);
    assert!(result.is_err());

    // Missing command before pipe
    let result = executor.execute(&parse(&lex("| echo hello"))?);
    assert!(result.is_err());

    // Command not found in pipeline
    let result = executor.execute(&parse(&lex("echo hello | nonexistentcmd"))?);
    assert!(result.is_err());

    Ok(())
}
