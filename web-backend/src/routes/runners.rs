use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use trading_engine::{market_data::MarketData, runner::RunnerSnapshot};

use crate::{ApiError, AppState};

/// Query parameters for price history
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    /// Number of recent data points to return (None = all)
    pub count: Option<usize>,
}

/// Request body for adding a runner
#[derive(Debug, Deserialize, Serialize)]
pub struct AddRunnerRequest {
    pub runner_id: String,
    pub symbol: String,
    pub strategy_path: String,
    #[serde(default = "default_window_size")]
    pub window_size: usize,
}

fn default_window_size() -> usize {
    200
}

/// Response for adding a runner
#[derive(Debug, Serialize)]
pub struct AddRunnerResponse {
    pub runner_id: String,
    pub symbol: String,
    pub message: String,
}

/// Get runner snapshot
///
/// Returns complete snapshot of runner state including position, context, and stats.
pub async fn get_runner_snapshot(
    Path(runner_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<RunnerSnapshot>, ApiError> {
    let engine = state.engine.lock().await;

    let snapshot = engine
        .get_runner_snapshot(&runner_id)
        .await
        .ok_or_else(|| ApiError::RunnerNotFound(runner_id.clone()))?;

    Ok(Json(snapshot))
}

/// Get runner price history
///
/// Returns recent price data from the runner's market data window.
pub async fn get_price_history(
    Path(runner_id): Path<String>,
    Query(params): Query<HistoryQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<MarketData>>, ApiError> {
    let engine = state.engine.lock().await;

    let history = engine
        .get_price_history(&runner_id, params.count)
        .await
        .ok_or_else(|| ApiError::RunnerNotFound(runner_id.clone()))?;

    Ok(Json(history))
}

/// Add a new runner
///
/// Creates a new runner with the specified strategy and symbol.
pub async fn add_runner(
    State(state): State<AppState>,
    Json(request): Json<AddRunnerRequest>,
) -> Result<(StatusCode, Json<AddRunnerResponse>), ApiError> {
    use trading_engine::strategy::LuaStrategy;

    // Validate inputs
    if request.runner_id.is_empty() {
        return Err(ApiError::InvalidRequest("runner_id cannot be empty".to_string()));
    }
    if request.symbol.is_empty() {
        return Err(ApiError::InvalidRequest("symbol cannot be empty".to_string()));
    }

    let mut engine = state.engine.lock().await;

    // Check if runner already exists
    if engine.has_runner(&request.runner_id) {
        return Err(ApiError::InvalidRequest(format!(
            "Runner '{}' already exists",
            request.runner_id
        )));
    }

    // Load the Lua strategy
    let strategy = LuaStrategy::new(&request.strategy_path)
        .map_err(|e| ApiError::StrategyError(format!("Failed to load strategy: {}", e)))?;

    // Add the runner to the engine
    engine
        .add_runner(
            request.runner_id.clone(),
            request.symbol.clone(),
            strategy,
        )
        .map_err(|e| ApiError::EngineError(e.to_string()))?;

    let response = AddRunnerResponse {
        runner_id: request.runner_id,
        symbol: request.symbol,
        message: "Runner created successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Remove runner request
pub async fn remove_runner(
    Path(runner_id): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    let mut engine = state.engine.lock().await;

    engine
        .remove_runner(&runner_id)
        .await
        .map_err(|e| ApiError::EngineError(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Response for control operations
#[derive(Debug, Serialize)]
pub struct ControlResponse {
    pub success: bool,
    pub message: String,
}

/// Pause a runner
///
/// Pauses the runner's tick processing while preserving state.
pub async fn pause_runner(
    Path(runner_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ControlResponse>, ApiError> {
    let engine = state.engine.lock().await;

    let success = engine
        .pause_runner(&runner_id)
        .await
        .map_err(|e| ApiError::EngineError(e.to_string()))?;

    let message = if success {
        format!("Runner '{}' paused successfully", runner_id)
    } else {
        format!("Runner '{}' was not in a running state", runner_id)
    };

    Ok(Json(ControlResponse { success, message }))
}

/// Resume a paused runner
///
/// Resumes a paused runner's tick processing.
pub async fn resume_runner(
    Path(runner_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ControlResponse>, ApiError> {
    let engine = state.engine.lock().await;

    let success = engine
        .resume_runner(&runner_id)
        .await
        .map_err(|e| ApiError::EngineError(e.to_string()))?;

    let message = if success {
        format!("Runner '{}' resumed successfully", runner_id)
    } else {
        format!("Runner '{}' was not in a paused state", runner_id)
    };

    Ok(Json(ControlResponse { success, message }))
}

/// Stop a runner
///
/// Stops the runner completely. It cannot be resumed after stopping.
pub async fn stop_runner(
    Path(runner_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ControlResponse>, ApiError> {
    let engine = state.engine.lock().await;

    let success = engine
        .stop_runner(&runner_id)
        .await
        .map_err(|e| ApiError::EngineError(e.to_string()))?;

    let message = format!("Runner '{}' stopped successfully", runner_id);

    Ok(Json(ControlResponse { success, message }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use trading_engine::runner::TradingEngine;

    #[tokio::test]
    async fn test_get_runner_snapshot_not_found() {
        let engine = TradingEngine::new();
        let state = AppState::new(engine);

        let result = get_runner_snapshot(Path("nonexistent".to_string()), State(state)).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_price_history_not_found() {
        let engine = TradingEngine::new();
        let state = AppState::new(engine);

        let query = HistoryQuery { count: Some(10) };
        let result = get_price_history(
            Path("nonexistent".to_string()),
            Query(query),
            State(state),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_runner_not_found() {
        let engine = TradingEngine::new();
        let state = AppState::new(engine);

        let result = remove_runner(Path("nonexistent".to_string()), State(state)).await;

        assert!(result.is_err());
    }
}
