use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, atomic::AtomicBool},
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::Sender;
use log::{trace, warn};
use quotes_lib::{datagram::DatagramParser, error::QuotesError, server_message::ServerMessage};

use crate::error::ClientError;

pub struct QuotesListener {
    handle: JoinHandle<Result<(), ClientError>>,
}

impl QuotesListener {
    pub fn new(
        running: Arc<AtomicBool>,
        socket: Arc<UdpSocket>,
        event_tx: Sender<QuotesListenerEvent>,
    ) -> Self {
        Self {
            handle: Self::setup_thread(running, socket, event_tx),
        }
    }

    fn setup_thread(
        running: Arc<AtomicBool>,
        socket: Arc<UdpSocket>,
        event_tx: Sender<QuotesListenerEvent>,
    ) -> JoinHandle<Result<(), ClientError>> {
        const READ_TIMEOUT: Duration = Duration::from_millis(2000);

        let mut datagram_parser = DatagramParser::new();
        let mut buf = [0u8; 2048];

        thread::spawn(move || {
            trace!("Starting quotes listener thread");
            socket.set_read_timeout(Some(READ_TIMEOUT))?;

            while running.load(std::sync::atomic::Ordering::SeqCst) {
                match socket.recv_from(&mut buf) {
                    Ok((len, address)) => {
                        let data = datagram_parser.parse(&buf[..len]);

                        let messages = data.map_or_else(
                            |e| vec![QuotesListenerEvent::from(e)],
                            |datagrams| {
                                datagrams
                                    .into_iter()
                                    .map(|dg| match ServerMessage::try_from(dg) {
                                        Ok(msg) => QuotesListenerEvent::Message(msg, address),
                                        Err(e) => QuotesListenerEvent::from(e),
                                    })
                                    .collect()
                            },
                        );

                        for message in messages {
                            if let Err(e) = event_tx.send(message) {
                                warn!("Unable to send error event, shutting down");
                                return Err(ClientError::from(e));
                            }
                        }
                    }
                    Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                        // read timeout to check commands
                        continue;
                    }
                    Err(io_err) => {
                        if let Err(send_err) =
                            event_tx.send(QuotesListenerEvent::Error(ClientError::from(io_err)))
                        {
                            warn!("Unable to send error event, shutting down");
                            return Err(ClientError::from(send_err));
                        }
                    }
                }
            }

            trace!("Pinger thread finished successfully");
            Ok(())
        })
    }

    pub fn shutdown(self) -> Result<(), ClientError> {
        trace!("Shutting down quotes listener");

        self.handle
            .join()
            .unwrap_or_else(|_| Err(ClientError::ThreadJoin))
    }
}

pub enum QuotesListenerEvent {
    Message(ServerMessage, SocketAddr),
    Error(ClientError),
}

impl From<ClientError> for QuotesListenerEvent {
    fn from(value: ClientError) -> Self {
        QuotesListenerEvent::Error(value)
    }
}

impl From<QuotesError> for QuotesListenerEvent {
    fn from(value: QuotesError) -> Self {
        Self::from(ClientError::Quotes(value))
    }
}
