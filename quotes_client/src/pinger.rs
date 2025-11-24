use std::{
    net::{SocketAddr, UdpSocket},
    sync::{Arc, atomic::AtomicBool},
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, unbounded};
use log::{trace, warn};
use quotes_lib::{datagram::Datagram, subscribe_message::PingMessage};

use crate::error::ClientError;

pub struct Pinger {
    command_tx: Sender<PingerCommand>,
    handle: JoinHandle<Result<(), ClientError>>,
}

impl Pinger {
    pub fn new(running: Arc<AtomicBool>, socket: Arc<UdpSocket>, interval: Duration) -> Self {
        let (command_tx, command_rx) = unbounded();
        Self {
            command_tx,
            handle: Self::setup_thread(running, socket, command_rx, interval),
        }
    }

    fn setup_thread(
        running: Arc<AtomicBool>,
        socket: Arc<UdpSocket>,
        command_rx: Receiver<PingerCommand>,
        interval: Duration,
    ) -> JoinHandle<Result<(), ClientError>> {
        const MAX_ERRORS: usize = 3;

        thread::spawn(move || {
            trace!("Starting pinger thread");
            let mut ping_address: Option<SocketAddr> = None;
            let mut error_count = 0;
            while running.load(std::sync::atomic::Ordering::SeqCst) {
                match command_rx.recv_timeout(interval) {
                    Ok(PingerCommand::Start(socket_addr)) => {
                        match ping_address {
                            Some(address) => {
                                warn!(
                                    "Received Start({socket_addr}) command when already have address {address}. Ignoring"
                                );
                            }
                            None => {
                                trace!("Received Start({socket_addr}), start sending ping");
                                ping_address = Some(socket_addr);
                            }
                        };
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        // ping interval tick
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        trace!("Sender is shut down. Finishing work");
                        break;
                    }
                }

                if let Some(address) = ping_address {
                    let buf: Vec<u8> = Datagram::new(PingMessage.into()).into();

                    if let Err(e) = socket.send_to(&buf, address) {
                        error_count += 1;
                        warn!("Send ping error({error_count}) {e}");
                        if error_count >= MAX_ERRORS {
                            warn!("Too many errors, shutting down");
                            return Err(ClientError::from(e));
                        }
                    } else {
                        trace!("Sent PING to {address}");
                        error_count = 0;
                    };
                }
            }

            trace!("Pinger thread finished successfully");
            Ok(())
        })
    }

    pub fn start_ping(&self, address: SocketAddr) -> Result<(), ClientError> {
        trace!("Starting ping {address}");
        self.command_tx.send(PingerCommand::Start(address))?;
        Ok(())
    }

    pub fn shutdown(self) -> Result<(), ClientError> {
        trace!("Shutting down pinger");

        self.handle
            .join()
            .unwrap_or_else(|_| Err(ClientError::ThreadJoin))
    }
}

enum PingerCommand {
    Start(SocketAddr),
}
