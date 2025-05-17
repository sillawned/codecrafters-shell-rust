#[derive(Debug, Clone, PartialEq)] // Added PartialEq
pub enum WordPart {
    Simple(String),
    SingleQuoted(String),
    DoubleQuoted(String),
}

#[derive(Debug, Clone, PartialEq)] // Added PartialEq
pub struct Word {
    pub parts: Vec<WordPart>, // Made this field public
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
                // Process C-style escapes for Simple parts (unquoted)
                process_escapes(s, false)
            },
            WordPart::SingleQuoted(s) => s.clone(), // Single quotes are literal, no escape processing
            WordPart::DoubleQuoted(s) => {
                // Process C-style escapes for DoubleQuoted parts
                // Special shell escapes like \\$, \\`, \\\\, \\" are handled by process_escapes
                process_escapes(s, true)
            },
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

// Helper function to process C-style escapes
// is_double_quoted: true if the string is from a double-quoted context
fn process_escapes(s: &str, is_double_quoted: bool) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next_char) = chars.next() {
                match next_char {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'), // Literal backslash
                    '\'' => result.push('\''),   // Literal single quote
                    '"' => result.push('"'),   // Literal double quote
                    '$' if is_double_quoted => result.push_str("\\$"), // In double quotes, \\$ -> $ (handled by var expansion later) or literal \$ if not var
                    '`' if is_double_quoted => result.push_str("\\`"), // In double quotes, \\` -> ` (handled by cmd sub later) or literal \\`
                    // For any other character following a backslash:
                    // - In double quotes, if it's not one of the special ones above (n, t, r, \\, ', ", $, `),
                    //   the backslash is literal. e.g., "foo\\bar" -> foo\\bar
                    // - In unquoted context (is_double_quoted = false), the backslash escapes the next character,
                    //   making it literal. e.g., foo\\bar -> foobar (unless it's a C-style escape)
                    other => {
                        if is_double_quoted {
                            // In double quotes, unknown escapes like \\a mean literal backslash then 'a'
                            result.push('\\');
                            result.push(other);
                        } else {
                            // In unquoted context, \\a means 'a'
                            result.push(other);
                        }
                    }
                }
            } else {
                // Trailing backslash, keep it literal
                result.push('\\');
            }
        } else {
            result.push(c);
        }
    }
    result
}
