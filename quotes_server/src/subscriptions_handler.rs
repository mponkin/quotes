use std::{
    io::{BufRead, BufReader},
    net::TcpListener,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender, unbounded};
use log::{debug, error, trace};
use quotes_lib::{error::QuotesError, subscribe_message::SubscribeMessage};

use crate::{error::ServerError, events::Event};

pub struct SubscriptionsHandler {
    port: u16,
    thread_handle: Option<JoinHandle<Result<(), ServerError>>>,
}

impl SubscriptionsHandler {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            thread_handle: None,
        }
    }

    pub fn start(&mut self) -> Result<Receiver<Event>, ServerError> {
        if self.thread_handle.is_some() {
            return Err(ServerError::ComponentAlreadyStarted(
                "SubscriptionsHandler".to_string(),
            ));
        }

        let port = self.port;
        let (tx, rx) = unbounded();
        let handle = thread::spawn(move || {
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
            debug!("Started TCP server on port {}", port);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        debug!("New client from {:?}", stream.peer_addr());
                        handle_client(stream, tx.clone())
                    }
                    Err(e) => {
                        return Err(ServerError::from(e));
                    }
                }
            }

            trace!("Stopped TCP server");

            Ok(())
        });

        self.thread_handle = Some(handle);

        Ok(rx)
    }

    pub fn stop(&mut self) -> Result<(), ServerError> {
        if let Some(handle) = self.thread_handle.take() {
            handle.join().unwrap_or(Err(ServerError::ComponentStopError(
                "SubscriptionsHandler".to_string(),
            )))
        } else {
            Ok(())
        }
    }
}

impl From<SubscribeMessage> for Event {
    fn from(value: SubscribeMessage) -> Self {
        Event::NewClient(value.address, value.tickers)
    }
}

impl From<Result<SubscribeMessage, QuotesError>> for Event {
    fn from(value: Result<SubscribeMessage, QuotesError>) -> Self {
        value.map(Event::from).unwrap_or_else(Event::from)
    }
}

fn handle_client(stream: std::net::TcpStream, tx: Sender<Event>) {
    thread::spawn(move || {
        trace!("Handling new client from {:?}", stream.peer_addr());
        let mut buf_reader = BufReader::new(stream);
        let mut buf = String::new();

        let event = if let Err(e) = buf_reader.read_line(&mut buf) {
            trace!("TCP READ ERR {e}");
            Event::from(ServerError::from(e))
        } else {
            trace!("TCP READ {buf:?}");
            Event::from(SubscribeMessage::try_from(buf.as_str()))
        };

        if let Err(e) = tx.send(event) {
            error!("Unable to send event {e}")
        } else {
            trace!("New client message sent OK");
        }
    });
}
