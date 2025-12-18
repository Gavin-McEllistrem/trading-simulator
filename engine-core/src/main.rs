use trading_engine::{MarketDataSource, SimulatedFeed, MarketDataStorage};
use trading_engine::sources::{BinanceFeed, BinanceRegion};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("trading_engine=info")
        .init();

    tracing::info!("Trading Engine Demo");
    tracing::info!("===================\n");

    // Get feed type from command line argument
    let args: Vec<String> = std::env::args().collect();
    let use_binance = args.get(1).map(|s| s.as_str()) == Some("--binance");

    if use_binance {
        run_binance_demo().await?;
    } else {
        run_simulated_demo().await?;
    }

    Ok(())
}

async fn run_simulated_demo() -> anyhow::Result<()> {
    tracing::info!("Running SIMULATED FEED demo");
    tracing::info!("(Use --binance flag for live Binance data)\n");

    // Create a simulated feed for testing
    let mut feed = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);
    let storage = MarketDataStorage::new(1000);

    // Connect and subscribe
    feed.connect().await?;
    feed.subscribe(vec!["BTCUSDT".to_string()]).await?;

    // Collect some data
    tracing::info!("Collecting 10 simulated data points...");
    for i in 0..10 {
        let data = feed.next_tick().await?;
        tracing::info!(
            "Tick {}: {} - O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{} | Bid:{:.2} Ask:{:.2}",
            i + 1,
            data.symbol,
            data.open,
            data.high,
            data.low,
            data.close,
            data.volume,
            data.bid,
            data.ask
        );
        storage.push(data);
    }

    // Disconnect
    feed.disconnect().await?;

    // Display storage stats
    let window = storage.get_window("BTCUSDT").unwrap();
    tracing::info!("\n=== Storage Statistics ===");
    tracing::info!("Total data points: {}", window.len());

    if let (Some(high), Some(low)) = (window.high(10), window.low(10)) {
        tracing::info!("10-period High: {:.2}", high);
        tracing::info!("10-period Low: {:.2}", low);
    }

    if let Some(avg_vol) = window.avg_volume(10) {
        tracing::info!("10-period Avg Volume: {:.0}", avg_vol);
    }

    tracing::info!("\nSimulated demo completed!");

    Ok(())
}

async fn run_binance_demo() -> anyhow::Result<()> {
    tracing::info!("Running BINANCE LIVE FEED demo\n");

    // Create Binance feed for BTC and ETH with 1-minute klines
    // Using Binance.US endpoint (use BinanceRegion::International for global)
    let symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
    let mut feed = BinanceFeed::new_with_region(
        symbols.clone(),
        "1m".to_string(),
        BinanceRegion::US
    );
    let storage = MarketDataStorage::new(1000);

    tracing::info!("Using Binance.US endpoint (wss://stream.binance.us:9443)");

    // Connect and subscribe
    tracing::info!("Connecting to Binance WebSocket...");
    feed.connect().await?;
    feed.subscribe(symbols).await?;
    tracing::info!("Connected successfully!");

    tracing::info!("\nWaiting for completed klines...");
    tracing::info!("Note: 1-minute klines complete at the top of each minute");
    tracing::info!("This demo will collect 3 completed klines (may take up to 3 minutes)\n");

    // Collect 3 completed klines
    for i in 0..3 {
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(90),
            feed.next_tick()
        ).await {
            Ok(Ok(data)) => {
                tracing::info!(
                    "Kline #{}: {} - O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{} | Bid:{:.2} Ask:{:.2}",
                    i + 1,
                    data.symbol,
                    data.open,
                    data.high,
                    data.low,
                    data.close,
                    data.volume,
                    data.bid,
                    data.ask
                );
                storage.push(data);
            }
            Ok(Err(e)) => {
                tracing::error!("Error getting market data: {:?}", e);
                return Err(e.into());
            }
            Err(_) => {
                tracing::error!("Timeout waiting for market data");
                return Err(anyhow::anyhow!("Timeout"));
            }
        }
    }

    // Disconnect
    feed.disconnect().await?;

    // Display storage stats
    tracing::info!("\n=== Storage Statistics ===");
    for symbol in &["BTCUSDT", "ETHUSDT"] {
        if let Some(window) = storage.get_window(symbol) {
            tracing::info!("\n{}: {} data points", symbol, window.len());
            if let Some(latest) = window.latest() {
                tracing::info!("  Latest close: {:.2}", latest.close);
            }
        }
    }

    tracing::info!("\nBinance demo completed!");

    Ok(())
}
