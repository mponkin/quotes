//! Stock quote module
use std::{array::TryFromSliceError, fmt::Display};

use crate::error::QuotesError;

/// Quote structure
#[derive(Debug, Clone)]
pub struct Quote {
    /// Ticker
    pub ticker: String,
    /// Last price
    pub price: f64,
    /// Volume in units
    pub volume: u32,
    /// Unix timestamp in millis
    pub timestamp: u64,
}

impl Display for Quote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Quote {} price: {}, volume: {}, timestamp: {}",
            self.ticker, self.price, self.volume, self.timestamp
        )
    }
}

impl Quote {
    const SPLITTER: u8 = b'|';
}

impl Into<Vec<u8>> for &Quote {
    fn into(self) -> Vec<u8> {
        let mut data = vec![];
        data.extend_from_slice(self.ticker.as_bytes());
        data.push(Quote::SPLITTER);
        data.extend_from_slice(&self.price.to_be_bytes());
        data.push(Quote::SPLITTER);
        data.extend_from_slice(&self.volume.to_be_bytes());
        data.push(Quote::SPLITTER);
        data.extend_from_slice(&self.timestamp.to_be_bytes());

        data
    }
}

macro_rules! slice_as_bytes {
    ($slice:expr, $count:expr) => {{
        let bytes: Result<[u8; $count], QuotesError> = $slice
            .try_into()
            .map_err(|e: TryFromSliceError| QuotesError::ParseQuoteError(e.to_string()));

        bytes
    }};
}

impl TryFrom<&[u8]> for Quote {
    type Error = QuotesError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let parts = value
            .split(|b| *b == Quote::SPLITTER)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();

        if parts.len() != 4
            || parts[0].len() < 1
            || parts[1].len() != 8
            || parts[2].len() != 4
            || parts[3].len() != 8
        {
            return Err(QuotesError::ParseQuoteError(
                "Incorrect data format".to_string(),
            ));
        }

        let ticker_bytes: Vec<u8> = parts[0].iter().copied().collect();

        let price_pytes = slice_as_bytes!(parts[1], 8)?;
        let volume_pytes = slice_as_bytes!(parts[2], 4)?;
        let timestamp_pytes = slice_as_bytes!(parts[3], 8)?;

        Ok(Self {
            ticker: String::from_utf8(ticker_bytes)
                .map_err(|e| QuotesError::ParseQuoteError(e.to_string()))?,
            price: f64::from_be_bytes(price_pytes),
            volume: u32::from_be_bytes(volume_pytes),
            timestamp: u64::from_be_bytes(timestamp_pytes),
        })
    }
}
