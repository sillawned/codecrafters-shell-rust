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
        file: String,
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
    Overwrite, // >
    Append,    // >>
    Input,     // <
}