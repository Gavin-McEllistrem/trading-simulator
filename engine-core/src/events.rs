//! Event system for real-time runner updates
//!
//! This module provides event types that runners emit during their lifecycle.
//! Events are broadcast to subscribers (e.g., WebSocket clients) for real-time
//! monitoring and visualization.
//!
//! # Event Flow
//!
//! ```text
//! SymbolRunner → emit_event() → TradingEngine event stream → WebSocket → Browser
//! ```
//!
//! # Example
//!
//! ```no_run
//! use trading_engine::events::{RunnerEvent, ErrorSeverity};
//! use tokio::sync::mpsc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (tx, mut rx) = mpsc::unbounded_channel();
//!
//!     // Runner emits events
//!     tx.send(RunnerEvent::RunnerStarted {
//!         runner_id: "btc_ema".to_string(),
//!         symbol: "BTCUSDT".to_string(),
//!         timestamp: 1234567890,
//!     }).unwrap();
//!
//!     // Subscriber receives events
//!     while let Some(event) = rx.recv().await {
//!         println!("Event: {:?}", event);
//!     }
//! }
//! ```

use crate::market_data::MarketData;
use crate::state_machine::{Action, Position, State};
use serde::{Deserialize, Serialize};

/// Events emitted by runners during their lifecycle
///
/// Each event contains enough information to update a live dashboard
/// without requiring additional API calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RunnerEvent {
    /// Runner was started
    ///
    /// Emitted once when a runner is added to the engine.
    RunnerStarted {
        runner_id: String,
        symbol: String,
        timestamp: i64,
    },

    /// Runner was stopped
    ///
    /// Emitted when a runner is removed from the engine or crashes.
    RunnerStopped {
        runner_id: String,
        reason: String,
        timestamp: i64,
    },

    /// Market data tick received
    ///
    /// Emitted on every tick. Can be used to update live charts.
    /// High frequency - clients may want to throttle/sample.
    TickReceived {
        runner_id: String,
        symbol: String,
        data: MarketData,
    },

    /// State machine transition
    ///
    /// Emitted when the FSM changes state (Idle → Analyzing → InPosition).
    StateTransition {
        runner_id: String,
        from: State,
        to: State,
        reason: String,
        timestamp: i64,
    },

    /// Trading action executed
    ///
    /// Emitted when the state machine executes an action returned by the strategy.
    ActionExecuted {
        runner_id: String,
        action: Action,
        timestamp: i64,
    },

    /// Position opened
    ///
    /// Emitted when entering a long or short position.
    PositionOpened {
        runner_id: String,
        position: Position,
        timestamp: i64,
    },

    /// Position updated
    ///
    /// Emitted on every tick while in a position.
    /// Contains current unrealized P&L and price.
    PositionUpdated {
        runner_id: String,
        current_price: f64,
        unrealized_pnl: f64,
        timestamp: i64,
    },

    /// Position closed
    ///
    /// Emitted when exiting a position (manual exit or stop loss/take profit hit).
    PositionClosed {
        runner_id: String,
        exit_price: f64,
        realized_pnl: f64,
        reason: String,
        timestamp: i64,
    },

    /// Error occurred
    ///
    /// Emitted when a runner encounters an error.
    /// Severity indicates if the runner can continue or must stop.
    Error {
        runner_id: String,
        error: String,
        severity: ErrorSeverity,
        timestamp: i64,
    },

    /// Statistics update
    ///
    /// Emitted periodically with runner performance metrics.
    /// Lower frequency than tick updates.
    StatsUpdate {
        runner_id: String,
        ticks_processed: u64,
        actions_executed: u64,
        error_rate: f64,
        avg_tick_duration_ms: f64,
        timestamp: i64,
    },
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Warning - runner can continue
    Warning,

    /// Error - runner can continue if stop_on_error=false
    Error,

    /// Critical - runner must stop
    Critical,
}

impl RunnerEvent {
    /// Get the runner_id for this event
    pub fn runner_id(&self) -> &str {
        match self {
            RunnerEvent::RunnerStarted { runner_id, .. } => runner_id,
            RunnerEvent::RunnerStopped { runner_id, .. } => runner_id,
            RunnerEvent::TickReceived { runner_id, .. } => runner_id,
            RunnerEvent::StateTransition { runner_id, .. } => runner_id,
            RunnerEvent::ActionExecuted { runner_id, .. } => runner_id,
            RunnerEvent::PositionOpened { runner_id, .. } => runner_id,
            RunnerEvent::PositionUpdated { runner_id, .. } => runner_id,
            RunnerEvent::PositionClosed { runner_id, .. } => runner_id,
            RunnerEvent::Error { runner_id, .. } => runner_id,
            RunnerEvent::StatsUpdate { runner_id, .. } => runner_id,
        }
    }

