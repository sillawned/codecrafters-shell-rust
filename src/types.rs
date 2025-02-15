#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuoteType {
    Single,     // '
    Double,     // "
    Escaped,    // \
    None,       // No quotes
}
