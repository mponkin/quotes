//! Quotes lib errors
use std::num::{ParseFloatError, ParseIntError};

/// Error variants
#[derive(Debug)]
pub enum QuotesError {
    /// Io error
    IoError(String),
    /// Problem parsing quote
    ParseQuoteError(String),
    /// Problem parsing client message
    ParseClientMessageError(String),
    /// Problem parsing server message
    ParseServerMessageError,
}

impl From<ParseFloatError> for QuotesError {
    fn from(value: ParseFloatError) -> Self {
        QuotesError::ParseQuoteError(value.to_string())
    }
}
impl From<ParseIntError> for QuotesError {
    fn from(value: ParseIntError) -> Self {
        QuotesError::ParseQuoteError(value.to_string())
    }
}

impl From<std::io::Error> for QuotesError {
    fn from(value: std::io::Error) -> Self {
        QuotesError::IoError(value.to_string())
    }
}
