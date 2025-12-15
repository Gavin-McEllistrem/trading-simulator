//! Integration tests for the complete data pipeline
//!
//! Tests the flow from data source → storage → queries

use trading_engine::{MarketDataSource, SimulatedFeed, MarketDataStorage};

#[tokio::test]
async fn test_simulated_feed_to_storage() {
    let mut feed = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);
    let storage = MarketDataStorage::new(1000);

    // Connect and subscribe
    feed.connect().await.expect("Should connect");
    feed.subscribe(vec!["BTCUSDT".to_string()])
        .await
        .expect("Should subscribe");

    // Collect 20 ticks
    for _ in 0..20 {
        let data = feed.next_tick().await.expect("Should get tick");
        assert_eq!(data.symbol, "BTCUSDT");
        storage.push(data);
    }

    // Disconnect
    feed.disconnect().await.expect("Should disconnect");

    // Verify storage
    let window = storage.get_window("BTCUSDT").expect("Should have window");
    assert_eq!(window.len(), 20);

    // Verify we can query the data
    let high = window.high(20).expect("Should have high");
    let low = window.low(20).expect("Should have low");
    assert!(high >= low);
}

#[tokio::test]
async fn test_multiple_feeds_to_shared_storage() {
    let storage = MarketDataStorage::new(1000);

    // Create two feeds
    let mut feed1 = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);
    let mut feed2 = SimulatedFeed::new("ETHUSDT".to_string(), 3000.0);

    // Connect both
    feed1.connect().await.expect("Feed1 should connect");
    feed2.connect().await.expect("Feed2 should connect");

    feed1
        .subscribe(vec!["BTCUSDT".to_string()])
        .await
        .expect("Feed1 should subscribe");
    feed2
        .subscribe(vec!["ETHUSDT".to_string()])
        .await
        .expect("Feed2 should subscribe");

    // Collect data from both
    for _ in 0..10 {
        let data1 = feed1.next_tick().await.expect("Should get tick from feed1");
        let data2 = feed2.next_tick().await.expect("Should get tick from feed2");

        storage.push(data1);
        storage.push(data2);
    }

    // Verify both symbols in storage
    let btc_window = storage.get_window("BTCUSDT").expect("BTC should exist");
    let eth_window = storage.get_window("ETHUSDT").expect("ETH should exist");

    assert_eq!(btc_window.len(), 10);
    assert_eq!(eth_window.len(), 10);

    // Verify symbols are listed
    let symbols = storage.symbols();
    assert_eq!(symbols.len(), 2);
    assert!(symbols.contains(&"BTCUSDT".to_string()));
    assert!(symbols.contains(&"ETHUSDT".to_string()));
}

#[tokio::test]
async fn test_feed_error_handling() {
    let mut feed = SimulatedFeed::new("TEST".to_string(), 100.0);

    // Should connect successfully
    assert!(feed.connect().await.is_ok());

    // Should subscribe successfully
    assert!(feed.subscribe(vec!["TEST".to_string()]).await.is_ok());

    // Should get ticks successfully
    for _ in 0..5 {
        assert!(feed.next_tick().await.is_ok());
    }

    // Should disconnect successfully
    assert!(feed.disconnect().await.is_ok());
}

#[tokio::test]
async fn test_storage_under_concurrent_access() {
    let storage = MarketDataStorage::new(1000);
    let storage_clone = storage.clone();

    // Spawn a task that writes data
    let writer = tokio::spawn(async move {
        let mut feed = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);
        feed.connect().await.unwrap();
        feed.subscribe(vec!["BTCUSDT".to_string()]).await.unwrap();

        for _ in 0..50 {
            let data = feed.next_tick().await.unwrap();
            storage_clone.push(data);
        }
    });

    // Wait a bit for writer to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Read from main thread while writer is writing
    for _ in 0..10 {
        if let Some(window) = storage.get_window("BTCUSDT") {
            let _high = window.high(10);
            let _range = window.range(5);
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Wait for writer to finish
    writer.await.expect("Writer task should complete");

    // Verify final state
    let window = storage.get_window("BTCUSDT").expect("Should have window");
    assert_eq!(window.len(), 50);
}

#[tokio::test]
async fn test_data_validation_in_pipeline() {
    let mut feed = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);
    let storage = MarketDataStorage::new(100);

    feed.connect().await.unwrap();
    feed.subscribe(vec!["BTCUSDT".to_string()]).await.unwrap();

    // Collect and validate data
    for _ in 0..10 {
        let data = feed.next_tick().await.unwrap();

        // Validate before storing
        data.validate().expect("Data should be valid");

        // Store validated data
        storage.push(data);
    }

    let window = storage.get_window("BTCUSDT").unwrap();
    assert_eq!(window.len(), 10);

    // All data in window should be valid
    for data in window.iter() {
        assert!(data.validate().is_ok());
    }
}

#[test]
fn test_window_with_realistic_queries() {
    let storage = MarketDataStorage::new(500);

    // Simulate 100 bars of data
    for i in 0..100 {
        let data = trading_engine::MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: i,
            open: 50000.0 + (i as f64 * 10.0),
            high: 50100.0 + (i as f64 * 10.0),
            low: 49900.0 + (i as f64 * 10.0),
            close: 50050.0 + (i as f64 * 10.0),
            volume: 1000,
            bid: 50049.0 + (i as f64 * 10.0),
            ask: 50051.0 + (i as f64 * 10.0),
        };
        storage.push(data);
    }

    let window = storage.get_window("BTCUSDT").unwrap();

    // Test various query periods (common in trading)
    let high_20 = window.high(20).unwrap();
    let high_50 = window.high(50).unwrap();
    let high_100 = window.high(100).unwrap();

    let low_20 = window.low(20).unwrap();
    let low_50 = window.low(50).unwrap();

    let range_20 = window.range(20).unwrap();
    let range_50 = window.range(50).unwrap();

    // Verify queries work
    assert!(high_20 > 0.0);
    assert!(high_50 > 0.0);
    assert!(high_100 > 0.0);
    assert!(low_20 > 0.0);
    assert!(low_50 > 0.0);
    assert!(range_20 > 0.0);
    assert!(range_50 > 0.0);
}
