use std::{fmt::Display, net::SocketAddrV4};

use crossbeam_channel::{RecvError, SendError};
use log::SetLoggerError;
use quotes_lib::error::QuotesError;

use crate::events::Event;

#[derive(Debug)]
pub enum ServerError {
    LoggerInit(String),
    Io(String),
    Quotes(QuotesError),
    ComponentAlreadyStarted(String),
    ComponentStopError(String),
    SendError(String),
    RecvError(String),
    QuotesSourceDataError,
    AddressAlreadyInUse(SocketAddrV4),
    QuotesReadError(String),
    ClientsReadError(String),
}

impl From<SetLoggerError> for ServerError {
    fn from(value: SetLoggerError) -> Self {
        ServerError::LoggerInit(value.to_string())
    }
}

impl From<std::io::Error> for ServerError {
    fn from(value: std::io::Error) -> Self {
        ServerError::Io(value.to_string())
    }
}

impl From<QuotesError> for ServerError {
    fn from(value: QuotesError) -> Self {
        ServerError::Quotes(value)
    }
}

impl From<SendError<Event>> for ServerError {
    fn from(value: SendError<Event>) -> Self {
        ServerError::SendError(value.to_string())
    }
}

impl From<RecvError> for ServerError {
    fn from(value: RecvError) -> Self {
        ServerError::RecvError(value.to_string())
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::LoggerInit(reason) => write!(f, "Logger init error: {reason}"),
            ServerError::Io(reason) => write!(f, "I/O error: {reason}"),
            ServerError::Quotes(quotes_error) => write!(f, "{}", quotes_error),
            ServerError::ComponentAlreadyStarted(name) => write!(f, "{name} is already started"),
            ServerError::ComponentStopError(name) => write!(f, "Error while stopping {name}"),
            ServerError::SendError(reason) => {
                write!(f, "Unable to send data through channel: {reason}")
            }
            ServerError::RecvError(reason) => {
                write!(f, "Unable to receive data through channel: {reason}")
            }
            ServerError::QuotesSourceDataError => write!(f, "Error updating quotes source"),
            ServerError::AddressAlreadyInUse(socket_addr_v4) => {
                write!(f, "Client with address {socket_addr_v4} already exists")
            }
            ServerError::QuotesReadError(reason) => write!(f, "Quotes lock read error: {reason}"),
            ServerError::ClientsReadError(reason) => write!(f, "Clients lock read error: {reason}"),
        }
    }
}
