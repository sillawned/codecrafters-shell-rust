use std::fs;
use tempfile::NamedTempFile;
use crate::{
    executor::Executor,
    parser::parse,
    lexer::lex,
};

#[test]
fn test_quoting_rules() -> Result<(), String> {
    let mut executor = Executor::new();
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Single quotes preserve everything literally
    let cmd = format!("echo '  $HOME  \"  \\n  \\t  ' > {}", path);
    let _ = executor.execute(&parse(&lex(&cmd))?);
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "  $HOME  \"  \\n  \\t  ");

    // Double quotes allow variable expansion and some escapes
    std::env::set_var("TESTVAR", "value");
    let cmd = format!("echo \"$TESTVAR \\n \\\"quoted\\\"\" > {}", path);
    let _ = executor.execute(&parse(&lex(&cmd))?);
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "value \n \"quoted\"");

    // Mixed quotes
    let cmd = format!("echo '\"$TESTVAR\"' \"'literal'\" > {}", path);
    let _ = executor.execute(&parse(&lex(&cmd))?);
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "\"$TESTVAR\" 'literal'");

    Ok(())
}

#[test]
fn test_escape_sequences() -> Result<(), String> {
    let mut executor = Executor::new();
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Escape special characters
    let cmd = format!("echo a\\ b\\>c > {}", path);
    let _ = executor.execute(&parse(&lex(&cmd))?);
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "a b>c");

    // Escape sequences in double quotes
    let cmd = format!("echo \"\\n\\t\\r\\\\\" > {}", path);
    let _ = executor.execute(&parse(&lex(&cmd))?);
    assert_eq!(fs::read_to_string(path).unwrap(), "\n\t\r\\\n");

    // Escape sequences in single quotes (preserved literally)
    let cmd = format!("echo '\\n\\t\\r\\\\' > {}", path);
    let _ = executor.execute(&parse(&lex(&cmd))?);
    assert_eq!(fs::read_to_string(path).unwrap().trim(), "\\n\\t\\r\\\\");

    Ok(())
}
