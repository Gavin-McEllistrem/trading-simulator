use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// API error types
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Runner not found: {0}")]
    RunnerNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Engine error: {0}")]
    EngineError(String),

    #[error("Strategy error: {0}")]
    StrategyError(String),

    #[error("Internal server error")]
    InternalError,
}

/// Error response format
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub error: ErrorDetail,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

impl ApiError {
    /// Convert error to error code
    fn error_code(&self) -> &'static str {
        match self {
            ApiError::RunnerNotFound(_) => "RUNNER_NOT_FOUND",
            ApiError::InvalidRequest(_) => "INVALID_REQUEST",
            ApiError::EngineError(_) => "ENGINE_ERROR",
            ApiError::StrategyError(_) => "STRATEGY_ERROR",
            ApiError::InternalError => "INTERNAL_ERROR",
        }
    }

    /// Convert error to HTTP status code
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::RunnerNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::EngineError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::StrategyError(_) => StatusCode::BAD_REQUEST,
            ApiError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ErrorResponse {
            status: "error".to_string(),
            error: ErrorDetail {
                code: self.error_code().to_string(),
                message: self.to_string(),
            },
            timestamp: chrono::Utc::now().timestamp(),
        };

        (status, Json(error_response)).into_response()
    }
}

/// Convert anyhow errors to ApiError
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::EngineError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(
            ApiError::RunnerNotFound("test".to_string()).error_code(),
            "RUNNER_NOT_FOUND"
        );
        assert_eq!(
            ApiError::InvalidRequest("test".to_string()).error_code(),
            "INVALID_REQUEST"
        );
        assert_eq!(
            ApiError::EngineError("test".to_string()).error_code(),
            "ENGINE_ERROR"
        );
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(
            ApiError::RunnerNotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::InvalidRequest("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::InternalError.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
