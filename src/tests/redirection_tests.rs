use std::fs;
use std::io::Read;
use tempfile::NamedTempFile;
use crate::{
    executor::Executor,
    parser::parse,
    lexer::lex,
};

#[test]
fn test_basic_redirection() -> Result<(), String> {
    let mut executor = Executor::new();
    let temp_file = NamedTempFile::new().unwrap();
    let path_buf = temp_file.path().to_path_buf(); // Use PathBuf for owned path
    let path_str = path_buf.to_str().unwrap();

    // Test output redirection
    let cmd = format!("echo hello > {}", path_str);
    let tokens = lex(&cmd);
    let ast = parse(&tokens)?;
    executor.execute(&ast);

    // Ensure I/O operations are flushed. 
    // This might be specific to how your Executor is implemented.
    // If your executor spawns a child process, ensure it waits for it to exit.
    // Forcing a sync_all on the temp_file before reading might help if it's a direct write.
    temp_file.as_file().sync_all().map_err(|e| format!("Failed to sync temp file: {}", e))?;
    // Adding a small delay can also help diagnose timing issues, 
    // though it's not a robust solution for production tests.
    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut content = String::new();
    // Re-open the file by path to ensure we are reading the latest state from disk
    match fs::File::open(&path_buf) { // Open using PathBuf
        Ok(mut file) => {
            if let Err(e) = file.read_to_string(&mut content) {
                return Err(format!("Failed to read temp file at '{:?}': {}", path_buf, e));
            }
        }
        Err(e) => {
            return Err(format!("Failed to open temp file at '{:?}': {}", path_buf, e));
        }
    };
    
    assert_eq!(content.trim(), "hello");

    // temp_file is dropped here, and the associated file is deleted.
    Ok(())
}

// #[test]
// fn test_append_redirection() -> Result<(), String> {
//     let mut executor = Executor::new();
//     let temp_file = NamedTempFile::new().unwrap();
//     let path = temp_file.path().to_str().unwrap();
// 
//     // First write
//     let cmd = format!("echo first > {}", path);
//     executor.execute(&parse(&lex(&cmd))?)?;
// 
//     // Append
//     let cmd = format!("echo second >> {}", path);
//     executor.execute(&parse(&lex(&cmd))?)?;
// 
//     let mut content = String::new();
//     fs::File::open(path).unwrap().read_to_string(&mut content).unwrap();
//     assert_eq!(content.trim(), "first\nsecond");
// 
//     Ok(())
// }

#[test]
fn test_stderr_redirection() -> Result<(), String> {
    let mut executor = Executor::new();
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let cmd = format!("ls /nonexistent 2> {}", path);
    executor.execute(&parse(&lex(&cmd))?);

    let mut content = String::new();
    fs::File::open(path).unwrap().read_to_string(&mut content).unwrap();
    assert!(content.contains("No such file or directory"));

    Ok(())
}

// #[test]
// fn test_file_descriptor_duplication() -> Result<(), String> {
//     let mut executor = Executor::new();
//     let temp_file = NamedTempFile::new().unwrap();
//     let path = temp_file.path().to_str().unwrap();
// 
//     // Redirect both stdout and stderr to the same file
//     let cmd = format!("ls /nonexistent > {} 2>&1", path);
//     executor.execute(&parse(&lex(&cmd))?)?;
// 
//     let mut content = String::new();
//     fs::File::open(path).unwrap().read_to_string(&mut content).unwrap();
//     assert!(content.contains("No such file or directory"));
// 
//     Ok(())
// }

// #[test]
// fn test_here_document() -> Result<(), String> {
//     let mut executor = Executor::new();
//     let temp_file = NamedTempFile::new().unwrap();
//     let path = temp_file.path().to_str().unwrap();
// 
//     let cmd = format!("cat << EOF > {}\nline1\nline2\nEOF", path);
//     executor.execute(&parse(&lex(&cmd))?)?;
// 
//     let mut content = String::new();
//     fs::File::open(path).unwrap().read_to_string(&mut content).unwrap();
//     assert_eq!(content.trim(), "line1\nline2");
// 
//     Ok(())
// }

// #[test]
// fn test_stdout_redirection() -> Result<(), String> {
//     let mut executor = Executor::new();
//     let temp_file = NamedTempFile::new().unwrap();
//     let path = temp_file.path().to_str().unwrap();
// 
//     // Basic redirection
//     let cmd = format!("echo hello > {}", path);
//     executor.execute(&parse(&lex(&cmd))?)?;
//     assert_eq!(fs::read_to_string(path).unwrap().trim(), "hello");
// 
//     // Redirection with spaces
//     let cmd = format!("echo    hello    >     {}", path);
//     executor.execute(&parse(&lex(&cmd))?)?;
//     assert_eq!(fs::read_to_string(path).unwrap().trim(), "hello");
// 
//     // Multiple redirections (last one wins)
//     let temp_file2 = NamedTempFile::new().unwrap();
//     let path2 = temp_file2.path().to_str().unwrap();
//     let cmd = format!("echo hello > {} > {}", path, path2);
//     executor.execute(&parse(&lex(&cmd))?)?;
//     assert_eq!(fs::read_to_string(path2).unwrap().trim(), "hello");
// 
//     Ok(())
// }

// #[test]
// fn test_append_mode() -> Result<(), String> {
//     let mut executor = Executor::new();
//     let temp_file = NamedTempFile::new().unwrap();
//     let path = temp_file.path().to_str().unwrap();
// 
//     // First write
//     let cmd = format!("echo first > {}", path);
//     executor.execute(&parse(&lex(&cmd))?)?;
// 
//     // Append
//     let cmd = format!("echo second >> {}", path);
//     executor.execute(&parse(&lex(&cmd))?)?;
// 
//     // Append with stderr
//     let cmd = format!("ls /nonexistent 2>> {}", path);
//     executor.execute(&parse(&lex(&cmd))?)?;
// 
//     let content = fs::read_to_string(path).unwrap();
//     assert!(content.contains("first"));
//     assert!(content.contains("second"));
//     assert!(content.contains("No such file"));
// 
//     Ok(())
// }

// :#[test]
// :fn test_redirection_errors() -> Result<(), String> {
// :    let mut executor = Executor::new();
// :
// :    // Missing filename
// :    let result = executor.execute(&parse(&lex("echo hello >"))?);
// :    assert!(result.is_err());
// :
// :    // Invalid file descriptor
// :    let result = executor.execute(&parse(&lex("echo hello 3> file"))?);
// :    assert!(result.is_err());
// :
// :    // Permission denied (try to write to /dev/null/file)
// :    let result = executor.execute(&parse(&lex("echo hello > /dev/null/file"))?);
// :    assert!(result.is_err());
// :
// :    Ok(())
// :}
