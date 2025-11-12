//! Client messages module
use std::{
    fmt::Display,
    net::{AddrParseError, SocketAddrV4},
};

use crate::error::QuotesError;

/// Client message variants
#[derive(Debug, Clone)]
pub enum ClientMessage {
    /// Start sending quotes to given address, filter tickers from vec
    Subscribe(SocketAddrV4, Vec<String>),
    /// Stop sending data to address
    Unsubscribe(SocketAddrV4),
    /// Ping message to keep connection alive
    Ping,
}

impl Display for ClientMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientMessage::Subscribe(socket_addr_v4, items) => {
                write!(f, "SUBSCRIBE {socket_addr_v4} {}", items.join(","))
            }
            ClientMessage::Unsubscribe(socket_addr_v4) => write!(f, "UNSUBSCRIBE {socket_addr_v4}"),
            ClientMessage::Ping => write!(f, "PING"),
        }
    }
}

impl TryFrom<&str> for ClientMessage {
    type Error = QuotesError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts = value.trim().split(" ").collect::<Vec<_>>();

        let parse_address = |str: &str| -> Result<SocketAddrV4, QuotesError> {
            str.parse()
                .map_err(|e: AddrParseError| QuotesError::ParseClientMessageError(e.to_string()))
        };

        match parts.len() {
            3 if parts[0] == "SUBSCRIBE" => {
                let tickers = parts[2]
                    .split(",")
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>();

                Ok(ClientMessage::Subscribe(parse_address(parts[1])?, tickers))
            }
            2 if parts[0] == "UNSUBSCRIBE" => {
                Ok(ClientMessage::Unsubscribe(parse_address(parts[1])?))
            }
            1 if parts[0] == "PING" => Ok(ClientMessage::Ping),
            _ => Err(QuotesError::ParseClientMessageError(
                "Unexpected request format".to_string(),
            )),
        }
    }
}
