// Trading Engine Library
// Core modules for the trading system

pub mod error;
pub mod market_data;
pub mod sources;
pub mod storage;
pub mod config;

// Re-export commonly used types
pub use error::{Result, TradingEngineError};
pub use market_data::{MarketData, MarketDataWindow};
pub use sources::{MarketDataSource, SimulatedFeed};
pub use storage::MarketDataStorage;
