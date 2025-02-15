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
                // TODO: The main issue is here
                // 1. Inside double quotes, we should:
                //    - Keep single quotes as literal quotes: 'world' -> 'world'
                //    - Keep escaped backslashes: \\' -> \'
                //    - Don't escape unescaped quotes
                let mut result = String::new();
                let mut chars = s.chars().peekable();
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => {
                            if let Some(&next) = chars.peek() {
                                match next {
                                    '"' | '\\' | '$' | '`' => {
                                        // For these special characters, consume the backslash
                                        // and only output the character
                                        chars.next();
                                        result.push(next);
                                    },
                                    _ => {
                                        // For all other characters after backslash,
                                        // keep both the backslash and the character
                                        result.push('\\');
                                        result.push(next);
                                        chars.next();
                                    }
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
