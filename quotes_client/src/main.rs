use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream, UdpSocket},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use clap::Parser;
use crossbeam_channel::unbounded;
use env_logger::Builder;
use log::{LevelFilter, debug, error, info, trace, warn};
use quotes_lib::{
    read_tickers_from_file, server_message::ServerMessage, subscribe_message::SubscribeMessage,
};

use crate::{
    error::ClientError,
    pinger::Pinger,
    quotes_listener::{QuotesListener, QuotesListenerEvent},
};

mod error;
mod pinger;
mod quotes_listener;

#[derive(Parser, Debug)]
struct Args {
    server_address: SocketAddr,
    #[arg(short = 'p', long)]
    port: u16,
    #[arg(short = 't', long)]
    tickers: PathBuf,
}

fn init_logger() -> Result<(), ClientError> {
    Builder::new()
        .filter_level(LevelFilter::Debug)
        .try_init()
        .map_err(ClientError::from)
}

fn main() {
    if let Err(e) = run_client() {
        error!("{e}");
    } else {
        debug!("Client shut down successfully");
    }
}

fn run_client() -> Result<(), ClientError> {
    const PING_INTERVAL: Duration = Duration::from_millis(1000);
    const MAX_ERRORS: usize = 3;
    init_logger()?;
    let args = Args::parse();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        debug!("Received Ctrl-C, stopping...");
        r.store(false, Ordering::SeqCst);
    })?;

    let tickers = read_tickers_from_file(args.tickers)?;
    let tcp_stream = setup_connection(args.server_address)?;

    debug!("Listenting to UDP socket on port {}", args.port);
    let socket = Arc::new(UdpSocket::bind(format!("127.0.0.1:{}", args.port))?);

    request_data(tcp_stream, args.port, tickers)?;

    let (event_tx, event_rx) = unbounded();

    let quotes_listener = QuotesListener::new(running.clone(), socket.clone(), event_tx);
    let pinger = Pinger::new(running.clone(), socket.clone(), PING_INTERVAL);

    let mut ping_started = false;
    let mut error_count = 0;

    while running.load(Ordering::SeqCst) {
        match event_rx.recv() {
            Ok(event) => match event {
                QuotesListenerEvent::Message(server_message, address) => {
                    if !ping_started {
                        if let Err(e) = pinger.start_ping(address) {
                            warn!("Unable to start ping {e}");
                            break;
                        };
                        ping_started = true;
                    }

                    match server_message {
                        ServerMessage::Quote(quote) => info!("{quote}"),
                        ServerMessage::Err(e) => warn!("SERVER ERROR {e}"),
                    }
                }
                QuotesListenerEvent::Error(client_error) => {
                    error_count += 1;
                    warn!("Error event({error_count}): {client_error}");
                    if error_count >= MAX_ERRORS {
                        warn!("Reached MAX_ERRORS, shutting down");
                        break;
                    }
                }
            },
            Err(e) => {
                warn!("Event receive error {e}");
                break;
            }
        }
    }

    running.store(false, Ordering::SeqCst);

    match pinger.shutdown() {
        Ok(()) => trace!("Pinger shut down corectly"),
        Err(e) => warn!("Pinger shutdown error: {e}"),
    }

    match quotes_listener.shutdown() {
        Ok(()) => trace!("Quotes listener shut down corectly"),
        Err(e) => warn!("Quotes listener shutdown error: {e}"),
    }

    Ok(())
}

fn setup_connection(server_address: SocketAddr) -> Result<TcpStream, ClientError> {
    debug!("Connecting to {}...", server_address);
    Ok(TcpStream::connect(server_address)?)
}

fn request_data(
    mut stream: TcpStream,
    local_port: u16,
    tickers: Vec<String>,
) -> Result<(), ClientError> {
    debug!(
        "Requesting data for tickers ({}) on port {local_port}",
        tickers.join(",")
    );

    stream.write_all(
        SubscribeMessage::new(
            SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), local_port),
            tickers,
        )
        .to_string()
        .as_bytes(),
    )?;

    Ok(())
}
