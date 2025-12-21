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
use crate::events::{ErrorSeverity, RunnerEvent};
use crate::market_data::{MarketData, MarketDataWindow};
use crate::state_machine::{Action, State, StateMachine};
use crate::strategy::{IndicatorApi, LuaStrategy};
use tokio::sync::mpsc;
use std::time::Instant;

mod config;
mod stats;
mod engine;
mod snapshot;

pub use config::RunnerConfig;
pub use stats::RunnerStats;
pub use engine::TradingEngine;
pub use snapshot::{RunnerCommand, RunnerSnapshot, ContextSnapshot, RunnerStatus};

/// Per-symbol trading orchestrator
///
/// `SymbolRunner` manages the complete trading loop for a single symbol,
/// coordinating market data, indicators, state machine, and strategy execution.
///
/// Each runner typically runs in its own Tokio task, receiving market data
/// via a channel and executing trades independently.
pub struct SymbolRunner {
    /// Unique runner ID (e.g., "btc_ema_prod")
    runner_id: String,

    /// Symbol being traded (e.g., "BTCUSDT")
    symbol: String,

    /// Runner execution status
    status: RunnerStatus,

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

    /// Optional event channel for real-time updates
    event_tx: Option<mpsc::UnboundedSender<RunnerEvent>>,

    /// Optional command channel for state introspection
    command_rx: Option<mpsc::UnboundedReceiver<RunnerCommand>>,
}

impl SymbolRunner {
    /// Create a new symbol runner
    ///
    /// # Arguments
    ///
    /// * `runner_id` - Unique identifier for this runner
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
    /// let runner = SymbolRunner::new(
    ///     "btc_ema".to_string(),
    ///     "BTCUSDT".to_string(),
    ///     strategy,
    ///     rx,
    ///     50
    /// );
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(
        runner_id: String,
        symbol: String,
        strategy: LuaStrategy,
        data_receiver: mpsc::UnboundedReceiver<MarketData>,
        window_size: usize,
    ) -> Self {
        let state_machine = StateMachine::new(symbol.clone());
        let window = MarketDataWindow::new(window_size);

        Self {
            runner_id,
            symbol,
            status: RunnerStatus::default(),
            window,
            state_machine,
            strategy,
            data_receiver,
            config: RunnerConfig::default(),
            stats: RunnerStats::new(),
            start_time: Instant::now(),
            event_tx: None,
            command_rx: None,
        }
    }

    /// Create a runner with custom configuration
    pub fn with_config(mut self, config: RunnerConfig) -> Self {
        self.config = config;
        self
    }

    /// Add an event channel for real-time updates
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use trading_engine::runner::SymbolRunner;
    /// # use trading_engine::strategy::LuaStrategy;
    /// # use tokio::sync::mpsc;
    /// # let (data_tx, data_rx) = mpsc::unbounded_channel();
    /// # let strategy = LuaStrategy::new("test.lua").unwrap();
    /// let (event_tx, event_rx) = mpsc::unbounded_channel();
    /// let runner = SymbolRunner::new("id".to_string(), "BTC".to_string(), strategy, data_rx, 50)
    ///     .with_event_channel(event_tx);
    /// ```
    pub fn with_event_channel(mut self, tx: mpsc::UnboundedSender<RunnerEvent>) -> Self {
        self.event_tx = Some(tx);
        self
    }

    /// Add a command channel for state introspection
    ///
    /// Allows external systems to query runner state on-demand.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use trading_engine::runner::SymbolRunner;
    /// # use trading_engine::strategy::LuaStrategy;
    /// # use tokio::sync::mpsc;
    /// # let (data_tx, data_rx) = mpsc::unbounded_channel();
    /// # let strategy = LuaStrategy::new("test.lua").unwrap();
    /// let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    /// let runner = SymbolRunner::new("id".to_string(), "BTC".to_string(), strategy, data_rx, 50)
    ///     .with_command_channel(cmd_rx);
    /// ```
    pub fn with_command_channel(mut self, rx: mpsc::UnboundedReceiver<RunnerCommand>) -> Self {
        self.command_rx = Some(rx);
        self
    }

    /// Get the runner ID
    pub fn runner_id(&self) -> &str {
        &self.runner_id
    }

    /// Get the symbol being traded
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Emit an event (if event channel is configured)
    fn emit_event(&self, event: RunnerEvent) {
        if let Some(tx) = &self.event_tx {
            // Ignore send errors (subscriber may have disconnected)
            let _ = tx.send(event);
        }
    }

