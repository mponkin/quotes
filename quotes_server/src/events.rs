use std::{fmt::Display, net::SocketAddrV4};

use quotes_lib::error::QuotesError;

use crate::error::ServerError;

#[derive(Debug)]
pub enum Event {
    QuotesUpdated,
    NewClient(SocketAddrV4, Vec<String>),
    Error(ServerError),
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::QuotesUpdated => write!(f, "QuotesUpdated"),
            Event::NewClient(address, tickers) => write!(f, "NewClient({address}, {tickers:?})"),
            Event::Error(server_error) => write!(f, "Error({server_error})"),
        }
    }
}

impl From<ServerError> for Event {
    fn from(value: ServerError) -> Self {
        Event::Error(value)
    }
}

impl From<QuotesError> for Event {
    fn from(value: QuotesError) -> Self {
        Self::Error(ServerError::Quotes(value))
    }
}
