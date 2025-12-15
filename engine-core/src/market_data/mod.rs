//! Market data structures and utilities.
//!
//! This module provides core data structures for handling market data including
//! OHLCV (Open, High, Low, Close, Volume) candlesticks and time-series windows.
//!
//! # Overview
//!
//! The main types in this module are:
//! - [`MarketData`]: Represents a single candlestick/bar
//! - [`MarketDataWindow`]: A circular buffer for storing recent market data
//!
//! # Examples
//!
//! ```
//! use trading_engine::MarketData;
//!
//! let data = MarketData {
//!     symbol: "BTCUSDT".to_string(),
//!     timestamp: 1234567890,
//!     open: 50000.0,
//!     high: 51000.0,
//!     low: 49500.0,
//!     close: 50500.0,
//!     volume: 1000,
//!     bid: 50499.0,
//!     ask: 50501.0,
//! };
//!
//! assert_eq!(data.mid_price(), 50500.0);
//! data.validate().unwrap();
//! ```

use serde::{Deserialize, Serialize};

/// Represents a single market data point (candlestick/bar).
///
/// Contains OHLCV (Open, High, Low, Close, Volume) data plus bid/ask prices
/// for a given symbol at a specific timestamp. This is the fundamental data
/// structure used throughout the trading engine.
///
/// # Fields
///
/// * `symbol` - Trading pair symbol (e.g., "BTCUSDT", "AAPL", "ETHUSDT")
/// * `timestamp` - Unix timestamp in milliseconds when this bar opened
/// * `open` - Opening price for the period
/// * `high` - Highest price during the period
/// * `low` - Lowest price during the period
/// * `close` - Closing price for the period
/// * `volume` - Total volume traded during the period
/// * `bid` - Current bid price (best buy price)
/// * `ask` - Current ask price (best sell price)
///
/// # Thread Safety
///
/// This struct is `Clone`, `Send`, and `Sync`, making it safe to share across threads.
///
/// # Serialization
///
/// Supports JSON serialization via serde for logging and storage.
///
/// # Examples
///
/// ```
/// use trading_engine::MarketData;
///
/// let data = MarketData {
///     symbol: "BTCUSDT".to_string(),
///     timestamp: 1734278400000, // 2024-12-15
///     open: 50000.0,
///     high: 51000.0,
///     low: 49500.0,
///     close: 50500.0,
///     volume: 1000,
///     bid: 50499.0,
///     ask: 50501.0,
/// };
///
/// // Calculate mid-price
/// assert_eq!(data.mid_price(), 50500.0);
///
/// // Validate consistency
/// assert!(data.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub bid: f64,
    pub ask: f64,
}

impl MarketData {
    /// Calculates the mid-price between bid and ask.
    ///
    /// The mid-price is the average of the best bid and ask prices,
    /// often used as a fair value estimate.
    ///
    /// # Returns
    ///
    /// The average of [`bid`](Self::bid) and [`ask`](Self::ask) prices.
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::MarketData;
    ///
    /// let data = MarketData {
    ///     symbol: "BTCUSDT".to_string(),
    ///     timestamp: 0,
    ///     open: 0.0, high: 0.0, low: 0.0, close: 0.0,
    ///     volume: 0,
    ///     bid: 100.0,
    ///     ask: 102.0,
    /// };
    ///
    /// assert_eq!(data.mid_price(), 101.0);
    /// ```
    pub fn mid_price(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }

    /// Validates the market data for consistency.
    ///
    /// Checks that the data satisfies basic invariants:
    /// - High price >= Low price
    /// - Prices are positive (> 0)
    ///
    /// Also logs a warning if volume is zero.
    ///
    /// # Errors
    ///
    /// Returns [`TradingEngineError::InvalidData`](crate::TradingEngineError::InvalidData) if:
    /// - `high < low` (impossible for valid OHLC data)
    /// - `open <= 0.0` or `close <= 0.0` (prices must be positive)
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::{MarketData, TradingEngineError};
    ///
    /// // Valid data
    /// let valid = MarketData {
    ///     symbol: "BTC".to_string(),
    ///     timestamp: 0,
    ///     open: 100.0,
    ///     high: 110.0,
    ///     low: 90.0,
    ///     close: 105.0,
    ///     volume: 1000,
    ///     bid: 104.0,
    ///     ask: 106.0,
    /// };
    /// assert!(valid.validate().is_ok());
    ///
    /// // Invalid data (high < low)
    /// let invalid = MarketData {
    ///     symbol: "BTC".to_string(),
    ///     timestamp: 0,
    ///     open: 100.0,
    ///     high: 90.0,  // Invalid!
    ///     low: 110.0,   // Invalid!
    ///     close: 105.0,
    ///     volume: 1000,
    ///     bid: 104.0,
    ///     ask: 106.0,
    /// };
    /// assert!(invalid.validate().is_err());
    /// ```
    pub fn validate(&self) -> crate::Result<()> {
        if self.high < self.low {
            return Err(crate::TradingEngineError::InvalidData(
                format!("Invalid OHLC: high ({}) < low ({})", self.high, self.low)
            ));
        }
        if self.open <= 0.0 || self.close <= 0.0 {
            return Err(crate::TradingEngineError::InvalidData(
                "Price values must be positive".to_string()
            ));
        }
        if self.volume == 0 {
            tracing::warn!("Zero volume bar for {}", self.symbol);
        }
        Ok(())
    }
}

// Re-export window module
pub mod window;
pub use window::MarketDataWindow;
