use crate::types::QuoteType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessingMode {
    Command,    // For command names 
    Argument,   // For command arguments
    Path,       // For file paths
    Literal,    // For literal strings
}

pub fn process_text(text: &str, mode: ProcessingMode) -> String {
    match mode {
        ProcessingMode::Command | ProcessingMode::Argument => {
            // Pattern match the string content
            match text {
                // Single-quoted string: preserve everything literally
                s if s.starts_with('\'') && s.ends_with('\'') => s[1..s.len()-1].to_string(),
                
                // Double-quoted string: preserve internal quotes
                s if s.starts_with('"') && s.ends_with('"') => {
                    let inner = &s[1..s.len()-1];
                    inner.replace("\\\"", "\"")
                         .replace("\\$", "$")
                         .replace("\\`", "`")
                         .replace("\\\\", "\\")
                },
                
                // Unquoted: handle escapes
                s => s.chars().fold(String::new(), |mut acc, c| {
                    match c {
                        '\\' => (),  // Skip backslash, next char will be literal
                        _ => acc.push(c)
                    }
                    acc
                })
            }
        },
        _ => text.to_string()  // Path and Literal modes preserve everything
    }
}

// Add a helper function to determine the quote type
pub fn get_quote_type(s: &str) -> QuoteType {
    match s {
        s if s.starts_with('\'') && s.ends_with('\'') => QuoteType::Single,
        s if s.starts_with('"') && s.ends_with('"') => QuoteType::Double,
        s if s.contains('\\') => QuoteType::Escaped,
        _ => QuoteType::None
    }
}