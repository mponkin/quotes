//! Client messages module
use std::{
    fmt::Display,
    net::{AddrParseError, SocketAddrV4},
};

use crate::error::QuotesError;

/// CLient message with request for streaming tickers data on address
#[derive(Debug, Clone)]
pub struct SubscribeMessage {
    /// address for UDP connection
    pub address: SocketAddrV4,
    /// list of tickers to stream
    pub tickers: Vec<String>,
}

impl SubscribeMessage {
    /// Create new SubscribeMessage
    pub fn new(address: SocketAddrV4, tickers: Vec<String>) -> Self {
        Self { address, tickers }
    }

    const HEADER: &str = "SUBSCRIBE";
}

impl Display for SubscribeMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            Self::HEADER,
            self.address,
            self.tickers.join(",")
        )
    }
}

impl TryFrom<&str> for SubscribeMessage {
    type Error = QuotesError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts = value.trim().split(" ").collect::<Vec<_>>();

        if parts.len() == 3 && parts[0] == Self::HEADER {
            let address = parts[1]
                .parse()
                .map_err(|e: AddrParseError| QuotesError::ParseClientMessageError(e.to_string()))?;

            let tickers = parts[2]
                .split(",")
                .map(|t| t.to_string())
                .collect::<Vec<_>>();

            Ok(Self { address, tickers })
        } else {
            Err(QuotesError::ParseClientMessageError(
                "Unexpected client message format".to_string(),
            ))
        }
    }
}

/// Client message to keep UDP connection alive
#[derive(Debug, Clone)]
pub struct PingMessage;

impl PingMessage {
    const HEADER: &str = "PING";
}

impl Display for PingMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::HEADER)
    }
}

impl TryFrom<&[u8]> for PingMessage {
    type Error = QuotesError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value == Self::HEADER.as_bytes() {
            Ok(Self)
        } else {
            Err(QuotesError::ParseClientMessageError(
                "Unexpected client message format".to_string(),
            ))
        }
    }
}

impl Into<Vec<u8>> for PingMessage {
    fn into(self) -> Vec<u8> {
        Self::HEADER.as_bytes().to_vec()
    }
}
