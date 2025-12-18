//! Market data source abstraction and implementations.
//!
//! This module provides the [`MarketDataSource`] trait which defines a uniform interface
//! for all market data providers, along with concrete implementations for various sources.
//!
//! # Available Sources
//!
//! - [`SimulatedFeed`] - Random walk price generation for testing
//! - [`BinanceFeed`] - Real-time cryptocurrency data from Binance
//!
//! # The MarketDataSource Trait
//!
//! All data sources implement the async [`MarketDataSource`] trait which provides
//! a uniform interface for connecting to and receiving data from various market data providers.
//!
//! # Examples
//!
//! ## Using Simulated Data
//!
//! ```rust
//! use trading_engine::{MarketDataSource, sources::SimulatedFeed};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut feed = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);
//!
//!     feed.connect().await?;
//!     feed.subscribe(vec!["BTCUSDT".to_string()]).await?;
//!
//!     let data = feed.next_tick().await?;
//!     println!("Price: ${:.2}", data.close);
//!
//!     feed.disconnect().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Using Live Binance Data
//!
//! ```rust,no_run
//! use trading_engine::{MarketDataSource, sources::{BinanceFeed, BinanceRegion}};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut feed = BinanceFeed::new_with_region(
//!         vec!["BTCUSDT".to_string()],
//!         "1m".to_string(),
//!         BinanceRegion::US
//!     );
//!
//!     feed.connect().await?;
//!     feed.subscribe(vec!["BTCUSDT".to_string()]).await?;
//!
//!     let data = feed.next_tick().await?;
//!     println!("BTC: ${:.2}", data.close);
//!
//!     feed.disconnect().await?;
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use crate::{MarketData, Result};

/// Asynchronous market data source interface.
///
/// This trait defines the contract for all market data providers. Implementations
/// must be `Send + Sync` to allow use across async tasks and threads.
///
/// # Lifecycle
///
/// 1. Create the source (e.g., `BinanceFeed::new(...)`)
/// 2. Call [`connect()`](MarketDataSource::connect) to establish connection
/// 3. Call [`subscribe()`](MarketDataSource::subscribe) with symbols
/// 4. Repeatedly call [`next_tick()`](MarketDataSource::next_tick) to receive data
/// 5. Call [`disconnect()`](MarketDataSource::disconnect) to clean up
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` since they may be used across
/// async task boundaries and shared between threads.
///
/// # Examples
///
/// ```rust
/// use trading_engine::{MarketDataSource, sources::SimulatedFeed};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mut feed = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);
///
///     feed.connect().await?;
///     feed.subscribe(vec!["BTCUSDT".to_string()]).await?;
///
///     for i in 0..5 {
///         let data = feed.next_tick().await?;
///         println!("Tick {}: ${:.2}", i + 1, data.close);
///     }
///
///     feed.disconnect().await?;
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait MarketDataSource: Send + Sync {
    /// Establish connection to the data source.
    ///
    /// This should perform any necessary setup such as opening WebSocket connections,
    /// authenticating with APIs, or opening file handles.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection cannot be established.
    async fn connect(&mut self) -> Result<()>;

    /// Subscribe to market data for specified symbols.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of trading symbols to subscribe to
    ///
    /// # Errors
    ///
    /// Returns an error if subscription fails or symbols are invalid.
    async fn subscribe(&mut self, symbols: Vec<String>) -> Result<()>;

    /// Get the next market data tick.
    ///
    /// This method blocks until new data is available. For real-time sources,
    /// this waits for the next update. For simulated sources, this may include
    /// a delay to simulate real market timing.
    ///
    /// # Returns
    ///
    /// The next available market data point.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The connection is lost
    /// - Data cannot be parsed
    /// - A timeout occurs
    async fn next_tick(&mut self) -> Result<MarketData>;

    /// Disconnect from the data source.
    ///
    /// This should clean up all resources including closing connections,
    /// file handles, etc.
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup fails (though the connection will still be closed).
    async fn disconnect(&mut self) -> Result<()>;

    /// Get the name of this data source.
    ///
    /// # Returns
    ///
    /// A string identifier for this source (e.g., "binance", "simulated").
    fn source_name(&self) -> &str;
}

// Module declarations
pub mod simulated;
pub mod binance;
// pub mod csv;
// pub mod alpaca;

// Re-exports
pub use simulated::SimulatedFeed;
pub use binance::{BinanceFeed, BinanceRegion};
