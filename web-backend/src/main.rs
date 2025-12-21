use anyhow::Result;
use std::collections::HashSet;
use trading_engine::runner::TradingEngine;
use trading_engine::sources::{BinanceFeed, BinanceRegion, MarketDataSource};
use trading_web_backend::{start_server, AppState, ServerConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trading_web_backend=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = ServerConfig::default();

    tracing::info!(
        "Trading System Web Backend v{}",
        env!("CARGO_PKG_VERSION")
    );
    tracing::info!("Server will listen on {}:{}", config.host, config.port);

    // Create application state
    tracing::info!("Initializing trading engine...");
    let engine = TradingEngine::new();
    let state = AppState::new(engine);
    tracing::info!("Trading engine initialized");

    // Spawn background task to feed market data
    let feed_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_market_data_feed(feed_state).await {
            tracing::error!("Market data feed error: {}", e);
        }
    });

    // Start the server
    start_server(config, state).await?;

    Ok(())
}

/// Background task that feeds market data from Binance to the engine
async fn run_market_data_feed(state: AppState) -> Result<()> {
    loop {
        // Get current symbols from engine
        let symbols = {
            let engine = state.engine.lock().await;
            engine.active_symbols()
        };

        if symbols.is_empty() {
            // No runners yet, wait and retry
            tracing::debug!("No active symbols, waiting for runners...");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        tracing::info!("Starting Binance US feed for symbols: {:?}", symbols);

        // Create and connect Binance feed with symbols and 1m interval (using US region)
        let mut feed = BinanceFeed::new_with_region(
            symbols.clone(),
            "1m".to_string(),
            BinanceRegion::US,
        );
        if let Err(e) = feed.connect().await {
            tracing::error!("Failed to connect to Binance US: {}", e);
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            continue;
        }

        let subscribed_symbols: HashSet<String> = symbols.iter().cloned().collect();

        // Feed data loop
        loop {
            // Check if symbols changed
            let current_symbols = {
                let engine = state.engine.lock().await;
                engine.active_symbols().iter().cloned().collect::<HashSet<_>>()
            };

            // If symbols changed, reconnect
            if current_symbols != subscribed_symbols {
                tracing::info!("Symbols changed, reconnecting feed...");
                if let Err(e) = feed.disconnect().await {
                    tracing::warn!("Error disconnecting feed: {}", e);
                }
                break; // Break inner loop to reconnect
            }

            // Get next tick
            match feed.next_tick().await {
                Ok(data) => {
                    let symbol = data.symbol.clone();
                    let price = data.close;

                    // Feed to engine
                    let engine = state.engine.lock().await;
                    if let Err(e) = engine.feed_data(data).await {
                        tracing::warn!("Failed to feed data for {}: {}", symbol, e);
                    } else {
                        tracing::debug!("Fed data for {} at price {}", symbol, price);
                    }
                }
                Err(e) => {
                    tracing::error!("Error receiving tick: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    break; // Break inner loop to reconnect
                }
            }
        }

        // Disconnect and retry
        if let Err(e) = feed.disconnect().await {
            tracing::warn!("Error disconnecting feed: {}", e);
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
