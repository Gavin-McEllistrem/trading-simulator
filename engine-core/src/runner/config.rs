//! Runner configuration

use serde::{Deserialize, Serialize};

/// Configuration for a SymbolRunner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerConfig {
    /// Stop runner on first error (vs continue)
    pub stop_on_error: bool,

    /// Log every action execution
    pub log_actions: bool,

    /// Log position updates
    pub log_positions: bool,

    /// Enable performance metrics collection
    pub collect_metrics: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            stop_on_error: false,
            log_actions: true,
            log_positions: false,
            collect_metrics: true,
        }
    }
}

impl RunnerConfig {
    /// Create a configuration for production use
    pub fn production() -> Self {
        Self {
            stop_on_error: true,
            log_actions: true,
            log_positions: true,
            collect_metrics: true,
        }
    }

    /// Create a configuration for development/testing
    pub fn development() -> Self {
        Self {
            stop_on_error: false,
            log_actions: true,
            log_positions: false,
            collect_metrics: false,
        }
    }

    /// Create a quiet configuration (minimal logging)
    pub fn quiet() -> Self {
        Self {
            stop_on_error: false,
            log_actions: false,
            log_positions: false,
            collect_metrics: true,
        }
    }
}
