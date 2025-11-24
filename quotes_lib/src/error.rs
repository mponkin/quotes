//! Quotes lib errors
use std::{
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
};

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
    ParseServerMessageError(String),
    /// Unable to parse datagram
    ParseDatagramError,
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

impl Display for QuotesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotesError::IoError(reason) => write!(f, "I/O error: {reason}"),
            QuotesError::ParseQuoteError(reason) => write!(f, "Parse quote error: {reason}"),
            QuotesError::ParseClientMessageError(reason) => {
                write!(f, "Parse client message error: {reason}")
            }
            QuotesError::ParseServerMessageError(reason) => {
                write!(f, "Parse server message error: {reason}")
            }
            QuotesError::ParseDatagramError => {
                write!(f, "Unable to parse datagram")
            }
        }
    }
}
