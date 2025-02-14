#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessingMode {
    Command,    // For command names (preserve most characters)
    Argument,   // For command arguments (handle escapes)
    Path,       // For file paths (preserve slashes and dots)
    Literal,    // For literal strings (no processing)
}

pub fn process_text(text: &str, mode: ProcessingMode) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    let mut in_quotes = false;
    
    while let Some(c) = chars.next() {
        match (mode, c, in_quotes) {
            (_, '\'', _) => {
                in_quotes = !in_quotes;
                match mode {
                    ProcessingMode::Literal => result.push(c),
                    _ => {}
                }
            }
            (ProcessingMode::Path, '\\', false) => {
                if let Some(next) = chars.next() {
                    if next == ' ' || next == '\\' || next == '\'' || next == '"' {
                        result.push(next);
                    } else {
                        result.push('\\');
                        result.push(next);
                    }
                }
            }
            (ProcessingMode::Argument, '\\', false) => {
                if let Some(next) = chars.next() {
                    match next {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' | ' ' | '\'' | '"' => result.push(next),
                        _ => {
                            result.push('\\');
                            result.push(next);
                        }
                    }
                }
            }
            (ProcessingMode::Command, '\\', false) => {
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            }
            (_, c, true) => result.push(c),  // Inside quotes, preserve everything
            (_, c, false) => result.push(c),  // Outside quotes, normal character
        }
    }
    result
}
