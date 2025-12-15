//! Unit tests for market_data module

use super::*;

/// Helper function to create test market data
fn create_test_data(symbol: &str, count: usize) -> Vec<MarketData> {
    (0..count)
        .map(|i| MarketData {
            symbol: symbol.to_string(),
            timestamp: i as i64,
            open: 100.0 + i as f64,
            high: 105.0 + i as f64,
            low: 95.0 + i as f64,
            close: 102.0 + i as f64,
            volume: 1000 + i as u64,
            bid: 101.0 + i as f64,
            ask: 103.0 + i as f64,
        })
        .collect()
}

// ============================================================================
// MarketData Tests
// ============================================================================

#[test]
fn test_market_data_mid_price() {
    let data = MarketData {
        symbol: "BTC".to_string(),
        timestamp: 0,
        open: 100.0,
        high: 110.0,
        low: 90.0,
        close: 105.0,
        volume: 1000,
        bid: 104.0,
        ask: 106.0,
    };

    assert_eq!(data.mid_price(), 105.0);
}

#[test]
fn test_market_data_validate_success() {
    let data = MarketData {
        symbol: "BTC".to_string(),
        timestamp: 0,
        open: 100.0,
        high: 110.0,
        low: 90.0,
        close: 105.0,
        volume: 1000,
        bid: 104.0,
        ask: 106.0,
    };

    assert!(data.validate().is_ok());
}

#[test]
fn test_market_data_validate_high_less_than_low() {
    let data = MarketData {
        symbol: "BTC".to_string(),
        timestamp: 0,
        open: 100.0,
        high: 90.0,  // Invalid!
        low: 110.0,  // Invalid!
        close: 105.0,
        volume: 1000,
        bid: 104.0,
        ask: 106.0,
    };

    assert!(data.validate().is_err());
}

#[test]
fn test_market_data_validate_negative_price() {
    let data = MarketData {
        symbol: "BTC".to_string(),
        timestamp: 0,
        open: -100.0,  // Invalid!
        high: 110.0,
        low: 90.0,
        close: 105.0,
        volume: 1000,
        bid: 104.0,
        ask: 106.0,
    };

    assert!(data.validate().is_err());
}

// ============================================================================
// MarketDataWindow Tests - Basic Operations
// ============================================================================

#[test]
fn test_window_new() {
    let window = MarketDataWindow::new(100);
    assert_eq!(window.len(), 0);
    assert!(window.is_empty());
}

#[test]
fn test_window_push_and_capacity() {
    let mut window = MarketDataWindow::new(3);
    let data = create_test_data("BTC", 5);

    for d in data.iter() {
        window.push(d.clone());
    }

    assert_eq!(window.len(), 3);
    assert_eq!(window.latest().unwrap().timestamp, 4);
    assert_eq!(window.oldest().unwrap().timestamp, 2);
}

#[test]
fn test_window_capacity_limit() {
    let mut window = MarketDataWindow::new(5);

    // Add 10 data points
    for i in 0..10 {
        let data = MarketData {
            symbol: "BTC".to_string(),
            timestamp: i,
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: i as f64,
            volume: 0,
            bid: 0.0,
            ask: 0.0,
        };
        window.push(data);
    }

    // Should only have last 5
    assert_eq!(window.len(), 5);

    // Closes should be 5, 6, 7, 8, 9
    let closes = window.closes(5);
    assert_eq!(closes, vec![5.0, 6.0, 7.0, 8.0, 9.0]);
}

#[test]
fn test_empty_window() {
    let window = MarketDataWindow::new(10);

    assert!(window.is_empty());
    assert_eq!(window.len(), 0);
    assert!(window.high(1).is_none());
    assert!(window.low(1).is_none());
    assert!(window.avg_volume(1).is_none());
    assert!(window.latest().is_none());
    assert!(window.oldest().is_none());
}

