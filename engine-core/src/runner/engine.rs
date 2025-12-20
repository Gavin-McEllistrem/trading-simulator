//! Trading Engine - Multi-Runner Orchestration
//!
//! This module provides the `TradingEngine` struct that manages multiple
//! `SymbolRunner` instances, each running in its own Tokio task.
//!
//! # Architecture
//!
//! ```text
//! TradingEngine
//!   ├── Runner 1: BTCUSDT + Strategy A (thread)
//!   ├── Runner 2: BTCUSDT + Strategy B (thread)
//!   ├── Runner 3: ETHUSDT + Strategy A (thread)
//!   └── Runner N: Symbol + Strategy (thread)
//!
//! Market Data Feed → Engine → Broadcast to all runners watching that symbol
//! ```
//!
//! # Key Features
//!
//! - **Multiple strategies per symbol**: Run different strategies on the same symbol
//! - **Strategy comparison**: A/B test strategies side-by-side
//! - **Independent runners**: Each runner has its own state, config, and lifecycle
//! - **Efficient broadcasting**: One data feed → N runners per symbol
//!
//! # Example
//!
//! ```no_run
//! use trading_engine::runner::{TradingEngine, RunnerConfig};
//! use trading_engine::strategy::LuaStrategy;
//! use trading_engine::MarketData;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create engine
//!     let mut engine = TradingEngine::new();
//!
//!     // Load strategies
//!     let ema_strategy = LuaStrategy::new("strategies/ema_crossover.lua")?;
//!     let rsi_strategy = LuaStrategy::new("strategies/rsi_mean_reversion.lua")?;
//!
//!     // Add multiple runners for same symbol with different strategies
//!     engine.add_runner(
//!         "btc_ema".to_string(),
//!         "BTCUSDT".to_string(),
//!         ema_strategy,
//!         50,
//!         RunnerConfig::default()
//!     )?;
//!
//!     engine.add_runner(
//!         "btc_rsi".to_string(),
//!         "BTCUSDT".to_string(),
//!         rsi_strategy,
//!         50,
//!         RunnerConfig::default()
//!     )?;
//!
//!     // Feed market data - broadcasts to both runners
//!     let data = MarketData {
//!         symbol: "BTCUSDT".to_string(),
//!         // ... other fields
//!         # timestamp: 0,
//!         # open: 0.0,
//!         # high: 0.0,
//!         # low: 0.0,
//!         # close: 0.0,
//!         # volume: 0,
//!         # bid: 0.0,
//!         # ask: 0.0,
//!     };
//!     engine.feed_data(data).await?;
//!
//!     // Graceful shutdown
//!     engine.shutdown().await?;
//!     Ok(())
//! }
//! ```

use crate::error::{Result, TradingEngineError};
use crate::events::RunnerEvent;
use crate::market_data::MarketData;
use crate::strategy::LuaStrategy;
use super::{RunnerConfig, RunnerCommand, RunnerSnapshot, SymbolRunner};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Handle to a running symbol runner
struct RunnerHandle {
    /// Unique runner ID
    runner_id: String,

    /// Symbol being traded
    symbol: String,

    /// Channel sender for market data
    tx: mpsc::UnboundedSender<MarketData>,

    /// Channel sender for commands (introspection)
    cmd_tx: mpsc::UnboundedSender<RunnerCommand>,

    /// Task handle for the runner
    task: JoinHandle<Result<()>>,

    /// Timestamp when runner was added
    started_at: std::time::Instant,
}

/// Multi-runner trading engine
///
/// `TradingEngine` orchestrates multiple `SymbolRunner` instances, each
/// running in its own Tokio task. Multiple runners can watch the same symbol
/// with different strategies.
///
/// # Key Concepts
///
/// - **Runner**: A unique instance with ID, symbol, strategy, and config
/// - **Symbol Broadcasting**: Market data for a symbol is sent to ALL runners watching it
/// - **Independent State**: Each runner maintains its own state machine and position
///
/// # Example: Multiple Strategies per Symbol
///
/// ```no_run
/// # use trading_engine::runner::{TradingEngine, RunnerConfig};
/// # use trading_engine::strategy::LuaStrategy;
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let mut engine = TradingEngine::new();
/// let strategy_a = LuaStrategy::new("strategies/ema.lua")?;
/// let strategy_b = LuaStrategy::new("strategies/rsi.lua")?;
///
/// // Both runners receive the same BTCUSDT data
/// engine.add_runner("btc_ema", "BTCUSDT", strategy_a, 50, RunnerConfig::default())?;
/// engine.add_runner("btc_rsi", "BTCUSDT", strategy_b, 50, RunnerConfig::default())?;
/// # Ok(())
/// # }
/// ```
pub struct TradingEngine {
    /// All active runners (runner_id → handle)
    runners: HashMap<String, RunnerHandle>,

    /// Symbol subscription map (symbol → list of runner_ids)
    /// Used for efficient broadcasting
    subscriptions: HashMap<String, Vec<String>>,

