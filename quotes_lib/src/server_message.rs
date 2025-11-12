//! Server messages module
use std::fmt::Display;

use crate::{error::QuotesError, quote::Quote};

/// Server messages variants
#[derive(Debug, Clone)]
pub enum ServerMessage {
    /// Message containing quote
    Quote(Quote),
    /// Message with error description
    Err(String),
}

impl Display for ServerMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerMessage::Quote(quote) => write!(f, "QUOTE({quote})"),
            ServerMessage::Err(message) => write!(f, "ERROR({message})"),
        }
    }
}

impl TryFrom<&str> for ServerMessage {
    type Error = QuotesError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.starts_with("QUOTE(") && value.ends_with(")") {
            let quote_str = &value[6..value.len() - 1];
            let quote = Quote::try_from(quote_str)?;
            Ok(ServerMessage::Quote(quote))
        } else if value.starts_with("ERROR(") && value.ends_with(")") {
            let message = value[6..value.len() - 1].to_string();
            Ok(ServerMessage::Err(message))
        } else {
            Err(QuotesError::ParseServerMessageError)
        }
    }
}
