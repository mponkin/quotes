use std::{
    net::{SocketAddrV4, UdpSocket},
    sync::Arc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender, unbounded};
use log::{debug, trace, warn};
use quotes_lib::{
    datagram::{Datagram, DatagramParser},
    quote::Quote,
    server_message::ServerMessage,
    subscribe_message::PingMessage,
};

use crate::error::ServerError;

pub enum SingleClientHandlerEvent {
    Disconnected(SocketAddrV4),
    Error(SocketAddrV4, ServerError),
}

pub enum SingleClientCommand {
    SendQuote(Quote),
    Stop,
}

pub struct SingleClientHandler {
    tickers: Vec<String>,
    command_tx: Sender<SingleClientCommand>,
    listen_thread: JoinHandle<()>,
    send_thread: JoinHandle<()>,
}

impl SingleClientHandler {
    pub fn new(
        address: SocketAddrV4,
        tickers: Vec<String>,
        event_sender: Sender<SingleClientHandlerEvent>,
        ping_timeout: Duration,
    ) -> Result<Self, ServerError> {
        let socket = Arc::new(UdpSocket::bind("127.0.0.1:0")?);
        let (command_tx, command_rx) = unbounded();
        let read_socket = socket.clone();

        let listen_thread = Self::setup_listen_thread(
            read_socket,
            command_rx.clone(),
            event_sender.clone(),
            address,
            ping_timeout,
        )?;
        let send_thread = Self::setup_send_thread(socket, command_rx, address, event_sender);
        Ok(Self {
            tickers,
            command_tx,
            listen_thread,
            send_thread,
        })
    }

    fn setup_listen_thread(
        socket: Arc<UdpSocket>,
        command_rx: Receiver<SingleClientCommand>,
        event_sender: Sender<SingleClientHandlerEvent>,
        address: SocketAddrV4,
        ping_timeout: Duration,
    ) -> Result<JoinHandle<()>, ServerError> {
        const SINGLE_READ_TIMEOUT: Duration = Duration::from_millis(250);
        socket.set_read_timeout(Some(SINGLE_READ_TIMEOUT))?;

        let handle = thread::spawn(move || {
            let mut buf = [0; 2048];
            let mut datagram_parser = DatagramParser::new();
            let mut last_ping_time = Instant::now();

            loop {
                let result = socket.recv_from(&mut buf);

                let datagrams = match result {
                    Ok((bytes_read, ..)) => match datagram_parser.parse(&buf[0..bytes_read]) {
                        Ok(datagrams) => datagrams,
                        // don't care if datagrams contains errors
                        Err(datagrams) => datagrams,
                    },
                    // both timeout and read error
                    Err(_) => vec![],
                };

                if let Ok(SingleClientCommand::Stop) = command_rx.try_recv() {
                    debug!("Stop command received, shutting down listen thread for {address}");
                    break;
                }

                let now = Instant::now();

                if now - last_ping_time >= ping_timeout {
                    if let Err(send_error) =
                        event_sender.send(SingleClientHandlerEvent::Disconnected(address))
                    {
                        warn!("Unable to send disconnect event {send_error}");
                    }
                    break;
                }

                let have_ping = datagrams
                    .into_iter()
                    .any(|dg| PingMessage::try_from(dg.data.as_slice()).is_ok());

                if have_ping {
                    last_ping_time = now;
                }
            }
        });

        Ok(handle)
    }

    fn setup_send_thread(
        socket: Arc<UdpSocket>,
        command_rx: Receiver<SingleClientCommand>,
        address: SocketAddrV4,
        event_tx: Sender<SingleClientHandlerEvent>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            loop {
                match command_rx.recv() {
                    Ok(SingleClientCommand::SendQuote(quote)) => {
                        trace!("Sending {quote} to {address}");
                        let datagram = Datagram::from(ServerMessage::Quote(quote));
                        let buf: Vec<u8> = datagram.into();
                        if let Err(io_err) = socket.send_to(&buf, address)
                            && let Err(send_err) = event_tx.send(SingleClientHandlerEvent::Error(
                                address,
                                ServerError::from(io_err),
                            ))
                        {
                            warn!("Unable to send client message {send_err}");
                            break;
                        }
                    }
                    Ok(SingleClientCommand::Stop) => {
                        debug!("Stop command received, shutting down send thread for {address}");
                        break;
                    }
                    Err(e) => {
                        warn!("Error recieving data: {e}");
                        break;
                    }
                }
            }
        })
    }

    pub fn send_quote(&self, quote: Quote) -> Result<(), ServerError> {
        self.command_tx
            .send(SingleClientCommand::SendQuote(quote))
            .map_err(|e| ServerError::SendError(e.to_string()))
    }

    pub fn stop(self) -> Result<(), ServerError> {
        trace!("Stopping single client handler");
        if let Err(e) = self.command_tx.send(SingleClientCommand::Stop) {
            warn!("Unable to send stop command {e}");
        } else {
            trace!("Sent stop signal");
        }

        if self.listen_thread.join().is_err() {
            warn!("Client listen_thread join error");
        } else {
            trace!("Stopped listen thread");
        }
        if self.send_thread.join().is_err() {
            warn!("Client send_thread join error");
        } else {
            trace!("Stopped send thread");
        }
        Ok(())
    }

    pub fn tickers(&self) -> &[String] {
        &self.tickers
    }
}
