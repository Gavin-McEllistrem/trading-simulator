//! Symbol Runner - Per-Symbol Trading Orchestration
//!
//! This module provides the `SymbolRunner` struct that orchestrates all components
//! for a single trading symbol: market data, indicators, state machine, and strategy.
//!
//! # Architecture
//!
//! ```text
//! SymbolRunner (per-symbol thread)
//!   ├── MarketDataWindow   (price history)
//!   ├── StateMachine       (state & position)
//!   ├── LuaStrategy        (trading logic)
//!   └── IndicatorApi       (technical analysis)
//! ```
//!
//! # Example
//!
//! ```no_run
//! use trading_engine::runner::SymbolRunner;
//! use trading_engine::strategy::LuaStrategy;
//! use tokio::sync::mpsc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create market data channel
//!     let (tx, rx) = mpsc::unbounded_channel();
//!
//!     // Load strategy
//!     let strategy = LuaStrategy::new("strategies/ema_crossover.lua")?;
//!
//!     // Create runner
//!     let mut runner = SymbolRunner::new(
//!         "BTCUSDT".to_string(),
//!         strategy,
//!         rx,
//!         50, // window size
//!     );
//!
//!     // Run in background
//!     tokio::spawn(async move {
//!         runner.run().await
//!     });
//!
//!     Ok(())
//! }
//! ```

use crate::error::Result;
use crate::market_data::{MarketData, MarketDataWindow};
use crate::state_machine::{Action, State, StateMachine};
use crate::strategy::{IndicatorApi, LuaStrategy};
use tokio::sync::mpsc;
use std::time::Instant;

mod config;
mod stats;
mod engine;

pub use config::RunnerConfig;
pub use stats::RunnerStats;
pub use engine::TradingEngine;

/// Per-symbol trading orchestrator
///
/// `SymbolRunner` manages the complete trading loop for a single symbol,
/// coordinating market data, indicators, state machine, and strategy execution.
///
/// Each runner typically runs in its own Tokio task, receiving market data
/// via a channel and executing trades independently.
pub struct SymbolRunner {
    /// Symbol being traded (e.g., "BTCUSDT")
    symbol: String,

    /// Market data window (circular buffer)
    window: MarketDataWindow,

    /// State machine (state & position management)
    state_machine: StateMachine,

    /// Lua strategy (trading logic)
    strategy: LuaStrategy,

    /// Channel receiver for market data
    data_receiver: mpsc::UnboundedReceiver<MarketData>,

    /// Configuration
    config: RunnerConfig,

    /// Statistics
    stats: RunnerStats,

    /// Start time
    start_time: Instant,
}

