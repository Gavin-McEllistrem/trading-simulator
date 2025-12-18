//! Trading Engine Library
//!
//! A high-performance trading system engine with support for real-time market data,
//! technical indicators, and automated trading strategies.
//!
//! # Overview
//!
//! This library provides the core components for building a trading system:
//!
//! - **Market Data**: Structures for OHLCV candlestick data with bid/ask prices
//! - **Data Sources**: Pluggable data feeds (Binance, simulated, etc.)
//! - **Storage**: Thread-safe multi-symbol data storage
//! - **Error Handling**: Comprehensive error types with context
//!
//! # Architecture
//!
//! The system follows a layered architecture:
//!
//! 1. **Data Sources** ([`sources`]) - Async market data feeds
//! 2. **Storage** ([`storage`]) - Thread-safe data management
//! 3. **Indicators** ([`indicators`]) - Technical analysis (SMA, EMA, RSI, MACD, Bollinger Bands)
//! 4. **State Machine** ([`state_machine`]) - Trading state management and position tracking
//! 5. **Strategies** (Phase 4) - Trading logic via Lua
//!
//! # Quick Start
//!
//! ## Simulated Data
//!
//! ```rust
//! use trading_engine::{MarketDataSource, SimulatedFeed, MarketDataStorage};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create simulated feed
//!     let mut feed = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);
//!     let storage = MarketDataStorage::new(100);
//!
//!     // Connect and collect data
//!     feed.connect().await?;
//!     feed.subscribe(vec!["BTCUSDT".to_string()]).await?;
//!
//!     for _ in 0..10 {
//!         let data = feed.next_tick().await?;
//!         storage.push(data);
//!     }
//!
//!     feed.disconnect().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Live Binance Data
//!
//! ```rust,no_run
//! use trading_engine::{
//!     MarketDataSource,
//!     MarketDataStorage,
//!     sources::{BinanceFeed, BinanceRegion},
//! };
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create Binance feed
//!     let mut feed = BinanceFeed::new_with_region(
//!         vec!["BTCUSDT".to_string()],
//!         "1m".to_string(),
//!         BinanceRegion::US
//!     );
//!     let storage = MarketDataStorage::new(1000);
//!
//!     // Connect and collect
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
//!
//! # Modules
//!
//! - [`error`] - Error types and result aliases
//! - [`market_data`] - OHLCV data structures and windows
//! - [`sources`] - Market data source implementations
//! - [`storage`] - Thread-safe multi-symbol storage
//! - [`config`] - Configuration structures
//! - [`indicators`] - Technical indicators (SMA, EMA, RSI, MACD, Bollinger Bands)
//! - [`state_machine`] - Trading state machine and position tracking

pub mod error;
pub mod market_data;
pub mod sources;
pub mod storage;
pub mod config;
pub mod indicators;
pub mod state_machine;

// Re-export commonly used types
pub use error::{Result, TradingEngineError};
pub use market_data::{MarketData, MarketDataWindow};
pub use sources::{MarketDataSource, SimulatedFeed};
pub use storage::MarketDataStorage;
