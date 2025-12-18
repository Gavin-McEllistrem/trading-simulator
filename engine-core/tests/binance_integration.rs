// Integration tests for Binance WebSocket feed

use trading_engine::sources::{BinanceFeed, MarketDataSource};
use trading_engine::MarketDataStorage;

#[tokio::test]
#[ignore] // Ignore by default since it requires network connection
async fn test_binance_connection() {
    // Initialize logging for debugging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    // Create a Binance feed for BTC and ETH on 1-minute klines
    let symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
    let mut feed = BinanceFeed::new(symbols.clone(), "1m".to_string());

    // Connect to Binance
    let connect_result = feed.connect().await;
    assert!(connect_result.is_ok(), "Failed to connect: {:?}", connect_result);

    // Subscribe to symbols (in Binance this is already done via URL)
    let subscribe_result = feed.subscribe(symbols).await;
    assert!(subscribe_result.is_ok(), "Failed to subscribe: {:?}", subscribe_result);

    println!("Connected and subscribed successfully!");
    println!("Waiting for completed klines (this may take up to 1 minute)...");

    // Try to get a few market data points (this will wait for completed klines)
    // Note: 1-minute klines complete every minute, so this test is slow
    for i in 1..=3 {
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(90), // Give it 90 seconds per kline
            feed.next_tick()
        ).await {
            Ok(Ok(data)) => {
                println!("\nReceived market data point #{}:", i);
                println!("  Symbol: {}", data.symbol);
                println!("  Timestamp: {}", data.timestamp);
                println!("  Open: {}", data.open);
                println!("  High: {}", data.high);
                println!("  Low: {}", data.low);
                println!("  Close: {}", data.close);
                println!("  Volume: {}", data.volume);
                println!("  Bid: {}", data.bid);
                println!("  Ask: {}", data.ask);

                // Validate the data
                assert!(data.validate().is_ok(), "Invalid market data received");
                assert!(data.bid > 0.0, "Bid price should be positive");
                assert!(data.ask > 0.0, "Ask price should be positive");
                assert!(data.ask >= data.bid, "Ask should be >= Bid");
            }
            Ok(Err(e)) => {
                panic!("Error getting market data: {:?}", e);
            }
            Err(_) => {
                panic!("Timeout waiting for market data");
            }
        }
    }

    // Disconnect
    let disconnect_result = feed.disconnect().await;
    assert!(disconnect_result.is_ok(), "Failed to disconnect: {:?}", disconnect_result);

    println!("\nDisconnected successfully!");
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network connection
async fn test_binance_with_storage() {
    // Initialize logging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    // Create storage
    let storage = MarketDataStorage::new(100);

    // Create Binance feed
    let symbols = vec!["BTCUSDT".to_string()];
    let mut feed = BinanceFeed::new(symbols.clone(), "1m".to_string());

    // Connect
    feed.connect().await.expect("Failed to connect");
    feed.subscribe(symbols).await.expect("Failed to subscribe");

    println!("Connected! Waiting for 2 completed klines...");

    // Collect a few data points
    for i in 1..=2 {
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(90),
            feed.next_tick()
        ).await {
            Ok(Ok(data)) => {
                println!("\nReceived kline #{} for {}", i, data.symbol);
                storage.push(data.clone());

                // Verify storage
                let window = storage.get_window(&data.symbol);
                assert!(window.is_some(), "Window should exist after push");
                assert_eq!(window.unwrap().len(), i, "Window should have {} data points", i);
            }
            Ok(Err(e)) => {
                panic!("Error: {:?}", e);
            }
            Err(_) => {
                panic!("Timeout waiting for kline");
            }
        }
    }

    feed.disconnect().await.expect("Failed to disconnect");
    println!("Test completed successfully!");
}

#[test]
fn test_binance_feed_creation() {
    // Test that we can create a feed without connecting
    let symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
    let feed = BinanceFeed::new(symbols, "5m".to_string());

    assert_eq!(feed.source_name(), "binance");
}