    /// Handle introspection and control commands
    fn handle_command(&mut self, cmd: RunnerCommand) {
        match cmd {
            RunnerCommand::GetSnapshot { response } => {
                let snapshot = self.create_snapshot();
                // Ignore send errors (requester may have cancelled)
                let _ = response.send(snapshot);
            }
            RunnerCommand::GetPriceHistory { count, response } => {
                let history = self.get_price_history(count);
                let _ = response.send(history);
            }
            RunnerCommand::Pause { response } => {
                let success = if self.status.is_active() {
                    self.status = RunnerStatus::Paused;
                    tracing::info!("Runner {} paused", self.runner_id);
                    true
                } else {
                    false
                };
                let _ = response.send(success);
            }
            RunnerCommand::Resume { response } => {
                let success = if self.status.is_paused() {
                    self.status = RunnerStatus::Running;
                    tracing::info!("Runner {} resumed", self.runner_id);
                    true
                } else {
                    false
                };
                let _ = response.send(success);
            }
            RunnerCommand::Stop { response } => {
                self.status = RunnerStatus::Stopped;
                tracing::info!("Runner {} stopped", self.runner_id);
                let _ = response.send(true);
            }
        }
    }

    /// Create a snapshot of the current runner state
    fn create_snapshot(&self) -> RunnerSnapshot {
        RunnerSnapshot::new(
            self.runner_id.clone(),
            self.symbol.clone(),
            self.status,
            *self.state_machine.current_state(),
            self.state_machine.position().cloned(),
            self.create_context_snapshot(),
            self.stats.clone(),
            self.start_time.elapsed(),
        )
    }

    /// Create a snapshot of the strategy context
    fn create_context_snapshot(&self) -> ContextSnapshot {
        let context = self.state_machine.context();

        ContextSnapshot {
            strings: context.strings.clone(),
            numbers: context.numbers.clone(),
            integers: context.integers.clone(),
            booleans: context.booleans.clone(),
        }
    }

    /// Get recent price history from the data window
    fn get_price_history(&self, count: Option<usize>) -> Vec<MarketData> {
        let window_data: Vec<MarketData> = self.window.iter().cloned().collect();

        match count {
            Some(n) => {
                // Return last N elements
                let start = if window_data.len() > n {
                    window_data.len() - n
                } else {
                    0
                };
                window_data[start..].to_vec()
            }
            None => window_data,
        }
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
            tokio::select! {
                // Handle incoming market data
                data_result = self.data_receiver.recv() => {
                    let market_data = match data_result {
                        Some(data) => data,
                        None => {
                            tracing::info!("Channel closed for {}, shutting down", self.symbol);
                            break;
                        }
                    };

                    // Check if stopped
                    if self.status.is_stopped() {
                        tracing::info!("Runner {} is stopped, exiting", self.runner_id);
                        break;
                    }

                    // Validate symbol matches
                    if market_data.symbol != self.symbol {
                        tracing::warn!(
                            "Received data for {} but runner is for {}",
                            market_data.symbol,
                            self.symbol
                        );
                        continue;
                    }

                    // Skip tick processing if paused
                    if !self.status.is_active() {
                        continue;
                    }

                    // Process the tick
                    if let Err(e) = self.process_tick(market_data.clone()).await {
                        tracing::error!("Error processing tick for {}: {}", self.symbol, e);

                        // Emit error event
                        let severity = if self.config.stop_on_error {
                            ErrorSeverity::Critical
                        } else {
                            ErrorSeverity::Error
                        };

                        self.emit_event(RunnerEvent::Error {
                            runner_id: self.runner_id.clone(),
                            error: e.to_string(),
                            severity,
                            timestamp: market_data.timestamp,
                        });

                        if self.config.stop_on_error {
                            return Err(e);
                        }

                        // Continue on error if configured
                        self.stats.record_error();
                    }
                },

                // Handle introspection commands
                cmd_result = async {
                    match &mut self.command_rx {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await, // Never resolves if no command channel
                    }
                } => {
                    if let Some(cmd) = cmd_result {
                        self.handle_command(cmd);
                    }
                }
            }
        }

