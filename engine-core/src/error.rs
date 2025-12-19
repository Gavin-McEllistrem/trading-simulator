//! Error handling for the trading engine.
//!
//! This module provides a unified error type [`TradingEngineError`] that covers
//! all error conditions that can occur in the trading engine, from network
//! failures to data validation errors.
//!
//! # Examples
//!
//! ```
//! use trading_engine::{Result, TradingEngineError};
//!
//! fn validate_price(price: f64) -> Result<()> {
//!     if price <= 0.0 {
//!         return Err(TradingEngineError::InvalidData(
//!             "Price must be positive".to_string()
//!         ));
//!     }
//!     Ok(())
//! }
//! ```

use thiserror::Error;

/// Error type for all trading engine operations.
///
/// This enum covers various error conditions including network failures,
/// data validation errors, and configuration problems. It uses [`thiserror`]
/// to provide good error messages and automatic conversions from underlying
/// error types.
#[derive(Error, Debug)]
pub enum TradingEngineError {
    /// WebSocket connection or communication error.
    ///
    /// This error occurs when there are problems with WebSocket connections
    /// to data sources (e.g., Binance, Alpaca).
    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    /// WebSocket library error
    #[error("WebSocket connection failed: {0}")]
    TungsteniteError(#[from] tokio_tungstenite::tungstenite::Error),

    /// Attempted operation on a disconnected data source.
    ///
    /// This error occurs when trying to use a data source that hasn't been
    /// connected via [`MarketDataSource::connect`](crate::sources::MarketDataSource::connect).
    #[error("Data source not connected")]
    NotConnected,

    /// Market data failed validation checks.
    ///
    /// This error occurs when received market data has inconsistencies
    /// (e.g., high < low, negative prices).
    #[error("Invalid market data: {0}")]
    InvalidData(String),

    /// Reconnection attempts exhausted.
    ///
    /// This error occurs when a data source fails to reconnect after
    /// the maximum number of retry attempts.
    #[error("Reconnection failed after {0} attempts")]
    ReconnectionFailed(u32),

    /// Failed to parse data from an external source.
    ///
    /// This error occurs when JSON or other format parsing fails,
    /// typically from WebSocket messages or API responses.
    #[error("Parsing error: {0}")]
    ParseError(String),

    /// File I/O error.
    ///
    /// This error occurs during file operations like reading configuration
    /// files or loading historical data.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    ///
    /// This error occurs when JSON parsing or serialization fails.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Configuration validation error.
    ///
    /// This error occurs when configuration files are malformed or
    /// contain invalid values.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Lua strategy error.
    ///
    /// This error occurs when Lua script execution fails or returns
    /// invalid values.
    #[error("Strategy error: {0}")]
    StrategyError(String),

    /// Lua runtime error.
    ///
    /// This error occurs when the Lua VM encounters an error during
    /// script execution.
    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::Error),

    /// Runner with this ID already exists.
    ///
    /// This error occurs when trying to add a runner with an ID that's
    /// already in use.
    #[error("Runner already exists: {0}")]
    RunnerAlreadyExists(String),

    /// Runner with this ID was not found.
    ///
    /// This error occurs when trying to remove or access a runner that
    /// doesn't exist.
    #[error("Runner not found: {0}")]
    RunnerNotFound(String),

    /// No runners are watching this symbol.
    ///
    /// This error occurs when trying to feed data to a symbol that has
    /// no active runners.
    #[error("No runners for symbol: {0}")]
    NoRunnersForSymbol(String),

    /// Channel closed unexpectedly.
    ///
    /// This error occurs when a runner's data channel closes prematurely.
    #[error("Channel closed for runner: {0}")]
    ChannelClosed(String),

    /// Runner task panicked.
    ///
    /// This error occurs when a runner's background task panics.
    #[error("Runner task panicked: {0}")]
    TaskPanic(String),
}

/// Convenience type alias for Results using [`TradingEngineError`].
///
/// This type alias is used throughout the trading engine codebase to simplify
/// function signatures.
///
/// # Examples
///
/// ```
/// use trading_engine::Result;
///
/// fn do_something() -> Result<i32> {
///     Ok(42)
/// }
/// ```
pub type Result<T> = std::result::Result<T, TradingEngineError>;
