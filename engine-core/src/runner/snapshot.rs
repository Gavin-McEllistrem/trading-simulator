//! Runner state snapshot and command types for state introspection.
//!
//! This module provides types for querying runner state on-demand via a command channel.
//! Complements the event system (push) with pull-based state queries.

use crate::market_data::MarketData;
use crate::state_machine::{Position, State};
use crate::runner::RunnerStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::oneshot;

/// Runner execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerStatus {
    /// Runner is actively processing ticks
    Running,
    /// Runner is paused (not processing ticks, preserving state)
    Paused,
    /// Runner has been stopped and cannot be resumed
    Stopped,
}

impl RunnerStatus {
    /// Check if runner can process ticks
    pub fn is_active(&self) -> bool {
        matches!(self, RunnerStatus::Running)
    }

    /// Check if runner is paused
    pub fn is_paused(&self) -> bool {
        matches!(self, RunnerStatus::Paused)
    }

    /// Check if runner is stopped
    pub fn is_stopped(&self) -> bool {
        matches!(self, RunnerStatus::Stopped)
    }
}

impl Default for RunnerStatus {
    fn default() -> Self {
        RunnerStatus::Running
    }
}

/// Commands that can be sent to a running SymbolRunner.
#[derive(Debug)]
pub enum RunnerCommand {
    /// Request a snapshot of the runner's current state.
    GetSnapshot {
        /// Channel to send the snapshot response.
        response: oneshot::Sender<RunnerSnapshot>,
    },

    /// Request recent price history from the runner's data window.
    GetPriceHistory {
        /// Number of recent data points to retrieve (or all if None).
        count: Option<usize>,
        /// Channel to send the price history response.
        response: oneshot::Sender<Vec<MarketData>>,
    },

    /// Pause the runner (stop processing ticks, preserve state).
    Pause {
        /// Channel to send confirmation response.
        response: oneshot::Sender<bool>,
    },

    /// Resume the runner from paused state.
    Resume {
        /// Channel to send confirmation response.
        response: oneshot::Sender<bool>,
    },

    /// Stop the runner completely (cannot be resumed).
    Stop {
        /// Channel to send confirmation response.
        response: oneshot::Sender<bool>,
    },
}

/// A point-in-time snapshot of a runner's complete state.
///
/// This type captures all relevant information about a runner at a specific moment,
/// suitable for dashboard display or debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerSnapshot {
    /// Unique identifier for this runner.
    pub runner_id: String,

    /// Symbol being traded.
    pub symbol: String,

    /// Runner execution status (Running/Paused/Stopped).
    pub status: RunnerStatus,

    /// Current state machine state.
    pub current_state: State,

    /// Current position (if in a trade).
    pub position: Option<Position>,

    /// Strategy context data.
    ///
    /// Contains all context variables as JSON-compatible values.
    /// Organized by type for easier serialization.
    pub context: ContextSnapshot,

    /// Runner statistics and performance metrics.
    pub stats: RunnerStats,

    /// How long the runner has been running.
    pub uptime_secs: u64,

    /// Timestamp when this snapshot was taken (milliseconds since Unix epoch).
    pub snapshot_timestamp: i64,
}

/// Snapshot of the strategy context.
///
/// Mirrors the Context struct but uses JSON-compatible types for serialization.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextSnapshot {
    /// String values from context.
    pub strings: HashMap<String, String>,

    /// Numeric values from context.
    pub numbers: HashMap<String, f64>,

    /// Integer values from context.
    pub integers: HashMap<String, i64>,

    /// Boolean values from context.
    pub booleans: HashMap<String, bool>,
}

impl RunnerSnapshot {
    /// Create a new snapshot with the given fields.
    pub fn new(
        runner_id: String,
        symbol: String,
        status: RunnerStatus,
        current_state: State,
        position: Option<Position>,
        context: ContextSnapshot,
        stats: RunnerStats,
        uptime: Duration,
    ) -> Self {
        Self {
            runner_id,
            symbol,
            status,
            current_state,
            position,
            context,
            stats,
            uptime_secs: uptime.as_secs(),
            snapshot_timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// Check if the runner is currently in a position.
    pub fn has_position(&self) -> bool {
        self.position.is_some()
    }

    /// Get the current state as a string.
    pub fn state_str(&self) -> &str {
        match self.current_state {
            State::Idle => "Idle",
            State::Analyzing => "Analyzing",
            State::InPosition => "InPosition",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::Side;

    #[test]
    fn test_runner_snapshot_creation() {
        let mut context = ContextSnapshot::default();
        context.strings.insert("signal".to_string(), "bullish".to_string());
        context.numbers.insert("confidence".to_string(), 0.8);

        let stats = RunnerStats::new();

        let snapshot = RunnerSnapshot::new(
            "test_runner".to_string(),
            "BTCUSDT".to_string(),
            RunnerStatus::Running,
            State::Idle,
            None,
            context,
            stats,
            Duration::from_secs(120),
        );

        assert_eq!(snapshot.runner_id, "test_runner");
        assert_eq!(snapshot.symbol, "BTCUSDT");
        assert_eq!(snapshot.status, RunnerStatus::Running);
        assert_eq!(snapshot.state_str(), "Idle");
        assert!(!snapshot.has_position());
        assert_eq!(snapshot.uptime_secs, 120);
        assert_eq!(snapshot.context.strings.get("signal").unwrap(), "bullish");
    }

    #[test]
    fn test_snapshot_with_position() {
        let position = Position::new(50000.0, 0.1, Side::Long, 1234567890);
        let snapshot = RunnerSnapshot::new(
            "btc_runner".to_string(),
            "BTCUSDT".to_string(),
            RunnerStatus::Running,
            State::InPosition,
            Some(position),
            ContextSnapshot::default(),
            RunnerStats::new(),
            Duration::from_secs(60),
        );

        assert!(snapshot.has_position());
        assert_eq!(snapshot.state_str(), "InPosition");
        assert_eq!(snapshot.position.as_ref().unwrap().entry_price(), 50000.0);
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = RunnerSnapshot::new(
            "test_runner".to_string(),
            "ETHUSDT".to_string(),
            RunnerStatus::Running,
            State::Analyzing,
            None,
            ContextSnapshot::default(),
            RunnerStats::new(),
            Duration::from_secs(30),
        );

        // Test JSON serialization
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("test_runner"));
        assert!(json.contains("ETHUSDT"));

        // Test deserialization
        let deserialized: RunnerSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.runner_id, "test_runner");
        assert_eq!(deserialized.symbol, "ETHUSDT");
        assert_eq!(deserialized.status, RunnerStatus::Running);
    }

    #[test]
    fn test_context_snapshot() {
        let mut context = ContextSnapshot::default();
        context.strings.insert("strategy".to_string(), "ema_cross".to_string());
        context.numbers.insert("fast_ema".to_string(), 50100.0);
        context.numbers.insert("slow_ema".to_string(), 49900.0);
        context.integers.insert("bars_analyzed".to_string(), 42);
        context.booleans.insert("signal_active".to_string(), true);

        assert_eq!(context.strings.len(), 1);
        assert_eq!(context.numbers.len(), 2);
        assert_eq!(context.integers.len(), 1);
        assert_eq!(context.booleans.len(), 1);

        // Test serialization
        let json = serde_json::to_string(&context).unwrap();
        let deserialized: ContextSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.booleans.get("signal_active"), Some(&true));
    }
}