    /// Default configuration for new runners
    default_config: RunnerConfig,

    /// Default window size
    default_window_size: usize,

    /// Global event broadcaster
    /// All runner events are aggregated here
    event_tx: mpsc::UnboundedSender<RunnerEvent>,

    /// Event subscribers (shared)
    /// Multiple clients can subscribe to the event stream
    event_subscribers: Arc<Mutex<Vec<mpsc::UnboundedSender<RunnerEvent>>>>,
}

impl TradingEngine {
    /// Create a new trading engine
    ///
    /// # Example
    ///
    /// ```
    /// use trading_engine::runner::TradingEngine;
    ///
    /// let engine = TradingEngine::new();
    /// ```
    pub fn new() -> Self {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<RunnerEvent>();
        let event_subscribers: Arc<Mutex<Vec<mpsc::UnboundedSender<RunnerEvent>>>> =
            Arc::new(Mutex::new(Vec::new()));

        // Spawn event forwarding task
        let subscribers = event_subscribers.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                // Forward to all subscribers
                let mut subs = subscribers.lock().unwrap();
                subs.retain(|tx| tx.send(event.clone()).is_ok());
            }
        });

        Self {
            runners: HashMap::new(),
            subscriptions: HashMap::new(),
            default_config: RunnerConfig::default(),
            default_window_size: 100,
            event_tx,
            event_subscribers,
        }
    }

    /// Create an engine with custom defaults
    ///
    /// # Arguments
    ///
    /// * `config` - Default configuration for all runners
    /// * `window_size` - Default window size for market data
    ///
    /// # Example
    ///
    /// ```
    /// use trading_engine::runner::{TradingEngine, RunnerConfig};
    ///
    /// let config = RunnerConfig::production();
    /// let engine = TradingEngine::with_defaults(config, 200);
    /// ```
    pub fn with_defaults(config: RunnerConfig, window_size: usize) -> Self {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<RunnerEvent>();
        let event_subscribers: Arc<Mutex<Vec<mpsc::UnboundedSender<RunnerEvent>>>> =
            Arc::new(Mutex::new(Vec::new()));

        // Spawn event forwarding task
        let subscribers = event_subscribers.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                // Forward to all subscribers
                let mut subs = subscribers.lock().unwrap();
                subs.retain(|tx| tx.send(event.clone()).is_ok());
            }
        });

        Self {
            runners: HashMap::new(),
            subscriptions: HashMap::new(),
            default_config: config,
            default_window_size: window_size,
            event_tx,
            event_subscribers,
        }
    }

    /// Subscribe to all runner events
    ///
    /// Returns a channel receiver that will receive all events from all runners.
    /// Multiple clients can subscribe simultaneously.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use trading_engine::runner::TradingEngine;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let engine = TradingEngine::new();
    /// let mut events = engine.subscribe_events();
    ///
    /// // Receive events
    /// while let Some(event) = events.recv().await {
    ///     println!("Event: {:?}", event);
    /// }
    /// # }
    /// ```
    pub fn subscribe_events(&self) -> mpsc::UnboundedReceiver<RunnerEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.event_subscribers.lock().unwrap().push(tx);
        rx
    }

    /// Add a runner with default configuration
    ///
    /// # Arguments
    ///
    /// * `runner_id` - Unique identifier for this runner (e.g., "btc_ema_prod")
    /// * `symbol` - Symbol to trade (e.g., "BTCUSDT")
    /// * `strategy` - Lua strategy for trading logic
    ///
    /// # Errors
    ///
    /// Returns error if runner_id already exists.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use trading_engine::runner::TradingEngine;
    /// # use trading_engine::strategy::LuaStrategy;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let mut engine = TradingEngine::new();
    /// let strategy = LuaStrategy::new("strategies/ema_crossover.lua")?;
    ///
    /// engine.add_runner(
    ///     "btc_ema_1".to_string(),
    ///     "BTCUSDT".to_string(),
    ///     strategy
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_runner(
        &mut self,
        runner_id: impl Into<String>,
        symbol: impl Into<String>,
        strategy: LuaStrategy,
    ) -> Result<()> {
        let runner_id = runner_id.into();
        let symbol = symbol.into();

        self.add_runner_with_config(
            runner_id,
            symbol,
            strategy,
            self.default_window_size,
            self.default_config.clone(),
        )
    }

    /// Add a runner with custom configuration
    ///
    /// Creates a new `SymbolRunner` and starts it in a background task.
    /// Multiple runners can watch the same symbol with different strategies.
    ///
    /// # Arguments
    ///
    /// * `runner_id` - Unique identifier for this runner
    /// * `symbol` - Symbol to trade
    /// * `strategy` - Lua strategy
    /// * `window_size` - Size of market data circular buffer
    /// * `config` - Runner configuration
    ///
    /// # Errors
    ///
    /// Returns error if runner_id already exists.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use trading_engine::runner::{TradingEngine, RunnerConfig};
    /// # use trading_engine::strategy::LuaStrategy;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let mut engine = TradingEngine::new();
    /// let strategy = LuaStrategy::new("strategies/ema_crossover.lua")?;
    /// let config = RunnerConfig::production();
    ///
    /// engine.add_runner_with_config(
    ///     "btc_ema_prod",
    ///     "BTCUSDT",
    ///     strategy,
    ///     200,
    ///     config
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_runner_with_config(
        &mut self,
        runner_id: impl Into<String>,
        symbol: impl Into<String>,
        strategy: LuaStrategy,
        window_size: usize,
        config: RunnerConfig,
    ) -> Result<()> {
        let runner_id = runner_id.into();
        let symbol = symbol.into();

        // Check if runner_id already exists
        if self.runners.contains_key(&runner_id) {
            return Err(TradingEngineError::RunnerAlreadyExists(runner_id));
        }

        // Create channel for market data
        let (tx, rx) = mpsc::unbounded_channel();

        // Create channel for commands
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        // Create runner with event channel and command channel
        let mut runner = SymbolRunner::new(
            runner_id.clone(),
            symbol.clone(),
            strategy,
            rx,
            window_size
        )
        .with_config(config)
        .with_event_channel(self.event_tx.clone())
        .with_command_channel(cmd_rx);

        // Emit RunnerStarted event
        let _ = self.event_tx.send(RunnerEvent::RunnerStarted {
            runner_id: runner_id.clone(),
            symbol: symbol.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        });

        // Spawn task
        let task_runner_id = runner_id.clone();
        let task_symbol = symbol.clone();
        let event_tx = self.event_tx.clone();
        let task = tokio::spawn(async move {
            tracing::info!("Starting runner '{}' for {}", task_runner_id, task_symbol);
            let result = runner.run().await;
            if let Err(ref e) = result {
                tracing::error!(
                    "Runner '{}' for {} stopped with error: {}",
                    task_runner_id,
                    task_symbol,
                    e
                );
            } else {
                tracing::info!(
                    "Runner '{}' for {} completed successfully",
                    task_runner_id,
                    task_symbol
                );
            }

            // Emit RunnerStopped event
            let _ = event_tx.send(RunnerEvent::RunnerStopped {
                runner_id: task_runner_id,
                reason: if result.is_ok() {
                    "Normal shutdown".to_string()
                } else {
                    format!("Error: {}", result.as_ref().unwrap_err())
                },
                timestamp: chrono::Utc::now().timestamp_millis(),
            });

            result
        });

        // Store handle
        self.runners.insert(
            runner_id.clone(),
            RunnerHandle {
                runner_id: runner_id.clone(),
                symbol: symbol.clone(),
                tx,
                cmd_tx,
                task,
                started_at: std::time::Instant::now(),
            },
        );

        // Add to subscriptions
        self.subscriptions
            .entry(symbol.clone())
            .or_insert_with(Vec::new)
            .push(runner_id.clone());

        tracing::info!(
            "Added runner '{}' for symbol {} (total runners for {}: {})",
            runner_id,
            symbol,
            symbol,
            self.subscriptions.get(&symbol).map(|v| v.len()).unwrap_or(0)
        );

        Ok(())
    }

    /// Remove a runner from the engine
    ///
    /// Closes the market data channel and waits for the runner to shut down.
    ///
    /// # Arguments
    ///
    /// * `runner_id` - ID of the runner to remove
    ///
    /// # Errors
    ///
    /// Returns error if runner doesn't exist or runner task panicked.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use trading_engine::runner::TradingEngine;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let mut engine = TradingEngine::new();
    /// engine.remove_runner("btc_ema_1").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_runner(&mut self, runner_id: &str) -> Result<()> {
        let handle = self.runners.remove(runner_id).ok_or_else(|| {
            TradingEngineError::RunnerNotFound(runner_id.to_string())
        })?;

        tracing::info!("Removing runner '{}'", runner_id);

        // Remove from subscriptions
        if let Some(subs) = self.subscriptions.get_mut(&handle.symbol) {
            subs.retain(|id| id != runner_id);
            if subs.is_empty() {
                self.subscriptions.remove(&handle.symbol);
            }
        }

        // Drop the sender to close the channel
        drop(handle.tx);

        // Wait for the task to complete
        match handle.task.await {
            Ok(Ok(())) => {
                tracing::info!("Runner '{}' removed successfully", runner_id);
                Ok(())
            }
            Ok(Err(e)) => {
                tracing::error!("Runner '{}' returned error: {}", runner_id, e);
                Err(e)
            }
            Err(e) => {
                tracing::error!("Runner '{}' task panicked: {}", runner_id, e);
                Err(TradingEngineError::TaskPanic(runner_id.to_string()))
            }
        }
    }

    /// Feed market data to all runners watching this symbol
    ///
    /// Broadcasts the data to ALL runners subscribed to the symbol.
    /// This allows multiple strategies to process the same data in parallel.
    ///
    /// # Arguments
    ///
    /// * `data` - Market data to distribute
    ///
    /// # Errors
    ///
    /// Returns error if no runners are watching this symbol or if any channel is closed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use trading_engine::runner::TradingEngine;
    /// # use trading_engine::MarketData;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let mut engine = TradingEngine::new();
    /// let data = MarketData {
    ///     symbol: "BTCUSDT".to_string(),
    ///     timestamp: 1234567890,
    ///     open: 50000.0,
    ///     high: 50100.0,
    ///     low: 49900.0,
    ///     close: 50050.0,
    ///     volume: 1000,
    ///     bid: 50045.0,
    ///     ask: 50055.0,
    /// };
    ///
    /// // Broadcasts to all runners watching BTCUSDT
    /// engine.feed_data(data).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn feed_data(&self, data: MarketData) -> Result<()> {
        let runner_ids = self.subscriptions.get(&data.symbol).ok_or_else(|| {
            TradingEngineError::NoRunnersForSymbol(data.symbol.clone())
        })?;

        // Broadcast to all runners watching this symbol
        for runner_id in runner_ids {
            if let Some(handle) = self.runners.get(runner_id) {
                // Clone data for each runner
                handle.tx.send(data.clone()).map_err(|_| {
                    TradingEngineError::ChannelClosed(runner_id.clone())
                })?;
            }
        }

        Ok(())
    }

    /// Feed market data to multiple symbols
    ///
    /// Distributes data to all matching runners in parallel.
    ///
    /// # Arguments
    ///
    /// * `data_batch` - Vector of market data for different symbols
    ///
    /// # Errors
    ///
    /// Returns error if no runners exist for any symbol or if channels are closed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use trading_engine::runner::TradingEngine;
    /// # use trading_engine::MarketData;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let mut engine = TradingEngine::new();
    /// let batch = vec![
    ///     MarketData { symbol: "BTCUSDT".to_string(), /* ... */ # timestamp: 0, open: 0.0, high: 0.0, low: 0.0, close: 0.0, volume: 0, bid: 0.0, ask: 0.0 },
    ///     MarketData { symbol: "ETHUSDT".to_string(), /* ... */ # timestamp: 0, open: 0.0, high: 0.0, low: 0.0, close: 0.0, volume: 0, bid: 0.0, ask: 0.0 },
    /// ];
    /// engine.feed_batch(batch).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn feed_batch(&self, data_batch: Vec<MarketData>) -> Result<()> {
        for data in data_batch {
            self.feed_data(data).await?;
        }
        Ok(())
    }

    /// Get list of all runner IDs
    ///
    /// # Example
    ///
    /// ```
    /// # use trading_engine::runner::TradingEngine;
    /// let engine = TradingEngine::new();
    /// let runner_ids = engine.runner_ids();
    /// ```
    pub fn runner_ids(&self) -> Vec<String> {
        self.runners.keys().cloned().collect()
    }

    /// Get list of symbols being watched
    ///
    /// Returns unique symbols that have at least one runner.
    ///
    /// # Example
    ///
    /// ```
    /// # use trading_engine::runner::TradingEngine;
    /// let engine = TradingEngine::new();
    /// let symbols = engine.active_symbols();
    /// ```
    pub fn active_symbols(&self) -> Vec<String> {
        self.subscriptions.keys().cloned().collect()
    }

    /// Get runner count for a specific symbol
    ///
    /// # Example
    ///
    /// ```
    /// # use trading_engine::runner::TradingEngine;
    /// let engine = TradingEngine::new();
    /// let count = engine.runner_count_for_symbol("BTCUSDT");
    /// ```
    pub fn runner_count_for_symbol(&self, symbol: &str) -> usize {
        self.subscriptions
            .get(symbol)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Get runner IDs watching a specific symbol
    ///
    /// # Example
    ///
    /// ```
    /// # use trading_engine::runner::TradingEngine;
    /// let engine = TradingEngine::new();
    /// let runners = engine.runners_for_symbol("BTCUSDT");
    /// ```
    pub fn runners_for_symbol(&self, symbol: &str) -> Vec<String> {
        self.subscriptions
            .get(symbol)
            .cloned()
            .unwrap_or_default()
    }

    /// Get total number of active runners
    ///
    /// # Example
    ///
    /// ```
    /// # use trading_engine::runner::TradingEngine;
    /// let engine = TradingEngine::new();
    /// assert_eq!(engine.runner_count(), 0);
    /// ```
    pub fn runner_count(&self) -> usize {
        self.runners.len()
    }

    /// Check if a runner exists
    ///
    /// # Example
    ///
    /// ```
    /// # use trading_engine::runner::TradingEngine;
    /// let engine = TradingEngine::new();
    /// assert!(!engine.has_runner("btc_ema_1"));
    /// ```
    pub fn has_runner(&self, runner_id: &str) -> bool {
        self.runners.contains_key(runner_id)
    }

    /// Get the symbol for a specific runner
    ///
    /// # Example
    ///
    /// ```
    /// # use trading_engine::runner::TradingEngine;
    /// let engine = TradingEngine::new();
    /// let symbol = engine.runner_symbol("btc_ema_1");
    /// ```
    pub fn runner_symbol(&self, runner_id: &str) -> Option<String> {
        self.runners.get(runner_id).map(|h| h.symbol.clone())
    }

    /// Get runner uptime
    ///
    /// Returns the duration since the runner was started.
    ///
    /// # Example
    ///
    /// ```
    /// # use trading_engine::runner::TradingEngine;
    /// let engine = TradingEngine::new();
    /// if let Some(uptime) = engine.runner_uptime("btc_ema_1") {
    ///     println!("Runner has been running for {:?}", uptime);
    /// }
    /// ```
    pub fn runner_uptime(&self, runner_id: &str) -> Option<std::time::Duration> {
        self.runners.get(runner_id).map(|h| h.started_at.elapsed())
    }

    /// Check if a runner task has completed or panicked
    ///
    /// Returns `Some(true)` if the runner is still healthy (task is running),
    /// `Some(false)` if the runner has completed or panicked,
    /// `None` if the runner doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// # use trading_engine::runner::TradingEngine;
    /// let engine = TradingEngine::new();
    /// if let Some(is_healthy) = engine.runner_is_healthy("btc_ema_1") {
    ///     if !is_healthy {
    ///         println!("Runner has stopped!");
    ///     }
    /// }
    /// ```
    pub fn runner_is_healthy(&self, runner_id: &str) -> Option<bool> {
        self.runners.get(runner_id).map(|h| !h.task.is_finished())
    }

    /// Get health status for all runners
    ///
    /// Returns a map of runner_id → is_healthy for all runners.
    ///
    /// # Example
    ///
    /// ```
    /// # use trading_engine::runner::TradingEngine;
    /// let engine = TradingEngine::new();
    /// let health = engine.health_check();
    /// for (runner_id, is_healthy) in health {
    ///     if !is_healthy {
    ///         println!("Warning: {} is not healthy", runner_id);
    ///     }
    /// }
    /// ```
    pub fn health_check(&self) -> HashMap<String, bool> {
        self.runners
            .iter()
            .map(|(id, handle)| (id.clone(), !handle.task.is_finished()))
            .collect()
    }

    /// Get list of unhealthy runners
    ///
    /// Returns runner IDs for all runners whose tasks have completed or panicked.
    ///
    /// # Example
    ///
    /// ```
    /// # use trading_engine::runner::TradingEngine;
    /// let engine = TradingEngine::new();
    /// let unhealthy = engine.unhealthy_runners();
    /// if !unhealthy.is_empty() {
    ///     println!("Unhealthy runners: {:?}", unhealthy);
    /// }
    /// ```
    pub fn unhealthy_runners(&self) -> Vec<String> {
        self.runners
            .iter()
            .filter(|(_, handle)| handle.task.is_finished())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get engine summary statistics
    ///
    /// Returns a summary of engine state including runner count, symbol count,
    /// and health status.
    ///
    /// # Example
    ///
    /// ```
    /// # use trading_engine::runner::TradingEngine;
    /// let engine = TradingEngine::new();
    /// let summary = engine.summary();
    /// println!("{}", summary);
    /// ```
    pub fn summary(&self) -> String {
        let total_runners = self.runner_count();
        let total_symbols = self.active_symbols().len();
        let unhealthy = self.unhealthy_runners();
        let healthy_count = total_runners - unhealthy.len();

        format!(
            "TradingEngine Summary:\n\
             - Total Runners: {}\n\
             - Healthy: {}\n\
             - Unhealthy: {}\n\
             - Symbols: {}\n\
             - Runners per symbol: {:.1}",
            total_runners,
            healthy_count,
            unhealthy.len(),
            total_symbols,
            if total_symbols > 0 {
                total_runners as f64 / total_symbols as f64
            } else {
                0.0
            }
        )
    }

    /// Shutdown all runners gracefully
    ///
    /// Closes all channels and waits for all tasks to complete.
    ///
    /// # Errors
    ///
    /// Returns error if any runner task panicked.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use trading_engine::runner::TradingEngine;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let mut engine = TradingEngine::new();
    /// // ... add runners and feed data ...
    /// engine.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(mut self) -> Result<()> {
        tracing::info!("Shutting down engine with {} runners", self.runners.len());

        let runner_ids: Vec<String> = self.runners.keys().cloned().collect();

        for runner_id in runner_ids {
            if let Err(e) = self.remove_runner(&runner_id).await {
                tracing::error!("Error shutting down runner '{}': {}", runner_id, e);
            }
        }

        tracing::info!("Engine shutdown complete");
        Ok(())
    }

    /// Shutdown and collect results from all runners
    ///
    /// Like `shutdown()` but returns a map of runner_id → result.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use trading_engine::runner::TradingEngine;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let mut engine = TradingEngine::new();
    /// // ... add runners and feed data ...
    /// let results = engine.shutdown_with_results().await;
    /// for (runner_id, result) in results {
    ///     println!("{}: {:?}", runner_id, result);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown_with_results(mut self) -> HashMap<String, Result<()>> {
        let mut results = HashMap::new();

        let runner_ids: Vec<String> = self.runners.keys().cloned().collect();

        for runner_id in runner_ids {
            let result = self.remove_runner(&runner_id).await;
            results.insert(runner_id, result);
        }

        results
    }

    /// Get a snapshot of a runner's current state
    ///
    /// This method queries the runner for its current state, including:
    /// - Current FSM state
    /// - Position information (if any)
    /// - Strategy context data
    /// - Statistics
    ///
    /// # Arguments
    ///
    /// * `runner_id` - The unique ID of the runner to query
    ///
    /// # Returns
    ///
    /// Returns `Some(RunnerSnapshot)` if the runner exists and is running,
    /// or `None` if the runner doesn't exist or the command channel is closed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use trading_engine::runner::TradingEngine;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let mut engine = TradingEngine::new();
    /// // ... add runners ...
    ///
    /// if let Some(snapshot) = engine.get_runner_snapshot("btc_ema").await {
    ///     println!("Runner state: {}", snapshot.state_str());
    ///     println!("Ticks processed: {}", snapshot.stats.ticks_processed);
    ///     if let Some(pos) = &snapshot.position {
    ///         println!("Position: {} at ${}", pos.side(), pos.entry_price());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_runner_snapshot(&self, runner_id: &str) -> Option<RunnerSnapshot> {
        // Get the runner handle
        let handle = self.runners.get(runner_id)?;

        // Create oneshot channel for response
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        // Send GetSnapshot command
        let cmd = RunnerCommand::GetSnapshot { response: response_tx };
        handle.cmd_tx.send(cmd).ok()?;

        // Wait for response (with timeout to avoid hanging)
        tokio::time::timeout(std::time::Duration::from_millis(100), response_rx)
            .await
            .ok()?
            .ok()
    }

    /// Get recent price history from a runner's data window
    ///
    /// # Arguments
    ///
    /// * `runner_id` - The unique ID of the runner to query
    /// * `count` - Optional number of recent data points to retrieve (all if None)
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<MarketData>)` if the runner exists,
    /// or `None` if the runner doesn't exist or the command channel is closed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use trading_engine::runner::TradingEngine;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let mut engine = TradingEngine::new();
    /// // ... add runners and feed data ...
    ///
    /// // Get last 10 data points
    /// if let Some(history) = engine.get_price_history("btc_ema", Some(10)).await {
    ///     for data in history {
    ///         println!("{}: ${}", data.symbol, data.close);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_price_history(
        &self,
        runner_id: &str,
        count: Option<usize>,
    ) -> Option<Vec<MarketData>> {
        // Get the runner handle
        let handle = self.runners.get(runner_id)?;

        // Create oneshot channel for response
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        // Send GetPriceHistory command
        let cmd = RunnerCommand::GetPriceHistory {
            count,
            response: response_tx,
        };
        handle.cmd_tx.send(cmd).ok()?;

        // Wait for response (with timeout)
        tokio::time::timeout(std::time::Duration::from_millis(100), response_rx)
            .await
            .ok()?
            .ok()
    }
}

impl Default for TradingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::LuaStrategy;

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = TradingEngine::new();
        assert_eq!(engine.runner_count(), 0);
        assert!(engine.active_symbols().is_empty());
        assert!(engine.runner_ids().is_empty());
    }

    #[tokio::test]
    async fn test_add_runner() {
        let mut engine = TradingEngine::new();
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema_1", "BTCUSDT", strategy).unwrap();

        assert_eq!(engine.runner_count(), 1);
        assert!(engine.has_runner("btc_ema_1"));
        assert_eq!(engine.runner_symbol("btc_ema_1"), Some("BTCUSDT".to_string()));
        assert_eq!(engine.runner_count_for_symbol("BTCUSDT"), 1);
    }

    #[tokio::test]
    async fn test_multiple_runners_same_symbol() {
        let mut engine = TradingEngine::new();
        let strategy1 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");
        let strategy2 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema", "BTCUSDT", strategy1).unwrap();
        engine.add_runner("btc_rsi", "BTCUSDT", strategy2).unwrap();

        assert_eq!(engine.runner_count(), 2);
        assert_eq!(engine.runner_count_for_symbol("BTCUSDT"), 2);
        assert_eq!(engine.active_symbols().len(), 1);

        let runners = engine.runners_for_symbol("BTCUSDT");
        assert_eq!(runners.len(), 2);
        assert!(runners.contains(&"btc_ema".to_string()));
        assert!(runners.contains(&"btc_rsi".to_string()));
    }

    #[tokio::test]
    async fn test_add_duplicate_runner_id() {
        let mut engine = TradingEngine::new();
        let strategy1 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");
        let strategy2 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema_1", "BTCUSDT", strategy1).unwrap();
        let result = engine.add_runner("btc_ema_1", "ETHUSDT", strategy2);

        assert!(result.is_err());
        assert_eq!(engine.runner_count(), 1);
    }

    #[tokio::test]
    async fn test_remove_runner() {
        let mut engine = TradingEngine::new();
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema_1", "BTCUSDT", strategy).unwrap();
        assert_eq!(engine.runner_count(), 1);

        engine.remove_runner("btc_ema_1").await.unwrap();
        assert_eq!(engine.runner_count(), 0);
        assert!(!engine.has_runner("btc_ema_1"));
        assert_eq!(engine.runner_count_for_symbol("BTCUSDT"), 0);
    }

    #[tokio::test]
    async fn test_remove_one_runner_keeps_others() {
        let mut engine = TradingEngine::new();
        let strategy1 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");
        let strategy2 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema", "BTCUSDT", strategy1).unwrap();
        engine.add_runner("btc_rsi", "BTCUSDT", strategy2).unwrap();

        engine.remove_runner("btc_ema").await.unwrap();

        assert_eq!(engine.runner_count(), 1);
        assert_eq!(engine.runner_count_for_symbol("BTCUSDT"), 1);
        assert!(engine.has_runner("btc_rsi"));
        assert!(!engine.has_runner("btc_ema"));
    }

    #[tokio::test]
    async fn test_remove_nonexistent_runner() {
        let mut engine = TradingEngine::new();
        let result = engine.remove_runner("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_feed_data() {
        let mut engine = TradingEngine::new();
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema_1", "BTCUSDT", strategy).unwrap();

        let data = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1234567890,
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 1000,
            bid: 50045.0,
            ask: 50055.0,
        };

        engine.feed_data(data).await.unwrap();
    }

    #[tokio::test]
    async fn test_feed_data_broadcasts_to_multiple_runners() {
        let mut engine = TradingEngine::new();
        let strategy1 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");
        let strategy2 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema", "BTCUSDT", strategy1).unwrap();
        engine.add_runner("btc_rsi", "BTCUSDT", strategy2).unwrap();

        let data = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1234567890,
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 1000,
            bid: 50045.0,
            ask: 50055.0,
        };

        // Should broadcast to both runners without error
        engine.feed_data(data).await.unwrap();
    }

    #[tokio::test]
    async fn test_feed_data_unknown_symbol() {
        let engine = TradingEngine::new();

        let data = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1234567890,
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 1000,
            bid: 50045.0,
            ask: 50055.0,
        };

        let result = engine.feed_data(data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_symbols_and_runners() {
        let mut engine = TradingEngine::new();
        let strategy1 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");
        let strategy2 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");
        let strategy3 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");
        let strategy4 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema", "BTCUSDT", strategy1).unwrap();
        engine.add_runner("btc_rsi", "BTCUSDT", strategy2).unwrap();
        engine.add_runner("eth_ema", "ETHUSDT", strategy3).unwrap();
        engine.add_runner("sol_ema", "SOLUSDT", strategy4).unwrap();

        assert_eq!(engine.runner_count(), 4);
        assert_eq!(engine.active_symbols().len(), 3);
        assert_eq!(engine.runner_count_for_symbol("BTCUSDT"), 2);
        assert_eq!(engine.runner_count_for_symbol("ETHUSDT"), 1);
        assert_eq!(engine.runner_count_for_symbol("SOLUSDT"), 1);
    }

    #[tokio::test]
    async fn test_shutdown() {
        let mut engine = TradingEngine::new();
        let strategy1 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");
        let strategy2 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema", "BTCUSDT", strategy1).unwrap();
        engine.add_runner("eth_ema", "ETHUSDT", strategy2).unwrap();

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_runner_uptime() {
        let mut engine = TradingEngine::new();
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema", "BTCUSDT", strategy).unwrap();

        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let uptime = engine.runner_uptime("btc_ema").unwrap();
        assert!(uptime.as_millis() >= 10);
    }

    #[tokio::test]
    async fn test_runner_health() {
        let mut engine = TradingEngine::new();
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema", "BTCUSDT", strategy).unwrap();

        // Runner should be healthy initially
        assert_eq!(engine.runner_is_healthy("btc_ema"), Some(true));

        // Remove runner
        engine.remove_runner("btc_ema").await.unwrap();

        // Runner should no longer exist
        assert_eq!(engine.runner_is_healthy("btc_ema"), None);
    }

    #[tokio::test]
    async fn test_health_check() {
        let mut engine = TradingEngine::new();
        let strategy1 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");
        let strategy2 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema", "BTCUSDT", strategy1).unwrap();
        engine.add_runner("eth_ema", "ETHUSDT", strategy2).unwrap();

        let health = engine.health_check();
        assert_eq!(health.len(), 2);
        assert_eq!(health.get("btc_ema"), Some(&true));
        assert_eq!(health.get("eth_ema"), Some(&true));
    }

    #[tokio::test]
    async fn test_unhealthy_runners() {
        let mut engine = TradingEngine::new();
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema", "BTCUSDT", strategy).unwrap();

        // Should have no unhealthy runners initially
        assert!(engine.unhealthy_runners().is_empty());
    }

    #[tokio::test]
    async fn test_summary() {
        let mut engine = TradingEngine::new();
        let strategy1 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");
        let strategy2 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        engine.add_runner("btc_ema", "BTCUSDT", strategy1).unwrap();
        engine.add_runner("btc_rsi", "BTCUSDT", strategy2).unwrap();

        let summary = engine.summary();
        assert!(summary.contains("Total Runners: 2"));
        assert!(summary.contains("Symbols: 1"));
        assert!(summary.contains("Runners per symbol: 2.0"));
    }

    #[tokio::test]
    async fn test_event_aggregation() {
        let mut engine = TradingEngine::new();
        let mut events = engine.subscribe_events();

        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        // Add runner - should emit RunnerStarted event
        engine.add_runner("btc_ema", "BTCUSDT", strategy).unwrap();

        // Wait for event
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Should receive RunnerStarted event
        let event = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            events.recv()
        ).await;

        assert!(event.is_ok());
        let event = event.unwrap().unwrap();
        assert_eq!(event.runner_id(), "btc_ema");
        assert!(matches!(event, crate::events::RunnerEvent::RunnerStarted { .. }));
    }

    #[tokio::test]
    async fn test_multiple_event_subscribers() {
        let mut engine = TradingEngine::new();
        let mut subscriber1 = engine.subscribe_events();
        let mut subscriber2 = engine.subscribe_events();

        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        // Add runner
        engine.add_runner("btc_ema", "BTCUSDT", strategy).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Both subscribers should receive the same event
        let event1 = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            subscriber1.recv()
        ).await.unwrap().unwrap();

        let event2 = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            subscriber2.recv()
        ).await.unwrap().unwrap();

        assert_eq!(event1.runner_id(), event2.runner_id());
    }

    #[tokio::test]
    async fn test_get_runner_snapshot() {
        let mut engine = TradingEngine::new();
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        // Add runner
        engine.add_runner("btc_ema", "BTCUSDT", strategy).unwrap();

        // Give runner time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Feed some data
        let data = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1234567890,
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 1000,
            bid: 50045.0,
            ask: 50055.0,
        };
        engine.feed_data(data).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Get snapshot
        let snapshot = engine.get_runner_snapshot("btc_ema").await;
        assert!(snapshot.is_some());

        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.runner_id, "btc_ema");
        assert_eq!(snapshot.symbol, "BTCUSDT");
        assert!(snapshot.stats.ticks_processed >= 1);
    }

    #[tokio::test]
    async fn test_get_price_history() {
        let mut engine = TradingEngine::new();
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load test strategy");

        // Add runner
        engine.add_runner("btc_ema", "BTCUSDT", strategy).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Feed multiple data points
        for i in 0..5 {
            let data = MarketData {
                symbol: "BTCUSDT".to_string(),
                timestamp: 1234567890 + i,
                open: 50000.0 + (i as f64) * 10.0,
                high: 50100.0 + (i as f64) * 10.0,
                low: 49900.0 + (i as f64) * 10.0,
                close: 50050.0 + (i as f64) * 10.0,
                volume: 1000,
                bid: 50045.0 + (i as f64) * 10.0,
                ask: 50055.0 + (i as f64) * 10.0,
            };
            engine.feed_data(data).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }

        // Get all history
        let history = engine.get_price_history("btc_ema", None).await;
        assert!(history.is_some());
        let history = history.unwrap();
        assert_eq!(history.len(), 5);

        // Get last 3 data points
        let history = engine.get_price_history("btc_ema", Some(3)).await;
        assert!(history.is_some());
        let history = history.unwrap();
        assert_eq!(history.len(), 3);

        // Verify last element has highest price
        let last = history.last().unwrap();
        assert_eq!(last.close, 50050.0 + 40.0); // 4th data point (0-indexed)
    }

    #[tokio::test]
    async fn test_snapshot_nonexistent_runner() {
        let engine = TradingEngine::new();

        // Try to get snapshot of non-existent runner
        let snapshot = engine.get_runner_snapshot("nonexistent").await;
        assert!(snapshot.is_none());

        // Try to get history of non-existent runner
        let history = engine.get_price_history("nonexistent", None).await;
        assert!(history.is_none());
    }
}
