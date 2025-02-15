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
        self.parts.iter().map(|part| match part {
            WordPart::Simple(s) => {
                let mut result = String::new();
                let mut chars = s.chars().peekable();
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => {
                            // In unquoted text:
                            // Any character after backslash should be taken literally
                            // The backslash is removed, the next character is used as-is
                            if let Some(next) = chars.next() {
                                result.push(next);
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
