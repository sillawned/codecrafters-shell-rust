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
            WordPart::Simple(s) => s.clone(),
            WordPart::SingleQuoted(s) => s.clone(),
            WordPart::DoubleQuoted(s) => {
                // FIXME: Within double quotes:
                // 1. Preserve backslashes that escape special chars (", \, $, `)
                // 2. Keep backslash-escaped backslashes as single backslash
                // 3. Keep regular backslashes as backslashes
                let mut result = String::new();
                let mut chars = s.chars().peekable();
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => {
                            if let Some(&next) = chars.peek() {
                                match next {
                                    // Special chars that can be escaped in double quotes
                                    '"' | '\\' | '$' | '`' => {
                                        chars.next();
                                        if next == '\\' {
                                            result.push('\\');
                                        }
                                        result.push(next);
                                    },
                                    // Any other backslash is preserved literally
                                    _ => result.push('\\')
                                }
                            } else {
                                result.push('\\');
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
