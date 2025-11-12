//! Library with structs for stock quotes streaming

#![deny(unreachable_pub)]
#![warn(missing_docs)]

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use crate::error::QuotesError;

pub mod client_message;
pub mod error;
pub mod quote;
pub mod server_message;

/// Read tickers list from file, one ticker per line
pub fn read_tickers_from_file(file: PathBuf) -> Result<Vec<String>, QuotesError> {
    let file = File::open(file)?;

    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|res| {
            res.map(|line| line.trim().to_string())
                .map_err(QuotesError::from)
        })
        .collect()
}
