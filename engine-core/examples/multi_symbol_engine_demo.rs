//! Multi-Symbol Trading Engine Demo
//!
//! This example demonstrates Phase 5: Multi-Symbol Threading Engine with SymbolRunner.
//!
//! Features showcased:
//! - Multiple runners per symbol with different strategies
//! - Concurrent processing across multiple symbols
//! - Health monitoring and statistics
//! - Graceful shutdown
//!
//! Run with: cargo run --example multi_symbol_engine_demo

use trading_engine::{
    SimulatedFeed, MarketDataSource,
    runner::{TradingEngine, RunnerConfig},
    strategy::LuaStrategy,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║   Multi-Symbol Trading Engine Demo (Phase 5)             ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Create the trading engine
    let mut engine = TradingEngine::with_defaults(
        RunnerConfig::development(),
        100, // window size
    );

    println!("📊 Setting up trading engine...\n");

    // Define symbols to trade
    let symbols = vec!["BTCUSDT", "ETHUSDT", "SOLUSDT"];
    let base_prices = vec![50000.0, 3000.0, 100.0];

    // Load strategy
    let strategy_path = "../lua-strategies/test_strategy.lua";
    println!("📜 Loading strategy from: {}\n", strategy_path);

    // Add multiple runners per symbol
    for (symbol, &base_price) in symbols.iter().zip(base_prices.iter()) {
        println!("🔧 Setting up runners for {}:", symbol);

        // Runner 1: EMA Strategy
        let strategy1 = LuaStrategy::new(strategy_path)?;
        engine.add_runner(
            format!("{}_ema", symbol.to_lowercase()),
            *symbol,
            strategy1,
        )?;
        println!("  ✓ Added EMA strategy runner");

        // Runner 2: RSI Strategy
        let strategy2 = LuaStrategy::new(strategy_path)?;
        engine.add_runner_with_config(
            format!("{}_rsi", symbol.to_lowercase()),
            *symbol,
            strategy2,
            150, // larger window for RSI
            RunnerConfig::development(),
        )?;
        println!("  ✓ Added RSI strategy runner");
    }

    println!("\n📈 Engine Summary:");
    println!("{}", engine.summary());
    println!();

    // Create simulated feeds for each symbol
    let mut feeds = vec![];
    for (symbol, &base_price) in symbols.iter().zip(base_prices.iter()) {
        let mut feed = SimulatedFeed::new(symbol.to_string(), base_price);
        feed.connect().await?;
        feed.subscribe(vec![symbol.to_string()]).await?;
        feeds.push((symbol.to_string(), feed));
    }

    println!("🚀 Starting live trading simulation...\n");
    println!("Press Ctrl+C to stop (will run 100 ticks per symbol)\n");

    // Simulate 100 ticks
    for tick in 0..100 {
        if tick % 20 == 0 {
            println!("📊 Tick {}/100", tick);
        }

        // Feed data for each symbol
        for (symbol, feed) in &mut feeds {
            match feed.next_tick().await {
                Ok(data) => {
                    if let Err(e) = engine.feed_data(data).await {
                        eprintln!("❌ Error feeding data for {}: {}", symbol, e);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error getting tick for {}: {}", symbol, e);
                }
            }
        }

        // Small delay to simulate real-time
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Health check every 25 ticks
        if tick % 25 == 0 && tick > 0 {
            println!("\n🏥 Health Check:");
            let unhealthy = engine.unhealthy_runners();
            if unhealthy.is_empty() {
                println!("  ✅ All {} runners healthy", engine.runner_count());
            } else {
                println!("  ⚠️  Unhealthy runners: {:?}", unhealthy);
            }
            println!();
        }
    }

    // Disconnect feeds
    for (symbol, mut feed) in feeds {
        feed.disconnect().await?;
        println!("✓ Disconnected feed for {}", symbol);
    }

    println!("\n📊 Final Engine Summary:");
    println!("{}", engine.summary());
    println!();

    // Show runner uptimes
    println!("⏱️  Runner Uptimes:");
    for runner_id in engine.runner_ids() {
        if let Some(uptime) = engine.runner_uptime(&runner_id) {
            println!("  - {}: {:.2}s", runner_id, uptime.as_secs_f64());
        }
    }
    println!();

    // Graceful shutdown
    println!("🛑 Shutting down engine...");
    let results = engine.shutdown_with_results().await;

    println!("\n📋 Shutdown Results:");
    for (runner_id, result) in results {
        match result {
            Ok(()) => println!("  ✅ {}: OK", runner_id),
            Err(e) => println!("  ❌ {}: {}", runner_id, e),
        }
    }

    println!("\n✨ Demo complete!\n");
    Ok(())
}
