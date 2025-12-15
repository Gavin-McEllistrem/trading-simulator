//! Integration tests for market data components
//!
//! These tests verify that the market data system works correctly
//! from an external user's perspective, testing the public API.

use trading_engine::{MarketData, MarketDataWindow, MarketDataStorage};

#[test]
fn test_end_to_end_data_flow() {
    // Create storage
    let storage = MarketDataStorage::new(1000);

    // Simulate incoming market data
    for i in 0..50 {
        let data = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: i,
            open: 50000.0 + i as f64,
            high: 51000.0 + i as f64,
            low: 49000.0 + i as f64,
            close: 50500.0 + i as f64,
            volume: 1000 + i as u64,
            bid: 50499.0 + i as f64,
            ask: 50501.0 + i as f64,
        };

        data.validate().expect("Data should be valid");
        storage.push(data);
    }

    // Retrieve and verify
    let window = storage.get_window("BTCUSDT").expect("Window should exist");
    assert_eq!(window.len(), 50);

    // Test queries
    let high = window.high(20).expect("Should have high");
    let low = window.low(20).expect("Should have low");
    let range = window.range(20).expect("Should calculate range");

    assert!(high > low);
    assert!(range > 0.0);
}

#[test]
fn test_multi_symbol_storage() {
    let storage = MarketDataStorage::new(100);

    // Add data for multiple symbols
    for symbol in &["BTCUSDT", "ETHUSDT", "BNBUSDT"] {
        for i in 0..10 {
            let data = MarketData {
                symbol: symbol.to_string(),
                timestamp: i,
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 105.0,
                volume: 1000,
                bid: 104.0,
                ask: 106.0,
            };
            storage.push(data);
        }
    }

    // Verify each symbol has its own window
    let symbols = storage.symbols();
    assert_eq!(symbols.len(), 3);

    for symbol in &["BTCUSDT", "ETHUSDT", "BNBUSDT"] {
        let window = storage.get_window(symbol).expect("Window should exist");
        assert_eq!(window.len(), 10);
    }
}

#[test]
fn test_storage_isolation() {
    let storage = MarketDataStorage::new(100);

    // Add data to one symbol
    for i in 0..5 {
        let data = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: i,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 1000,
            bid: 104.0,
            ask: 106.0,
        };
        storage.push(data);
    }

    // Verify other symbol is empty
    assert!(storage.get_window("ETHUSDT").is_none());

    // Verify BTCUSDT has data
    assert_eq!(storage.get_window("BTCUSDT").unwrap().len(), 5);
}

#[test]
fn test_window_query_consistency() {
    let mut window = MarketDataWindow::new(100);

    // Add sequential data
    for i in 0..20 {
        let data = MarketData {
            symbol: "TEST".to_string(),
            timestamp: i,
            open: 100.0,
            high: 100.0 + (i as f64 * 0.1),
            low: 100.0 - (i as f64 * 0.1),
            close: 100.0 + (i as f64 * 0.05),
            volume: 1000,
            bid: 100.0,
            ask: 100.0,
        };
        window.push(data);
    }

    // Verify consistency across queries
    let high_10 = window.high(10).unwrap();
    let low_10 = window.low(10).unwrap();
    let range_10 = window.range(10).unwrap();

    // Range should equal high - low
    assert_eq!(range_10, high_10 - low_10);
}

#[test]
fn test_storage_clone_independence() {
    let storage1 = MarketDataStorage::new(100);

    // Add data to storage1
    let data = MarketData {
        symbol: "BTCUSDT".to_string(),
        timestamp: 0,
        open: 100.0,
        high: 110.0,
        low: 90.0,
        close: 105.0,
        volume: 1000,
        bid: 104.0,
        ask: 106.0,
    };
    storage1.push(data.clone());

    // Clone storage
    let storage2 = storage1.clone();

    // Both should have the data (Arc is shared)
    assert_eq!(storage1.get_window("BTCUSDT").unwrap().len(), 1);
    assert_eq!(storage2.get_window("BTCUSDT").unwrap().len(), 1);

    // Add more data to storage2
    storage2.push(data);

    // Both should see the new data (Arc is shared)
    assert_eq!(storage1.get_window("BTCUSDT").unwrap().len(), 2);
    assert_eq!(storage2.get_window("BTCUSDT").unwrap().len(), 2);
}

#[test]
fn test_large_dataset() {
    let mut window = MarketDataWindow::new(10000);

    // Add 10,000 data points
    for i in 0..10000 {
        let data = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: i,
            open: 50000.0,
            high: 51000.0,
            low: 49000.0,
            close: 50500.0,
            volume: 1000,
            bid: 50499.0,
            ask: 50501.0,
        };
        window.push(data);
    }

    assert_eq!(window.len(), 10000);

    // Test queries on large dataset
    let high = window.high(1000);
    let range = window.range(200);

    assert!(high.is_some());
    assert!(range.is_some());
}

#[test]
fn test_realistic_trading_scenario() {
    let storage = MarketDataStorage::new(500);

    // Simulate realistic price movement
    let mut price = 50000.0;
    for i in 0..100 {
        // Random walk
        price += (i % 5) as f64 - 2.0; // Simple oscillation

        let data = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: i,
            open: price,
            high: price + 10.0,
            low: price - 10.0,
            close: price + 5.0,
            volume: 1000 + (i % 100) as u64,
            bid: price + 4.0,
            ask: price + 6.0,
        };

        storage.push(data);
    }

    let window = storage.get_window("BTCUSDT").unwrap();

    // Test typical strategy calculations
    let high_20 = window.high(20).unwrap();
    let low_20 = window.low(20).unwrap();
    let range_20 = window.range(20).unwrap();
    let avg_vol = window.avg_volume(20).unwrap();

    // Verify reasonable values
    assert!(high_20 > low_20);
    assert!(range_20 > 0.0);
    assert!(avg_vol > 0.0);

    // Latest should be accessible
    let latest = window.latest().unwrap();
    assert_eq!(latest.timestamp, 99);
}
