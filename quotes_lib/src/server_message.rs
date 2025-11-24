//! Server messages module
use std::fmt::Display;

use crate::{datagram::Datagram, error::QuotesError, quote::Quote};

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

impl ServerMessage {
    const QUOTE_TYPE_CODE: u8 = 0;
    const ERROR_TYPE_CODE: u8 = u8::MAX;

    fn type_code(&self) -> u8 {
        match self {
            ServerMessage::Quote(_) => ServerMessage::QUOTE_TYPE_CODE,
            ServerMessage::Err(_) => ServerMessage::ERROR_TYPE_CODE,
        }
    }

    fn content_bytes(&self) -> Vec<u8> {
        match self {
            ServerMessage::Quote(quote) => quote.into(),
            ServerMessage::Err(message) => message.as_bytes().to_vec(),
        }
    }
}

impl From<ServerMessage> for Vec<u8> {
    fn from(value: ServerMessage) -> Self {
        let mut result = vec![];
        result.push(value.type_code());
        result.extend_from_slice(&value.content_bytes());
        result
    }
}

impl TryFrom<Vec<u8>> for ServerMessage {
    type Error = QuotesError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(QuotesError::ParseServerMessageError(
                "Bytes are empty".to_string(),
            ));
        }
        match value[0] {
            ServerMessage::QUOTE_TYPE_CODE => {
                Ok(ServerMessage::Quote(Quote::try_from(&value[1..])?))
            }
            ServerMessage::ERROR_TYPE_CODE => {
                let message = String::from_utf8(value[1..].to_vec())
                    .map_err(|e| QuotesError::ParseServerMessageError(e.to_string()))?;

                Ok(ServerMessage::Err(message))
            }
            other => Err(QuotesError::ParseServerMessageError(format!(
                "Unexpected message code {other}"
            ))),
        }
    }
}

impl TryFrom<Datagram> for ServerMessage {
    type Error = QuotesError;

    fn try_from(value: Datagram) -> Result<Self, Self::Error> {
        Self::try_from(value.data)
    }
}
