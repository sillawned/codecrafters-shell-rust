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
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    
    // Strip outer quotes if they exist
    if text.starts_with('\'') && text.ends_with('\'') ||
       text.starts_with('"') && text.ends_with('"') {
        chars.next(); // Skip opening quote
        let text = text[1..text.len()-1].to_string();
        chars = text.chars().peekable();
    }
    
    while let Some(c) = chars.next() {
        match (c, in_single_quote, in_double_quote) {
            ('\'', false, false) => {
                in_single_quote = true;
                match mode {
                    ProcessingMode::Literal => result.push(c),
                    _ => {}
                }
            }
            ('\'', true, false) => {
                in_single_quote = false;
                match mode {
                    ProcessingMode::Literal => result.push(c),
                    _ => {}
                }
            }
            ('"', false, false) => {
                in_double_quote = true;
                match mode {
                    ProcessingMode::Literal => result.push(c),
                    _ => {}
                }
            }
            ('"', false, true) => {
                in_double_quote = false;
                match mode {
                    ProcessingMode::Literal => result.push(c),
                    _ => {}
                }
            }
            ('\\', false, true) => {
                if let Some(next) = chars.next() {
                    match next {
                        '$' | '`' | '"' | '\\' => result.push(next),
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        _ => {
                            if !in_double_quote {
                                result.push('\\');
                            }
                            result.push(next);
                        }
                    }
                }
            }
            ('\\', false, false) => {
                if let Some(next) = chars.next() {
                    match next {
                        ' ' | '\'' | '"' | '\\' | '$' | '`' => result.push(next),
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        _ => {
                            result.push('\\');
                            result.push(next);
                        }
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
