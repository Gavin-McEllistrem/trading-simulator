//! Configuration structures for the trading engine.
//!
//! This module provides configuration types for data sources, storage, and the overall
//! engine setup. Configurations can be loaded from TOML files or constructed programmatically.
//!
//! # Examples
//!
//! ## TOML Configuration
//!
//! ```toml
//! [data_source]
//! type = "binance"
//!
//! [data_source.binance]
//! symbols = ["BTCUSDT", "ETHUSDT"]
//! interval = "1m"
//!
//! [storage]
//! window_size = 1000
//! ```
//!
//! ## Programmatic Configuration
//!
//! ```
//! use trading_engine::config::{
//!     EngineConfig, DataSourceConfig, DataSourceType,
//!     DataSourceSpecific, BinanceConfig, StorageConfig
//! };
//!
//! let config = EngineConfig {
//!     data_source: DataSourceConfig {
//!         source_type: DataSourceType::Binance,
//!         specific: Some(DataSourceSpecific::Binance(BinanceConfig {
//!             symbols: vec!["BTCUSDT".to_string()],
//!             interval: "1m".to_string(),
//!         })),
//!     },
//!     storage: StorageConfig {
//!         window_size: 1000,
//!     },
//! };
//! ```

use serde::{Deserialize, Serialize};

/// Configuration for a market data source.
///
/// Specifies which data source to use and its specific configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct DataSourceConfig {
    /// Type of data source
    #[serde(rename = "type")]
    pub source_type: DataSourceType,

    /// Source-specific configuration
    #[serde(flatten)]
    pub specific: Option<DataSourceSpecific>,
}

/// Available data source types.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DataSourceType {
    /// Binance cryptocurrency exchange
    Binance,
    /// Alpaca stock market API
    Alpaca,
    /// Simulated random walk data
    Simulated,
    /// CSV file data source
    Csv,
}

/// Source-specific configuration variants.
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DataSourceSpecific {
    /// Binance configuration
    Binance(BinanceConfig),
    /// Alpaca configuration
    Alpaca(AlpacaConfig),
    /// Simulated feed configuration
    Simulated(SimulatedConfig),
    /// CSV file configuration
    Csv(CsvConfig),
}

/// Configuration for Binance WebSocket data source.
///
/// # Example
///
/// ```toml
/// [data_source.binance]
/// symbols = ["BTCUSDT", "ETHUSDT"]
/// interval = "1m"
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceConfig {
    /// Trading symbols to subscribe to (e.g., "BTCUSDT", "ETHUSDT")
    pub symbols: Vec<String>,
    /// Kline interval (1s, 1m, 5m, 15m, 30m, 1h, 4h, 1d, etc.)
    pub interval: String,
}

/// Configuration for Alpaca stock market data source.
///
/// # Example
///
/// ```toml
/// [data_source.alpaca]
/// api_key_env = "APCA_API_KEY_ID"
/// secret_key_env = "APCA_API_SECRET_KEY"
/// symbols = ["AAPL", "MSFT", "TSLA"]
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct AlpacaConfig {
    /// Environment variable name for API key
    pub api_key_env: String,
    /// Environment variable name for secret key
    pub secret_key_env: String,
    /// Stock symbols to subscribe to (e.g., "AAPL", "MSFT")
    pub symbols: Vec<String>,
}

/// Configuration for simulated data feed.
///
/// # Example
///
/// ```toml
/// [data_source.simulated]
/// symbol = "BTCUSDT"
/// starting_price = 50000.0
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct SimulatedConfig {
    /// Symbol name for simulated data
    pub symbol: String,
    /// Starting price for random walk
    pub starting_price: f64,
}

/// Configuration for CSV file data source.
///
/// # Example
///
/// ```toml
/// [data_source.csv]
/// path = "/path/to/market_data.csv"
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct CsvConfig {
    /// Path to CSV file containing market data
    pub path: String,
}

/// Configuration for market data storage.
///
/// # Example
///
/// ```toml
/// [storage]
/// window_size = 1000
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Maximum number of data points to keep per symbol
    pub window_size: usize,
}

/// Top-level engine configuration.
///
/// Combines data source and storage configurations.
///
/// # Example
///
/// ```toml
/// [data_source]
/// type = "binance"
///
/// [data_source.binance]
/// symbols = ["BTCUSDT", "ETHUSDT"]
/// interval = "1m"
///
/// [storage]
/// window_size = 1000
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct EngineConfig {
    /// Data source configuration
    pub data_source: DataSourceConfig,
    /// Storage configuration
    pub storage: StorageConfig,
}
