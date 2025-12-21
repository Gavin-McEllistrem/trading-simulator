use std::sync::Arc;
use tokio::sync::Mutex;
use trading_engine::runner::TradingEngine;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    /// The trading engine instance
    pub engine: Arc<Mutex<TradingEngine>>,
}

impl AppState {
    /// Create a new AppState with a TradingEngine
    pub fn new(engine: TradingEngine) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine)),
        }
    }

    /// Get a reference to the engine (for testing/inspection)
    pub fn engine(&self) -> Arc<Mutex<TradingEngine>> {
        self.engine.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        let engine = TradingEngine::new();
        let state = AppState::new(engine);

        // Should be able to clone state
        let _cloned = state.clone();
    }

    #[tokio::test]
    async fn test_app_state_engine_access() {
        let engine = TradingEngine::new();
        let state = AppState::new(engine);

        // Should be able to lock engine
        let engine_lock = state.engine.lock().await;

        // Verify it's a TradingEngine by checking runner count
        assert_eq!(engine_lock.runner_count(), 0);
    }
}
