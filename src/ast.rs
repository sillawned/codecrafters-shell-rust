use crate::word::Word; // Import Word

#[derive(Debug, Clone)] // Added Clone
pub enum ASTNode {
    Command {
        name: Word,         // Changed from String to Word
        args: Vec<Word>,    // Changed from Vec<String> to Vec<Word>
    },
    Pipe {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    Redirect {
        command: Box<ASTNode>,
        fd: i32,
        target: RedirectTarget, // RedirectTarget itself will be updated
        mode: RedirectMode,
    },
    Background {
        command: Box<ASTNode>,
    },
    LogicalAnd {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    LogicalOr {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    Subshell {
        command: Box<ASTNode>,
    },
    Semicolon {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    Assignment {
        var: String, // Keep as String for now, or change to Word if assignments can be complex
        val: Word,   // Changed from String to Word
    },
    CommandSubstitution { // This might be removable if lexer fully handles it into Word
        command_string: String,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum RedirectMode {
    Overwrite,      // >
    Append,         // >>
    Input,          // <
    HereDoc,        // <<
    HereString,     // <<<
    DupOutput,      // >&
    DupInput,       // <&
}

#[derive(Debug, Clone)] // Added Clone
pub enum RedirectTarget {
    File(Word),             // Changed from String to Word
    Descriptor(i32),        // File descriptor number
    HereDoc(String),        // Here document content (remains String as it's raw content)
    HereString(String),     // Here string content (remains String)
}