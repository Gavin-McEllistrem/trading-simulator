// Market data source abstraction and implementations

use async_trait::async_trait;
use crate::{MarketData, Result};

#[async_trait]
pub trait MarketDataSource: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn subscribe(&mut self, symbols: Vec<String>) -> Result<()>;
    async fn next_tick(&mut self) -> Result<MarketData>;
    async fn disconnect(&mut self) -> Result<()>;
    fn source_name(&self) -> &str;
}

// Module declarations - these will be implemented in subsequent days
pub mod simulated;
// pub mod csv;
// pub mod binance;
// pub mod alpaca;

// Re-exports
pub use simulated::SimulatedFeed;
