//! Stock quote module
use std::fmt::Display;

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

impl TryFrom<&str> for Quote {
    type Error = QuotesError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split('|').collect();
        if parts.len() == 4 {
            Ok(Quote {
                ticker: parts[0].to_string(),
                price: parts[1].parse()?,
                volume: parts[2].parse()?,
                timestamp: parts[3].parse()?,
            })
        } else {
            Err(QuotesError::ParseQuoteError(
                "Incorrect data format".to_string(),
            ))
        }
    }
}

impl Display for Quote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}",
            self.ticker, self.price, self.volume, self.timestamp
        )
    }
}