impl SymbolRunner {
    /// Create a new symbol runner
    ///
    /// # Arguments
    ///
    /// * `symbol` - Symbol to trade (e.g., "BTCUSDT")
    /// * `strategy` - Lua strategy for trading logic
    /// * `data_receiver` - Channel to receive market data updates
    /// * `window_size` - Size of the circular buffer for market data
    ///
    /// # Example
    ///
    /// ```no_run
    /// use trading_engine::runner::SymbolRunner;
    /// use trading_engine::strategy::LuaStrategy;
    /// use tokio::sync::mpsc;
    ///
    /// let (tx, rx) = mpsc::unbounded_channel();
    /// let strategy = LuaStrategy::new("strategies/ema_crossover.lua")?;
    /// let runner = SymbolRunner::new("BTCUSDT".to_string(), strategy, rx, 50);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(
        symbol: String,
        strategy: LuaStrategy,
        data_receiver: mpsc::UnboundedReceiver<MarketData>,
        window_size: usize,
    ) -> Self {
        let state_machine = StateMachine::new(symbol.clone());
        let window = MarketDataWindow::new(window_size);

        Self {
            symbol,
            window,
            state_machine,
            strategy,
            data_receiver,
            config: RunnerConfig::default(),
            stats: RunnerStats::new(),
            start_time: Instant::now(),
        }
    }

    /// Create a runner with custom configuration
    pub fn with_config(mut self, config: RunnerConfig) -> Self {
        self.config = config;
        self
    }

    /// Get the symbol being traded
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Get the current state
    pub fn state(&self) -> State {
        *self.state_machine.current_state()
    }

    /// Get runner statistics
    pub fn stats(&self) -> &RunnerStats {
        &self.stats
    }

    /// Run the trading loop
    ///
    /// This is the main loop that:
    /// 1. Receives market data from the channel
    /// 2. Updates the market data window
    /// 3. Calls the strategy based on current state
    /// 4. Executes actions returned by the strategy
    /// 5. Updates the state machine
    ///
    /// The loop runs until the channel is closed or an unrecoverable error occurs.
    pub async fn run(&mut self) -> Result<()> {
        tracing::info!("Starting SymbolRunner for {}", self.symbol);

        loop {
            // Receive next market data
            let market_data = match self.data_receiver.recv().await {
                Some(data) => data,
                None => {
                    tracing::info!("Channel closed for {}, shutting down", self.symbol);
                    break;
                }
            };

            // Validate symbol matches
            if market_data.symbol != self.symbol {
                tracing::warn!(
                    "Received data for {} but runner is for {}",
                    market_data.symbol,
                    self.symbol
                );
                continue;
            }

            // Process the tick
            if let Err(e) = self.process_tick(market_data).await {
                tracing::error!("Error processing tick for {}: {}", self.symbol, e);

                if self.config.stop_on_error {
                    return Err(e);
                }

                // Continue on error if configured
                self.stats.record_error();
            }
        }

        tracing::info!("SymbolRunner for {} stopped", self.symbol);
        Ok(())
    }

    /// Process a single market data tick
    async fn process_tick(&mut self, market_data: MarketData) -> Result<()> {
        let tick_start = Instant::now();

        // Update window
        self.window.push(market_data.clone());

        // Update context with latest data
        self.state_machine
            .context_mut()
            .set("latest_price", market_data.close);
        self.state_machine
            .context_mut()
            .set("latest_timestamp", market_data.timestamp);

        // Create indicator API
        let indicator_api = IndicatorApi::new(self.window.clone());

        // Call strategy based on current state
        let action = match self.state_machine.current_state() {
            State::Idle => self.handle_idle(&market_data, &indicator_api)?,
            State::Analyzing => self.handle_analyzing(&market_data, &indicator_api)?,
            State::InPosition => self.handle_in_position(&market_data, &indicator_api)?,
        };

        // Execute action if returned
        if let Some(act) = action {
            if self.config.log_actions {
                tracing::info!("Symbol {}: Executing action: {:?}", self.symbol, act);
            }
            self.state_machine.execute(act)?;
            self.stats.record_action();
        }

        // Update state machine (handles auto-exits)
        self.state_machine.update(&market_data);

        // Record statistics
        let tick_duration = tick_start.elapsed();
        self.stats.record_tick(tick_duration);

        // Log position updates
        if self.config.log_positions {
            if let Some(position) = self.state_machine.position() {
                if let Some(pnl) = position.unrealized_pnl() {
                    tracing::debug!(
                        "Symbol {}: Position {} @ ${:.2}, P&L: ${:.2}",
                        self.symbol,
                        position.side(),
                        position.entry_price(),
                        pnl
                    );
                }
            }
        }

        Ok(())
    }

    /// Handle Idle state - look for opportunities
    fn handle_idle(
        &mut self,
        market_data: &MarketData,
        indicator_api: &IndicatorApi,
    ) -> Result<Option<Action>> {
        let opportunity = self.strategy.detect_opportunity(
            market_data,
            self.state_machine.context(),
            indicator_api,
        )?;

        if let Some(opp_table) = opportunity {
            // Update context with opportunity data
            // Extract common fields if they exist
            if let Ok(signal) = opp_table.get::<_, String>("signal") {
                self.state_machine.context_mut().set("signal", signal);
            }
            if let Ok(confidence) = opp_table.get::<_, f64>("confidence") {
                self.state_machine
                    .context_mut()
                    .set("confidence", confidence);
            }

            // Transition to Analyzing
            Ok(Some(Action::StartAnalyzing {
                reason: "Strategy detected opportunity".to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Handle Analyzing state - decide on entry
    fn handle_analyzing(
        &mut self,
        market_data: &MarketData,
        indicator_api: &IndicatorApi,
    ) -> Result<Option<Action>> {
        self.strategy.filter_commitment(
            market_data,
            self.state_machine.context(),
            indicator_api,
        )
    }

    /// Handle InPosition state - manage the trade
    fn handle_in_position(
        &mut self,
        market_data: &MarketData,
        indicator_api: &IndicatorApi,
    ) -> Result<Option<Action>> {
        self.strategy.manage_position(
            market_data,
            self.state_machine.context(),
            indicator_api,
        )
    }

    /// Get mutable access to context (for testing)
    #[cfg(test)]
    pub fn context_mut(&mut self) -> &mut crate::state_machine::Context {
        self.state_machine.context_mut()
    }

    /// Get current position (for monitoring)
    pub fn position(&self) -> Option<&crate::state_machine::Position> {
        self.state_machine.position()
    }

    /// Get uptime
    pub fn uptime(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market_data::MarketData;

    fn create_test_data(close: f64) -> MarketData {
        MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1234567890,
            open: close - 10.0,
            high: close + 10.0,
            low: close - 20.0,
            close,
            volume: 1000,
            bid: close - 5.0,
            ask: close + 5.0,
        }
    }

    #[tokio::test]
    async fn test_runner_creation() {
        let (_tx, rx) = mpsc::unbounded_channel();
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        let runner = SymbolRunner::new("BTCUSDT".to_string(), strategy, rx, 50);

        assert_eq!(runner.symbol(), "BTCUSDT");
        assert_eq!(runner.state(), State::Idle);
    }

    #[tokio::test]
    async fn test_runner_receives_data() {
        let (tx, rx) = mpsc::unbounded_channel();
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        let mut runner = SymbolRunner::new("BTCUSDT".to_string(), strategy, rx, 50);

        // Send data
        tx.send(create_test_data(50000.0)).unwrap();

        // Process one tick manually
        let data = runner.data_receiver.recv().await.unwrap();
        runner.process_tick(data).await.unwrap();

        // Check window has data
        assert_eq!(runner.window.len(), 1);
    }
}
