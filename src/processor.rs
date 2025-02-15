#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessingMode {
    Command,    // For command names (preserve most characters)
    Argument,   // For command arguments (handle escapes)
    Path,       // For file paths (preserve slashes and dots)
    Literal,    // For literal strings (no processing)
}

pub fn process_text(text: &str, mode: ProcessingMode) -> String {
    let mut result = String::new();
    
    // Strip outer quotes if they exist
    let text = match mode {
        ProcessingMode::Argument => {
            if (text.starts_with('\'') && text.ends_with('\'')) ||
               (text.starts_with('"') && text.ends_with('"')) {
                &text[1..text.len()-1]
            } else {
                text
            }
        },
        _ => text
    };
    
    let mut chars = text.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    
    while let Some(c) = chars.next() {
        match (c, in_single_quote, in_double_quote) {
            ('\'', false, false) => {
                in_single_quote = true;
                match mode {
                    ProcessingMode::Command => result.push(c),
                    ProcessingMode::Literal => result.push(c),
                    _ => {}
                }
            }
            ('\'', true, false) => {
                in_single_quote = false;
                match mode {
                    ProcessingMode::Command => result.push(c),
                    ProcessingMode::Literal => result.push(c),
                    _ => {}
                }
            }
            ('"', false, false) => {
                in_double_quote = true;
                match mode {
                    ProcessingMode::Command => result.push(c),
                    ProcessingMode::Literal => result.push(c),
                    _ => {}
                }
            }
            ('"', false, true) => {
                in_double_quote = false;
                match mode {
                    ProcessingMode::Command => result.push(c),
                    ProcessingMode::Literal => result.push(c),
                    _ => {}
                }
            }
            ('\\', _, _) => {
                if let Some(next) = chars.next() {
                    match (next, mode) {
                        ('\'', ProcessingMode::Command) |
                        ('"', ProcessingMode::Command) => {
                            result.push(next); // Keep quotes in command names
                        },
                        ('\'', _) | ('"', _) => {
                            result.push('\\');
                            result.push(next);
                        },
                        ('n', _) => result.push('\n'),
                        ('t', _) => result.push('\t'),
                        ('r', _) => result.push('\r'),
                        (c, _) => result.push(c),
                    }
                }
            }
            (c, true, _) => result.push(c),  // In single quotes, preserve everything
            (c, _, true) => result.push(c),  // In double quotes, preserve most things
            (c, false, false) => result.push(c),  // Outside quotes, normal character
        }
    }
    result
}
