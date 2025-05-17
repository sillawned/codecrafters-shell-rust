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
            if let Some(&next_char_val) = chars.peek() {
                if is_double_quoted {
                    // Inside double quotes: backslash retains special meaning only before $, `, ", \\, or newline
                    match next_char_val {
                        '$' | '`' | '"' | '\\' => {
                            result.push(next_char_val); // Backslash removed, character is itself
                            chars.next(); // Consume the peeked character
                        }
                        // Removed newline handling: In double quotes, \\n is typically literal \\n.
                        // If `echo -e` like behavior is desired, it's handled differently.
                        _ => {
                            result.push('\\'); // Keep the backslash
                            result.push(next_char_val); // This was pushing the char again, making \\X -> \\XX
                            chars.next(); // This was consuming the char, correct is to let the outer loop pick it up or push it here and consume
                        }
                    }
                } else { // Not double_quoted (unquoted context)
                    // In unquoted context, backslash preserves the literal value of the following character.
                    // \X becomes X.
                    result.push(next_char_val);
                    chars.next(); // Consume the peeked character
                }
            } else {
                // Trailing backslash: becomes literal (standard behavior)
                result.push('\\');
            }
        } else {
            result.push(c);
        }
    }
    result
}
