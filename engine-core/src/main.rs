use trading_engine::{MarketDataSource, SimulatedFeed, MarketDataStorage};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("trading_engine=debug,info")
        .init();

    tracing::info!("Trading Engine starting...");

    // Create a simulated feed for testing
    let mut feed = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);
    let storage = MarketDataStorage::new(1000);

    // Connect and subscribe
    feed.connect().await?;
    feed.subscribe(vec!["BTCUSDT".to_string()]).await?;

    // Collect some data
    tracing::info!("Collecting market data...");
    for i in 0..10 {
        let data = feed.next_tick().await?;
        tracing::info!(
            "Tick {}: {} - close: {:.2}, volume: {}",
            i + 1,
            data.symbol,
            data.close,
            data.volume
        );
        storage.push(data);
    }

    // Disconnect
    feed.disconnect().await?;

    // Display storage stats
    let window = storage.get_window("BTCUSDT").unwrap();
    tracing::info!("Storage contains {} data points", window.len());

    if let (Some(high), Some(low)) = (window.high(10), window.low(10)) {
        tracing::info!("10-period High: {:.2}, Low: {:.2}", high, low);
    }

    tracing::info!("Trading Engine stopped");

    Ok(())
}
