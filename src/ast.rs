#[derive(Debug)]
pub enum ASTNode {
    Command {
        name: String,
        args: Vec<String>,
    },
    Pipe {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    Redirect {
        command: Box<ASTNode>,
        fd: i32,
        target: RedirectTarget,
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
        var: String,
        val: String,
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

#[derive(Debug)]
pub enum RedirectTarget {
    File(String),           // Regular file
    Descriptor(i32),        // File descriptor number
    HereDoc(String),        // Here document content
    HereString(String),     // Here string content
}