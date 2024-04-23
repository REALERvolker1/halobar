use super::ParserState;

/// An error that can be returned by the formatter
#[derive(Debug, Clone, PartialEq, Eq, derive_more::Error, derive_more::Display)]
pub enum FormatStrError {
    /// The current scope was ended without returning to Literal.
    #[display(fmt = "Current scope ({_0:?}) came to an abrupt end at position {_1}")]
    AbruptEnd(ParserState, usize),
    /// A symbol in the wrong position, or maybe the wrong type of symbol
    #[display(fmt = "Invalid symbol in current scope ({_0:?}) at position {_1}: {_2}")]
    InvalidSymbol(ParserState, usize, char),
    /// Invalid variable, most likely returned from a call to [`super::variable_map`]
    #[display(fmt = "Invalid variable: {_0}")]
    #[error(ignore)]
    InvalidVariable(String),
}
