use std::{
    collections::{HashMap, hash_map::Entry},
    net::SocketAddrV4,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::{Receiver, Sender, unbounded};
use log::{error, trace, warn};
use quotes_lib::quote::Quote;

use crate::{
    error::ServerError,
    single_client_handler::{SingleClientHandler, SingleClientHandlerEvent},
};

pub struct ClientsHandler {
    quotes: Arc<RwLock<HashMap<String, Quote>>>,
    clients: Arc<RwLock<HashMap<SocketAddrV4, SingleClientHandler>>>,
    event_tx: Sender<SingleClientHandlerEvent>,
    event_rx: Receiver<SingleClientHandlerEvent>,
    thread_handle: Option<JoinHandle<()>>,
}

impl ClientsHandler {
    pub fn new(quotes: Arc<RwLock<HashMap<String, Quote>>>) -> Self {
        let (event_tx, event_rx) = unbounded();
        let clients = Arc::new(RwLock::new(HashMap::new()));

        Self {
            quotes,
            clients,
            event_tx,
            event_rx,
            thread_handle: None,
        }
    }

    pub fn start(&mut self) -> Result<(), ServerError> {
        if self.thread_handle.is_some() {
            return Err(ServerError::ComponentAlreadyStarted(
                "ClientsHandler".to_string(),
            ));
        }

        let handle = {
            let event_rx = self.event_rx.clone();
            let clients = self.clients.clone();

            thread::spawn(move || {
                loop {
                    match event_rx.recv() {
                        Ok(msg) => match msg {
                            SingleClientHandlerEvent::Disconnected(socket_addr_v4) => {
                                if let Err(e) = Self::remove_and_stop_clients(
                                    clients.clone(),
                                    &[socket_addr_v4],
                                ) {
                                    error!("Error stopping clients {e}");
                                    break;
                                }
                            }
                            SingleClientHandlerEvent::Error(socket_addr_v4, server_error) => {
                                warn!("Error in client for {socket_addr_v4}: {server_error}")
                            }
                        },
                        Err(e) => warn!("ClientsHandler listen events read error {e}"),
                    }
                }
            })
        };

        self.thread_handle = Some(handle);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), ServerError> {
        if let Some(handle) = self.thread_handle.take() {
            let all_clients = self
                .clients
                .read()
                .map(|guard| guard.keys().copied().collect::<Vec<_>>())
                .map_err(|e| ServerError::ClientsReadError(e.to_string()))?;
            if let Err(e) = Self::remove_and_stop_clients(self.clients.clone(), &all_clients) {
                error!("Error stopping clients {e}");
            }

            handle
                .join()
                .map_err(|_| ServerError::ComponentStopError("ClientsHandler".to_string()))
        } else {
            Ok(())
        }
    }

    pub fn handle_quotes_updated(&mut self) -> Result<(), ServerError> {
        trace!("handle_quotes_updated");
        let mut clients_with_errors = vec![];

        {
            let quotes = self
                .quotes
                .read()
                .map_err(|e| ServerError::QuotesReadError(e.to_string()))?;

            let clients = self
                .clients
                .read()
                .map_err(|e| ServerError::ClientsReadError(e.to_string()))?;

            for (addr, client) in clients.iter() {
                for ticker in client.tickers().iter() {
                    if let Some(quote) = quotes.get(ticker) {
                        if let Err(e) = client.send_quote(quote.clone()) {
                            warn!("Client unable to send quote {e}");
                            clients_with_errors.push(*addr);
                            break;
                        }
                    } else {
                        warn!("Ticker not found {ticker}");
                    }
                }
            }

            trace!("Clients with errors count {}", clients_with_errors.len());
        }

        Self::remove_and_stop_clients(self.clients.clone(), &clients_with_errors)
    }

    fn remove_and_stop_clients(
        clients: Arc<RwLock<HashMap<SocketAddrV4, SingleClientHandler>>>,
        addr_to_remove: &[SocketAddrV4],
    ) -> Result<(), ServerError> {
        trace!("remove_and_stop_clients {addr_to_remove:?}");
        if addr_to_remove.is_empty() {
            trace!("list is empty");
            return Ok(());
        }

        {
            let mut guard = match clients.write() {
                Ok(guard) => guard,
                Err(e) => return Err(ServerError::ClientsReadError(e.to_string())),
            };

            for addr in addr_to_remove {
                if let Some(client) = guard.remove(addr)
                    && let Err(e) = client.stop()
                {
                    warn!("Client stop error {e}");
                }
            }
        }

        Ok(())
    }

    const CLIENT_PING_TIMEOUT: Duration = Duration::from_secs(5);

    pub fn handle_new_client(
        &mut self,
        address: SocketAddrV4,
        tickers: Vec<String>,
    ) -> Result<(), ServerError> {
        let mut guard = match self.clients.write() {
            Ok(guard) => guard,
            Err(e) => return Err(ServerError::ClientsReadError(e.to_string())),
        };

        match guard.entry(address) {
            Entry::Occupied(_) => Err(ServerError::AddressAlreadyInUse(address)),
            Entry::Vacant(entry) => {
                let client = SingleClientHandler::new(
                    address,
                    tickers,
                    self.event_tx.clone(),
                    Self::CLIENT_PING_TIMEOUT,
                )?;
                entry.insert(client);
                Ok(())
            }
        }
    }
}
