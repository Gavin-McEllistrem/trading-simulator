use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{ApiError, AppState};

/// Engine health response
#[derive(Debug, Serialize, Deserialize)]
pub struct EngineHealthResponse {
    pub status: String,
    pub runners_count: usize,
    pub healthy_runners: usize,
    pub timestamp: i64,
}

/// Engine health check endpoint
///
/// Returns the status of the trading engine and its runners.
/// This endpoint will query the TradingEngine for health information.
pub async fn engine_health(
    State(state): State<AppState>,
) -> (StatusCode, Json<EngineHealthResponse>) {
    let engine = state.engine.lock().await;

    let total_runners = engine.runner_count();
    let unhealthy = engine.unhealthy_runners().len();
    let healthy_runners = total_runners - unhealthy;

    let response = EngineHealthResponse {
        status: "ok".to_string(),
        runners_count: total_runners,
        healthy_runners,
        timestamp: chrono::Utc::now().timestamp(),
    };

    (StatusCode::OK, Json(response))
}

/// Runner summary information
#[derive(Debug, Serialize, Deserialize)]
pub struct RunnerSummary {
    pub runner_id: String,
    pub symbol: String,
}

/// Engine summary response
#[derive(Debug, Serialize, Deserialize)]
pub struct EngineSummaryResponse {
    pub status: String,
    pub total_runners: usize,
    pub healthy_runners: usize,
    pub active_symbols: Vec<String>,
    pub runners: Vec<RunnerSummary>,
    pub timestamp: i64,
}

/// Get engine summary
///
/// Returns comprehensive engine state including all runners and their symbols.
pub async fn engine_summary(
    State(state): State<AppState>,
) -> Result<Json<EngineSummaryResponse>, ApiError> {
    let engine = state.engine.lock().await;

    let total_runners = engine.runner_count();
    let unhealthy = engine.unhealthy_runners().len();
    let healthy_runners = total_runners - unhealthy;
    let active_symbols = engine.active_symbols();

    // Get all runners
    let runners = engine
        .runner_ids()
        .into_iter()
        .filter_map(|runner_id| {
            engine
                .runner_symbol(&runner_id)
                .map(|symbol| RunnerSummary { runner_id, symbol })
        })
        .collect();

    let response = EngineSummaryResponse {
        status: "ok".to_string(),
        total_runners,
        healthy_runners,
        active_symbols,
        runners,
        timestamp: chrono::Utc::now().timestamp(),
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;

    #[tokio::test]
    async fn test_engine_health() {
        use trading_engine::runner::TradingEngine;

        let engine = TradingEngine::new();
        let state = AppState::new(engine);
        let (status, Json(response)) = engine_health(State(state)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response.status, "ok");
        assert_eq!(response.runners_count, 0);
        assert_eq!(response.healthy_runners, 0);
    }
}
