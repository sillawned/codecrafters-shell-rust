#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuoteType {
    Single,     // '
    Double,     // "
    Escaped,    // \\
    None,       // No quotes
}

#[derive(Debug)]
pub enum ShellError {
    IoError(std::io::Error),
    NixError(nix::errno::Errno, String), // For nix-related errors, with context
    ParseError(String),
    CommandNotFound(String),
    InvalidSyntax(String),
    InternalError(String),
    // Add other specific error types as needed
}

// Optional: Implement From for easier error conversion
impl From<std::io::Error> for ShellError {
    fn from(err: std::io::Error) -> Self {
        ShellError::IoError(err)
    }
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellError::IoError(e) => write!(f, "IO error: {}", e),
            ShellError::NixError(e, ctx) => write!(f, "System error ({}): {}", ctx, e),
            ShellError::ParseError(s) => write!(f, "Parse error: {}", s),
            ShellError::CommandNotFound(s) => write!(f, "Command not found: {}", s),
            ShellError::InvalidSyntax(s) => write!(f, "Invalid syntax: {}", s),
            ShellError::InternalError(s) => write!(f, "Internal shell error: {}", s),
        }
    }
}

impl std::error::Error for ShellError {}

pub type ExecuteResult = Result<i32, ShellError>;
