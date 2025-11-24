use std::path::PathBuf;

use clap::Parser;
use crossbeam_channel::Select;
use env_logger::Builder;
use log::{LevelFilter, error, trace, warn};
use quotes_lib::read_tickers_from_file;

use crate::{
    clients_handler::ClientsHandler, error::ServerError, events::Event,
    quotes_source::QuotesSource, subscriptions_handler::SubscriptionsHandler,
};

mod clients_handler;
mod error;
mod events;
mod quotes_source;
mod single_client_handler;
mod subscriptions_handler;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 3000)]
    port: u16,
    #[arg(long, default_value = "all_tickers.txt")]
    tickers: PathBuf,
}

fn init_logger() -> Result<(), ServerError> {
    Builder::new()
        .filter_level(LevelFilter::Debug)
        .try_init()
        .map_err(ServerError::from)
}

fn main() {
    if let Err(e) = run_server() {
        error!("{e}");
    }
}

fn run_server() -> Result<(), ServerError> {
    init_logger()?;
    let args = Args::parse();

    let tickers = read_tickers_from_file(args.tickers)?;
    let mut quotes_source = QuotesSource::new(tickers);
    let mut subscriptions_handler = SubscriptionsHandler::new(args.port);
    let mut clients_handler = ClientsHandler::new(quotes_source.quotes().clone());

    if let Err(run_loop_error) = run_loop(
        &mut quotes_source,
        &mut subscriptions_handler,
        &mut clients_handler,
    ) {
        error!("Error in run_loop {run_loop_error}")
    } else {
        trace!("Server loop finished");
    }
    if let Err(stop_error) = clients_handler.stop() {
        error!("Error stopping clients_handler {stop_error}")
    } else {
        trace!("Clients handler stopped");
    }

    if let Err(stop_error) = subscriptions_handler.stop() {
        error!("Error stopping subscriptions_handler {stop_error}")
    } else {
        trace!("Subscriptions handler stopped");
    }

    if let Err(stop_error) = quotes_source.stop() {
        error!("Error stopping quotes_source {stop_error}")
    } else {
        trace!("Quotes source stopped");
    }

    Ok(())
}

fn run_loop(
    quotes_source: &mut QuotesSource,
    subscriptions_handler: &mut SubscriptionsHandler,
    clients_handler: &mut ClientsHandler,
) -> Result<(), ServerError> {
    let quotes_rx = quotes_source.start()?;
    let subscriptions_rx = subscriptions_handler.start()?;

    let mut select = Select::new();
    let quotes_index = select.recv(&quotes_rx);
    let subscriptions_index = select.recv(&subscriptions_rx);
    clients_handler.start()?;

    trace!("Starting server loop");
    loop {
        trace!("Wait for index");
        let index = select.ready();
        trace!("Event receiver index {index}");

        let event = match index {
            i if i == quotes_index => match quotes_rx.recv() {
                Ok(msg) => msg,
                Err(e) => {
                    return Err(ServerError::from(e));
                }
            },
            i if i == subscriptions_index => {
                let subscriptions_msg = subscriptions_rx.recv();
                trace!("Subscriptions msg {subscriptions_msg:?}");

                match subscriptions_msg {
                    Ok(msg) => msg,
                    Err(e) => {
                        return Err(ServerError::from(e));
                    }
                }
            }
            other => {
                error!("Unreacheable receiver index {other}");
                break;
            }
        };

        trace!("Event {event}");

        match event {
            Event::QuotesUpdated => {
                if let Err(e) = clients_handler.handle_quotes_updated() {
                    warn!("Error in handle_quotes_updated {e}");
                }
            }
            Event::NewClient(address, tickers) => {
                trace!("Event::NewClient {address} [{}]", tickers.join(","));
                if let Err(e) = clients_handler.handle_new_client(address, tickers) {
                    warn!("Error adding new client {e}");
                }
            }
            Event::Error(server_error) => warn!("Server error {server_error}"),
        }

        trace!("Loop end");
    }

    trace!("Loop completed");

    Ok(())
}
