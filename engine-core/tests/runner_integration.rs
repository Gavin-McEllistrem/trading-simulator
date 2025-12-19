//! Integration tests for the Runner system (Phase 5)
//!
//! These tests verify that the SymbolRunner and TradingEngine work correctly
//! with real market data, strategies, and state machines.

use trading_engine::{
    MarketData, SimulatedFeed, MarketDataSource,
    runner::{TradingEngine, SymbolRunner, RunnerConfig},
    strategy::LuaStrategy,
};
use tokio::sync::mpsc;

/// Helper to create test market data
fn create_market_data(symbol: &str, close: f64, timestamp: i64) -> MarketData {
    MarketData {
        symbol: symbol.to_string(),
        timestamp,
        open: close - 10.0,
        high: close + 10.0,
        low: close - 20.0,
        close,
        volume: 1000,
        bid: close - 1.0,
        ask: close + 1.0,
    }
}

#[tokio::test]
async fn test_single_runner_with_strategy() {
    // Create a runner for BTCUSDT
    let (tx, rx) = mpsc::unbounded_channel();
    let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load strategy");

    let mut runner = SymbolRunner::new(
        "BTCUSDT".to_string(),
        strategy,
        rx,
        50,
    );

    // Spawn runner in background
    let runner_task = tokio::spawn(async move {
        runner.run().await
    });

    // Feed some market data
    for i in 0..10 {
        let data = create_market_data("BTCUSDT", 50000.0 + (i as f64 * 100.0), 1000 + i);
        tx.send(data).unwrap();
    }

    // Give it time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Close the channel to stop the runner
    drop(tx);

    // Wait for runner to complete
    let result = runner_task.await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_runners_same_symbol() {
    let mut engine = TradingEngine::new();

    // Load two different strategies (or same strategy with different configs)
    let strategy1 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load strategy 1");
    let strategy2 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load strategy 2");

    // Add two runners for the same symbol
    engine.add_runner("btc_runner_1", "BTCUSDT", strategy1).unwrap();
    engine.add_runner("btc_runner_2", "BTCUSDT", strategy2).unwrap();

    // Verify both runners exist
    assert_eq!(engine.runner_count(), 2);
    assert_eq!(engine.runner_count_for_symbol("BTCUSDT"), 2);

    // Feed data - should broadcast to both runners
    for i in 0..5 {
        let data = create_market_data("BTCUSDT", 50000.0 + (i as f64 * 100.0), 1000 + i);
        engine.feed_data(data).await.unwrap();
    }

    // Give runners time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Verify both runners are healthy
    assert_eq!(engine.runner_is_healthy("btc_runner_1"), Some(true));
    assert_eq!(engine.runner_is_healthy("btc_runner_2"), Some(true));

    // Shutdown
    engine.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_multi_symbol_engine() {
    let mut engine = TradingEngine::new();

    // Load strategies
    let btc_strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load BTC strategy");
    let eth_strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load ETH strategy");
    let sol_strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load SOL strategy");

    // Add runners for different symbols
    engine.add_runner("btc_ema", "BTCUSDT", btc_strategy).unwrap();
    engine.add_runner("eth_ema", "ETHUSDT", eth_strategy).unwrap();
    engine.add_runner("sol_ema", "SOLUSDT", sol_strategy).unwrap();

    // Verify runners are added
    assert_eq!(engine.runner_count(), 3);
    assert_eq!(engine.active_symbols().len(), 3);

    // Feed data for each symbol
    for i in 0..10 {
        let btc_data = create_market_data("BTCUSDT", 50000.0 + (i as f64 * 100.0), 1000 + i);
        let eth_data = create_market_data("ETHUSDT", 3000.0 + (i as f64 * 50.0), 1000 + i);
        let sol_data = create_market_data("SOLUSDT", 100.0 + (i as f64 * 5.0), 1000 + i);

        engine.feed_data(btc_data).await.unwrap();
        engine.feed_data(eth_data).await.unwrap();
        engine.feed_data(sol_data).await.unwrap();
    }

    // Give runners time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check health
    let health = engine.health_check();
    assert_eq!(health.len(), 3);
    assert_eq!(health.get("btc_ema"), Some(&true));
    assert_eq!(health.get("eth_ema"), Some(&true));
    assert_eq!(health.get("sol_ema"), Some(&true));

    // Shutdown
    engine.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_runner_with_simulated_feed() {
    let mut engine = TradingEngine::new();

    // Create simulated feed
    let mut feed = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);

    // Load strategy
    let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load strategy");

    // Add runner
    engine.add_runner("btc_sim", "BTCUSDT", strategy).unwrap();

    // Connect feed
    feed.connect().await.unwrap();
    feed.subscribe(vec!["BTCUSDT".to_string()]).await.unwrap();

    // Feed simulated data
    for _ in 0..20 {
        let data = feed.next_tick().await.unwrap();
        engine.feed_data(data).await.unwrap();
    }

    // Disconnect feed
    feed.disconnect().await.unwrap();

    // Give runner time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Verify runner is healthy
    assert_eq!(engine.runner_is_healthy("btc_sim"), Some(true));

    // Shutdown
    engine.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_runner_with_config() {
    let mut engine = TradingEngine::new();

    // Create production config
    let config = RunnerConfig::production();

    // Load strategy
    let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load strategy");

    // Add runner with custom config
    engine.add_runner_with_config(
        "btc_prod",
        "BTCUSDT",
        strategy,
        200, // larger window
        config,
    ).unwrap();

    // Feed data
    for i in 0..10 {
        let data = create_market_data("BTCUSDT", 50000.0 + (i as f64 * 100.0), 1000 + i);
        engine.feed_data(data).await.unwrap();
    }

    // Give runner time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Shutdown
    engine.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_runner_removal_during_operation() {
    let mut engine = TradingEngine::new();

    // Add runners
    let strategy1 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load strategy");
    let strategy2 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load strategy");

    engine.add_runner("btc_1", "BTCUSDT", strategy1).unwrap();
    engine.add_runner("btc_2", "BTCUSDT", strategy2).unwrap();

    // Feed some data
    for i in 0..5 {
        let data = create_market_data("BTCUSDT", 50000.0 + (i as f64 * 100.0), 1000 + i);
        engine.feed_data(data).await.unwrap();
    }

    // Remove one runner
    engine.remove_runner("btc_1").await.unwrap();

    // Verify state
    assert_eq!(engine.runner_count(), 1);
    assert_eq!(engine.runner_count_for_symbol("BTCUSDT"), 1);
    assert!(!engine.has_runner("btc_1"));
    assert!(engine.has_runner("btc_2"));

    // Continue feeding data to remaining runner
    for i in 5..10 {
        let data = create_market_data("BTCUSDT", 50000.0 + (i as f64 * 100.0), 1000 + i);
        engine.feed_data(data).await.unwrap();
    }

    // Shutdown
    engine.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_engine_health_monitoring() {
    let mut engine = TradingEngine::new();

    // Add multiple runners
    for i in 0..5 {
        let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
            .expect("Failed to load strategy");
        engine.add_runner(
            format!("runner_{}", i),
            "BTCUSDT",
            strategy,
        ).unwrap();
    }

    // Verify all runners are healthy
    assert_eq!(engine.runner_count(), 5);
    assert!(engine.unhealthy_runners().is_empty());

    // Get summary
    let summary = engine.summary();
    assert!(summary.contains("Total Runners: 5"));
    assert!(summary.contains("Healthy: 5"));
    assert!(summary.contains("Unhealthy: 0"));

    // Feed data
    for i in 0..10 {
        let data = create_market_data("BTCUSDT", 50000.0 + (i as f64 * 100.0), 1000 + i);
        engine.feed_data(data).await.unwrap();
    }

    // Give runners time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify all still healthy
    let health = engine.health_check();
    assert_eq!(health.len(), 5);
    for is_healthy in health.values() {
        assert!(is_healthy);
    }

    // Shutdown
    engine.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_concurrent_multi_symbol_processing() {
    let mut engine = TradingEngine::new();

    // Add runners for 3 symbols, 2 strategies each
    let symbols = vec!["BTCUSDT", "ETHUSDT", "SOLUSDT"];

    for symbol in &symbols {
        for i in 1..=2 {
            let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
                .expect("Failed to load strategy");
            engine.add_runner(
                format!("{}_{}", symbol, i),
                *symbol,
                strategy,
            ).unwrap();
        }
    }

    // Verify setup
    assert_eq!(engine.runner_count(), 6); // 3 symbols * 2 runners
    assert_eq!(engine.active_symbols().len(), 3);

    // Feed data concurrently
    for i in 0..20 {
        let btc_data = create_market_data("BTCUSDT", 50000.0 + (i as f64 * 100.0), 1000 + i);
        let eth_data = create_market_data("ETHUSDT", 3000.0 + (i as f64 * 50.0), 1000 + i);
        let sol_data = create_market_data("SOLUSDT", 100.0 + (i as f64 * 5.0), 1000 + i);

        // All three feeds happen "simultaneously"
        engine.feed_data(btc_data).await.unwrap();
        engine.feed_data(eth_data).await.unwrap();
        engine.feed_data(sol_data).await.unwrap();

        // Small delay to simulate real-time ticks
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    }

    // Verify all runners processed data
    assert!(engine.unhealthy_runners().is_empty());

    // Check uptime for all runners
    for symbol in &symbols {
        for i in 1..=2 {
            let runner_id = format!("{}_{}", symbol, i);
            let uptime = engine.runner_uptime(&runner_id).unwrap();
            assert!(uptime.as_millis() > 0);
        }
    }

    // Shutdown
    let results = engine.shutdown_with_results().await;
    assert_eq!(results.len(), 6);
    for (runner_id, result) in results {
        assert!(result.is_ok(), "Runner {} failed: {:?}", runner_id, result);
    }
}

#[tokio::test]
async fn test_error_handling_unknown_symbol() {
    let engine = TradingEngine::new();

    // Try to feed data for a symbol with no runners
    let data = create_market_data("UNKNOWN", 1000.0, 1234567890);
    let result = engine.feed_data(data).await;

    // Should error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_duplicate_runner_id_error() {
    let mut engine = TradingEngine::new();

    let strategy1 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load strategy");
    let strategy2 = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load strategy");

    // Add first runner
    engine.add_runner("my_runner", "BTCUSDT", strategy1).unwrap();

    // Try to add runner with same ID
    let result = engine.add_runner("my_runner", "ETHUSDT", strategy2);

    // Should error
    assert!(result.is_err());
    assert_eq!(engine.runner_count(), 1);
}

#[tokio::test]
async fn test_remove_nonexistent_runner_error() {
    let mut engine = TradingEngine::new();

    // Try to remove a runner that doesn't exist
    let result = engine.remove_runner("nonexistent").await;

    // Should error
    assert!(result.is_err());
}
