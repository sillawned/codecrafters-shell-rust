#[derive(Debug, Clone)]
pub enum WordPart {
    Simple(String),
    SingleQuoted(String),
    DoubleQuoted(String),
    Escaped(char),
}

#[derive(Debug, Clone)]
pub struct Word {
    parts: Vec<WordPart>,
}

impl Word {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    pub fn add_part(&mut self, part: WordPart) {
        self.parts.push(part);
    }

    pub fn to_string(&self) -> String {
        // TODO: The WordPart::Simple handling needs to be modified
        // 1. Keep backslashes in paths
        // 2. For non-quoted text, preserve backslashes
        // 3. Only process escapes in quoted strings
        self.parts.iter().map(|part| match part {
            WordPart::Simple(s) => {
                // Add handling for escaped characters in unquoted text
                let mut result = String::new();
                let mut chars = s.chars().peekable();
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => {
                            if let Some(&next) = chars.peek() {
                                match next {
                                    // In unquoted text, preserve escaped quotes
                                    '"' | '\'' => {
                                        result.push(next);
                                        chars.next();
                                    },
                                    _ => {
                                        result.push('\\');
                                        result.push(next);
                                        chars.next();
                                    }
                                }
                            }
                        },
                        _ => result.push(c)
                    }
                }
                result
            },
            WordPart::SingleQuoted(s) => s.clone(),
            WordPart::DoubleQuoted(s) => {
                let mut result = String::new();
                let mut chars = s.chars().peekable();
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => {
                            if let Some(&next) = chars.peek() {
                                match next {
                                    // Only escape quote characters in double quotes
                                    '$' | '`' | '"' | '\\' | '\n' => {
                                        result.push(next);
                                        chars.next();  // consume the quote
                                    },
                                    _ => {
                                        // Any other escaped character should remain as-is
                                        result.push('\\');
                                        result.push(next);
                                        chars.next();
                                    }
                                }
                            }
                        },
                        _ => result.push(c)
                    }
                }
                result
            },
            WordPart::Escaped(c) => c.to_string(),
        }).collect()
    }

    pub fn push_str(&mut self, s: &str) {
        if let Some(WordPart::Simple(last)) = self.parts.last_mut() {
            last.push_str(s);
        } else {
            self.add_part(WordPart::Simple(s.to_string()));
        }
    }
}