#[test]
fn test_window_clear() {
    let mut window = MarketDataWindow::new(100);
    let data = create_test_data("BTC", 10);

    for d in data {
        window.push(d);
    }

    assert_eq!(window.len(), 10);
    window.clear();
    assert_eq!(window.len(), 0);
    assert!(window.is_empty());
    assert!(window.latest().is_none());
}

// ============================================================================
// MarketDataWindow Tests - Query Methods
// ============================================================================

#[test]
fn test_high_low_functions() {
    let mut window = MarketDataWindow::new(10);
    let data = create_test_data("ETH", 5);

    for d in data {
        window.push(d);
    }

    let high = window.high(3).unwrap();
    let low = window.low(3).unwrap();

    assert!(high > low);
    assert_eq!(high, 109.0); // 105 + 4 (last data point, i=4)
    assert_eq!(low, 97.0);   // 95 + 2 (i=2)
}

#[test]
fn test_avg_volume() {
    let mut window = MarketDataWindow::new(10);
    let data = create_test_data("BTC", 5);

    for d in data {
        window.push(d);
    }

    let avg = window.avg_volume(5).unwrap();
    // volumes: 1000, 1001, 1002, 1003, 1004
    // average: 5010 / 5 = 1002
    assert_eq!(avg, 1002.0);
}

#[test]
fn test_closes_extraction() {
    let mut window = MarketDataWindow::new(100);
    let data = create_test_data("ETH", 5);

    for d in data {
        window.push(d);
    }

    let closes = window.closes(3);
    assert_eq!(closes.len(), 3);
    assert_eq!(closes, vec![104.0, 105.0, 106.0]);
}

#[test]
fn test_range_calculation() {
    let mut window = MarketDataWindow::new(100);
    let data = create_test_data("BTC", 5);

    for d in data {
        window.push(d);
    }

    // Highest: 109 (105 + 4), Lowest: 95 (from i=0)
    // Range: 109 - 95 = 14
    let range = window.range(5).unwrap();
    assert_eq!(range, 14.0);
}

#[test]
fn test_oldest_and_latest() {
    let mut window = MarketDataWindow::new(3);
    let data = create_test_data("BTC", 5);

    for d in data {
        window.push(d);
    }

    // Window holds last 3 (timestamps 2, 3, 4)
    assert_eq!(window.oldest().unwrap().timestamp, 2);
    assert_eq!(window.latest().unwrap().timestamp, 4);
}

#[test]
fn test_iterator() {
    let mut window = MarketDataWindow::new(100);
    let data = create_test_data("ETH", 5);

    for d in data {
        window.push(d);
    }

    let timestamps: Vec<i64> = window.iter().map(|d| d.timestamp).collect();
    assert_eq!(timestamps, vec![0, 1, 2, 3, 4]);
}

#[test]
fn test_get_by_index() {
    let mut window = MarketDataWindow::new(100);
    let data = create_test_data("BTC", 5);

    for d in data {
        window.push(d);
    }

    assert_eq!(window.get(0).unwrap().timestamp, 0);
    assert_eq!(window.get(4).unwrap().timestamp, 4);
    assert!(window.get(10).is_none());
}

// ============================================================================
// MarketDataWindow Tests - Edge Cases
// ============================================================================

#[test]
fn test_period_larger_than_window() {
    let mut window = MarketDataWindow::new(100);
    let data = create_test_data("BTC", 5);

    for d in data {
        window.push(d);
    }

    // Request period larger than data available
    let high = window.high(100).unwrap();
    let low = window.low(100).unwrap();

    // Should still work, using all available data
    assert_eq!(high, 109.0);
    assert_eq!(low, 95.0);
}

#[test]
fn test_clone_window() {
    let mut window = MarketDataWindow::new(100);
    let data = create_test_data("BTC", 5);

    for d in data {
        window.push(d);
    }

    let cloned = window.clone();
    assert_eq!(cloned.len(), window.len());
    assert_eq!(cloned.latest().unwrap().timestamp, window.latest().unwrap().timestamp);
}