    /// Get the timestamp for this event
    pub fn timestamp(&self) -> Option<i64> {
        match self {
            RunnerEvent::RunnerStarted { timestamp, .. } => Some(*timestamp),
            RunnerEvent::RunnerStopped { timestamp, .. } => Some(*timestamp),
            RunnerEvent::TickReceived { data, .. } => Some(data.timestamp),
            RunnerEvent::StateTransition { timestamp, .. } => Some(*timestamp),
            RunnerEvent::ActionExecuted { timestamp, .. } => Some(*timestamp),
            RunnerEvent::PositionOpened { timestamp, .. } => Some(*timestamp),
            RunnerEvent::PositionUpdated { timestamp, .. } => Some(*timestamp),
            RunnerEvent::PositionClosed { timestamp, .. } => Some(*timestamp),
            RunnerEvent::Error { timestamp, .. } => Some(*timestamp),
            RunnerEvent::StatsUpdate { timestamp, .. } => Some(*timestamp),
        }
    }

    /// Check if this is a high-frequency event
    ///
    /// High-frequency events (ticks, position updates) may need throttling.
    pub fn is_high_frequency(&self) -> bool {
        matches!(
            self,
            RunnerEvent::TickReceived { .. } | RunnerEvent::PositionUpdated { .. }
        )
    }

    /// Check if this is a critical event
    ///
    /// Critical events should always be delivered immediately.
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            RunnerEvent::Error {
                severity: ErrorSeverity::Critical,
                ..
            } | RunnerEvent::RunnerStopped { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::action::Side;

    #[test]
    fn test_event_serialization() {
        let event = RunnerEvent::RunnerStarted {
            runner_id: "test_runner".to_string(),
            symbol: "BTCUSDT".to_string(),
            timestamp: 1234567890,
        };

        // Should serialize to JSON
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("RunnerStarted"));
        assert!(json.contains("test_runner"));

        // Should deserialize back
        let deserialized: RunnerEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.runner_id(), "test_runner");
    }

    #[test]
    fn test_state_transition_event() {
        let event = RunnerEvent::StateTransition {
            runner_id: "btc_ema".to_string(),
            from: State::Idle,
            to: State::Analyzing,
            reason: "Opportunity detected".to_string(),
            timestamp: 1234567890,
        };

        assert_eq!(event.runner_id(), "btc_ema");
        assert_eq!(event.timestamp(), Some(1234567890));
        assert!(!event.is_high_frequency());
    }

    #[test]
    fn test_tick_received_event() {
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

        let event = RunnerEvent::TickReceived {
            runner_id: "btc_ema".to_string(),
            symbol: "BTCUSDT".to_string(),
            data,
        };

        assert!(event.is_high_frequency());
        assert!(!event.is_critical());
    }

    #[test]
    fn test_position_events() {
        let position = Position::new(50000.0, 0.1, Side::Long, 1234567890);

        let opened = RunnerEvent::PositionOpened {
            runner_id: "btc_ema".to_string(),
            position: position.clone(),
            timestamp: 1234567890,
        };

        let updated = RunnerEvent::PositionUpdated {
            runner_id: "btc_ema".to_string(),
            current_price: 50500.0,
            unrealized_pnl: 50.0,
            timestamp: 1234567900,
        };

        let closed = RunnerEvent::PositionClosed {
            runner_id: "btc_ema".to_string(),
            exit_price: 51000.0,
            realized_pnl: 100.0,
            reason: "Take profit hit".to_string(),
            timestamp: 1234567910,
        };

        assert!(!opened.is_high_frequency());
        assert!(updated.is_high_frequency());
        assert!(!closed.is_high_frequency());
    }

    #[test]
    fn test_error_severity() {
        let warning = RunnerEvent::Error {
            runner_id: "btc_ema".to_string(),
            error: "Minor issue".to_string(),
            severity: ErrorSeverity::Warning,
            timestamp: 1234567890,
        };

        let critical = RunnerEvent::Error {
            runner_id: "btc_ema".to_string(),
            error: "Fatal error".to_string(),
            severity: ErrorSeverity::Critical,
            timestamp: 1234567890,
        };

        assert!(!warning.is_critical());
        assert!(critical.is_critical());
    }

    #[test]
    fn test_action_executed_event() {
        let event = RunnerEvent::ActionExecuted {
            runner_id: "btc_ema".to_string(),
            action: Action::EnterLong {
                price: 50000.0,
                quantity: 0.1,
            },
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RunnerEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.runner_id(), "btc_ema");
    }

    #[test]
    fn test_stats_update_event() {
        let event = RunnerEvent::StatsUpdate {
            runner_id: "btc_ema".to_string(),
            ticks_processed: 1000,
            actions_executed: 5,
            error_rate: 0.001,
            avg_tick_duration_ms: 0.5,
            timestamp: 1234567890,
        };

        assert!(!event.is_high_frequency());
        assert_eq!(event.runner_id(), "btc_ema");
    }
}
