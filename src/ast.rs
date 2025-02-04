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
        file: String,
        mode: RedirectMode,
    },
    Builtin {
        name: String,
        args: Vec<String>,
    },
}

#[derive(Debug)]
pub enum RedirectMode {
    Overwrite, // >
    Append,    // >>
    Input,     // <
}