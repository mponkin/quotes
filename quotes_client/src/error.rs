use std::fmt::Display;

use log::SetLoggerError;
use quotes_lib::error::QuotesError;

#[derive(Debug)]
pub enum ClientError {
    LoggerInit(String),
    Io(String),
    Quotes(QuotesError),
    SendError(String),
    RecvError(String),
    ThreadJoin,
    CtrlCError(ctrlc::Error),
}

impl From<SetLoggerError> for ClientError {
    fn from(value: SetLoggerError) -> Self {
        ClientError::LoggerInit(value.to_string())
    }
}

impl From<std::io::Error> for ClientError {
    fn from(value: std::io::Error) -> Self {
        ClientError::Io(value.to_string())
    }
}

impl From<QuotesError> for ClientError {
    fn from(value: QuotesError) -> Self {
        ClientError::Quotes(value)
    }
}

impl<T> From<crossbeam_channel::SendError<T>> for ClientError {
    fn from(value: crossbeam_channel::SendError<T>) -> Self {
        ClientError::SendError(value.to_string())
    }
}
impl From<crossbeam_channel::RecvError> for ClientError {
    fn from(value: crossbeam_channel::RecvError) -> Self {
        ClientError::SendError(value.to_string())
    }
}

impl From<ctrlc::Error> for ClientError {
    fn from(value: ctrlc::Error) -> Self {
        ClientError::CtrlCError(value)
    }
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::LoggerInit(reason) => write!(f, "Logger init error: {reason}"),
            ClientError::Io(reason) => write!(f, "I/O error: {reason}"),
            ClientError::Quotes(quotes_error) => write!(f, "{quotes_error}",),
            ClientError::SendError(reason) => write!(f, "Send error: {reason}"),
            ClientError::RecvError(reason) => write!(f, "Receive error: {reason}"),
            ClientError::ThreadJoin => write!(f, "Thread stop error"),
            ClientError::CtrlCError(e) => write!(f, "Ctrl-C setup error {e}"),
        }
    }
}
