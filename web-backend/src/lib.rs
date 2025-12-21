pub mod error;
pub mod routes;
pub mod state;
pub mod websocket;

pub use error::{ApiError, ErrorResponse};
pub use state::AppState;

use anyhow::Result;
use axum::{
    routing::{delete, get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

/// Configuration for the web server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            enable_cors: true,
        }
    }
}


/// Build the application router with all routes and middleware
pub fn build_router(state: AppState) -> Router {
    let mut router = Router::new()
        // Health endpoints
        .route("/health", get(routes::health::health_check))
        .route("/api/engine/health", get(routes::engine::engine_health))
        .route("/api/engine/summary", get(routes::engine::engine_summary))
        // Runner endpoints
        .route(
            "/api/runners/:id/snapshot",
            get(routes::runners::get_runner_snapshot),
        )
        .route(
            "/api/runners/:id/history",
            get(routes::runners::get_price_history),
        )
        .route("/api/runners", post(routes::runners::add_runner))
        .route("/api/runners/:id", delete(routes::runners::remove_runner))
        .with_state(state);

    // Add CORS middleware
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    router = router.layer(cors);

    // Add request logging/tracing middleware
    router = router.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO)),
    );

    router
}

/// Start the HTTP server
pub async fn start_server(config: ServerConfig, state: AppState) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;

    tracing::info!("Starting server on {}", addr);

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert!(config.enable_cors);
    }

    #[tokio::test]
    async fn test_router_builds() {
        use trading_engine::runner::TradingEngine;

        let engine = TradingEngine::new();
        let state = AppState::new(engine);
        let _router = build_router(state);
        // Router builds successfully
    }
}
