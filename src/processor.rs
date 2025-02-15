use crate::types::QuoteType;
use crate::word::{Word, WordPart};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessingMode {
    Command,    // For command names 
    Argument,   // For command arguments
    Path,       // For file paths
    Literal,    // For literal strings
}

pub fn process_text(text: &str, mode: ProcessingMode) -> String {
    match mode {
        ProcessingMode::Command => text.to_string(),
        ProcessingMode::Argument => {
            let mut word = Word::new();
            let mut chars = text.chars().peekable();
            let mut current = String::new();
            let mut in_quote = None;
            let mut escaped = false;

            while let Some(c) = chars.next() {
                match (c, escaped, in_quote) {
                    (c, true, _) => {
                        current.push(c);
                        escaped = false;
                    },
                    ('\\', false, Some('"')) | ('\\', false, None) => {
                        escaped = true;
                        current.push('\\');
                    },
                    ('\'', false, None) => {
                        if !current.is_empty() {
                            word.add_part(WordPart::Simple(current));
                            current = String::new();
                        }
                        in_quote = Some('\'');
                    },
                    ('\'', false, Some('\'')) => {
                        word.add_part(WordPart::SingleQuoted(current));
                        current = String::new();
                        in_quote = None;
                    },
                    ('"', false, None) => {
                        if !current.is_empty() {
                            word.add_part(WordPart::Simple(current));
                            current = String::new();
                        }
                        in_quote = Some('"');
                    },
                    ('"', false, Some('"')) => {
                        word.add_part(WordPart::DoubleQuoted(current));
                        current = String::new();
                        in_quote = None;
                    },
                    (c, _, _) => current.push(c),
                }
            }

            if !current.is_empty() {
                match in_quote {
                    Some('\'') => word.add_part(WordPart::SingleQuoted(current)),
                    Some('"') => word.add_part(WordPart::DoubleQuoted(current)),
                    _ => word.add_part(WordPart::Simple(current)),
                }
            }

            word.to_string()
        },
        _ => text.to_string()
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