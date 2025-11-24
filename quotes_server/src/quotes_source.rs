use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossbeam_channel::{Receiver, unbounded};
use log::debug;
use quotes_lib::quote::Quote;

use crate::{error::ServerError, events::Event};

/// Simulate some data source that posts new quotes from stock
pub struct QuotesSource {
    tickers: Vec<String>,
    quotes: Arc<RwLock<HashMap<String, Quote>>>,
    thread_handle: Option<JoinHandle<Result<(), ServerError>>>,
}

impl QuotesSource {
    pub fn new(tickers: Vec<String>) -> Self {
        Self {
            tickers,
            quotes: Arc::new(RwLock::new(HashMap::new())),
            thread_handle: None,
        }
    }

    pub fn start(&mut self) -> Result<Receiver<Event>, ServerError> {
        if self.thread_handle.is_some() {
            return Err(ServerError::ComponentAlreadyStarted(
                "QuotesSource".to_string(),
            ));
        }

        let (tx, rx) = unbounded::<Event>();
        let interval = Duration::from_secs(1);
        let generator = QuotesGenerator {
            tickers: self.tickers.clone(),
        };
        let quotes = self.quotes.clone();

        let handle = thread::spawn(move || {
            debug!("Start QuotesSource loop");
            loop {
                let new_quotes = generator.generate_new_quotes();

                match quotes.write() {
                    Ok(mut lock) => *lock = new_quotes,
                    Err(_) => return Err(ServerError::QuotesSourceDataError),
                }

                if let Err(e) = tx.send(Event::QuotesUpdated) {
                    return Err(ServerError::from(e));
                }
                thread::sleep(interval);
            }
        });

        self.thread_handle = Some(handle);

        Ok(rx)
    }

    pub fn stop(&mut self) -> Result<(), ServerError> {
        if let Some(handle) = self.thread_handle.take() {
            handle.join().unwrap_or_else(|_| {
                Err(ServerError::ComponentStopError("QuotesSource".to_string()))
            })
        } else {
            Ok(())
        }
    }

    pub fn quotes(&self) -> &Arc<RwLock<HashMap<String, Quote>>> {
        &self.quotes
    }
}

struct QuotesGenerator {
    tickers: Vec<String>,
}

impl QuotesGenerator {
    fn generate_new_quote(ticker: String) -> Quote {
        Quote {
            ticker,
            price: 100f64 + rand::random::<f64>() * 50f64,
            volume: 1000 + (rand::random::<f32>() * 1000f32) as u32,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    fn generate_new_quotes(&self) -> HashMap<String, Quote> {
        HashMap::from_iter(
            self.tickers
                .iter()
                .map(|ticker| (ticker.clone(), Self::generate_new_quote(ticker.clone()))),
        )
    }
}