        tracing::info!("SymbolRunner for {} stopped", self.symbol);
        Ok(())
    }

    /// Process a single market data tick
    async fn process_tick(&mut self, market_data: MarketData) -> Result<()> {
        let tick_start = Instant::now();

        // Emit tick received event
        self.emit_event(RunnerEvent::TickReceived {
            runner_id: self.runner_id.clone(),
            symbol: self.symbol.clone(),
            data: market_data.clone(),
        });

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

        // Track state before strategy execution
        let state_before = *self.state_machine.current_state();

        // Call strategy based on current state
        let action = match self.state_machine.current_state() {
            State::Idle => self.handle_idle(&market_data, &indicator_api)?,
            State::Analyzing => self.handle_analyzing(&market_data, &indicator_api)?,
            State::InPosition => self.handle_in_position(&market_data, &indicator_api)?,
        };

        // Execute action if returned
        if let Some(act) = action.clone() {
            if self.config.log_actions {
                tracing::info!("Symbol {}: Executing action: {:?}", self.symbol, act);
            }

            // Check if this is a position opening action
            let is_position_open = act.is_entry();

            self.state_machine.execute(act.clone())?;
            self.stats.record_action();

            // Emit action executed event
            self.emit_event(RunnerEvent::ActionExecuted {
                runner_id: self.runner_id.clone(),
                action: act,
                timestamp: market_data.timestamp,
            });

            // Emit position opened event if entering position
            if is_position_open {
                if let Some(position) = self.state_machine.position() {
                    self.emit_event(RunnerEvent::PositionOpened {
                        runner_id: self.runner_id.clone(),
                        position: position.clone(),
                        timestamp: market_data.timestamp,
                    });
                }
            }
        }

        // Update state machine (handles auto-exits)
        let position_before = self.state_machine.position().cloned();
        self.state_machine.update(&market_data);
        let state_after = *self.state_machine.current_state();

        // Emit state transition event if state changed
        if state_before != state_after {
            self.emit_event(RunnerEvent::StateTransition {
                runner_id: self.runner_id.clone(),
                from: state_before,
                to: state_after,
                reason: format!("State machine transition"),
                timestamp: market_data.timestamp,
            });
        }

        // Emit position update or closed event
        if let Some(position) = self.state_machine.position() {
            // Position still active - emit update
            if let Some(unrealized_pnl) = position.unrealized_pnl() {
                self.emit_event(RunnerEvent::PositionUpdated {
                    runner_id: self.runner_id.clone(),
                    current_price: market_data.close,
                    unrealized_pnl,
                    timestamp: market_data.timestamp,
                });
            }
        } else if position_before.is_some() {
            // Position was closed
            if let Some(pos) = position_before {
                if let Some(realized_pnl) = pos.realized_pnl() {
                    self.emit_event(RunnerEvent::PositionClosed {
                        runner_id: self.runner_id.clone(),
                        exit_price: pos.current_price(),
                        realized_pnl,
                        reason: "Position closed".to_string(),
                        timestamp: market_data.timestamp,
                    });
                }
            }
        }

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

        let runner = SymbolRunner::new(
            "test_runner".to_string(),
            "BTCUSDT".to_string(),
            strategy,
            rx,
            50
        );

        assert_eq!(runner.runner_id(), "test_runner");
        assert_eq!(runner.symbol(), "BTCUSDT");
        assert_eq!(runner.state(), State::Idle);
    }

    #[tokio::test]
    async fn test_runner_receives_data() {
        let (tx, rx) = mpsc::unbounded_channel();
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        let mut runner = SymbolRunner::new(
            "test_runner".to_string(),
            "BTCUSDT".to_string(),
            strategy,
            rx,
            50
        );

        // Send data
        tx.send(create_test_data(50000.0)).unwrap();

        // Process one tick manually
        let data = runner.data_receiver.recv().await.unwrap();
        runner.process_tick(data).await.unwrap();

        // Check window has data
        assert_eq!(runner.window.len(), 1);
    }

    #[tokio::test]
    async fn test_runner_events() {
        let (data_tx, data_rx) = mpsc::unbounded_channel();
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        let mut runner = SymbolRunner::new(
            "test_runner".to_string(),
            "BTCUSDT".to_string(),
            strategy,
            data_rx,
            50
        )
        .with_event_channel(event_tx);

        // Send tick
        data_tx.send(create_test_data(50000.0)).unwrap();

        // Process tick
        let data = runner.data_receiver.recv().await.unwrap();
        runner.process_tick(data).await.unwrap();

        // Should have received TickReceived event
        let event = event_rx.try_recv().unwrap();
        assert_eq!(event.runner_id(), "test_runner");
        assert!(matches!(event, RunnerEvent::TickReceived { .. }));
    }
}
