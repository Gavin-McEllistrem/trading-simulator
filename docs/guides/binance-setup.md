# Binance WebSocket Setup Guide

This guide covers setting up and using the Binance WebSocket integration for live market data.

## Overview

The trading engine supports real-time market data from Binance via WebSocket connections. The integration provides:

- **OHLCV candlestick data** via kline streams
- **Real-time bid/ask prices** via bookTicker streams
- Support for **multiple symbols** simultaneously
- **Automatic reconnection** with ping/pong keepalive
- Both **Binance.com** (international) and **Binance.US** endpoints

## Quick Start

### Basic Usage

```rust
use trading_engine::sources::{BinanceFeed, BinanceRegion, MarketDataSource};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create feed for BTC with 1-minute candles
    let symbols = vec!["BTCUSDT".to_string()];
    let mut feed = BinanceFeed::new_with_region(
        symbols.clone(),
        "1m".to_string(),
        BinanceRegion::US  // or BinanceRegion::International
    );

    // Connect
    feed.connect().await?;
    feed.subscribe(symbols).await?;

    // Get market data
    let data = feed.next_tick().await?;
    println!("BTC: ${:.2}", data.close);
    println!("Bid: ${:.2}, Ask: ${:.2}", data.bid, data.ask);

    feed.disconnect().await?;
    Ok(())
}
```

### Running the Demo

The trading engine includes a demo application:

```bash
# Simulated data (no network required)
cargo run

# Live Binance.US data
cargo run -- --binance
```

## Regional Endpoints

### Binance.US (For US Customers)

```rust
use trading_engine::sources::{BinanceFeed, BinanceRegion};

let feed = BinanceFeed::new_with_region(
    vec!["BTCUSDT".to_string()],
    "1m".to_string(),
    BinanceRegion::US
);
```

- **Endpoint:** `wss://stream.binance.us:9443`
- **For:** US-based traders
- **Compliance:** Regulated US exchange

### Binance.com (International)

```rust
use trading_engine::sources::{BinanceFeed, BinanceRegion};

let feed = BinanceFeed::new_with_region(
    vec!["BTCUSDT".to_string()],
    "1m".to_string(),
    BinanceRegion::International
);
```

- **Endpoint:** `wss://stream.binance.com:9443`
- **For:** Non-US traders
- **Note:** Blocked in US and some other regions (HTTP 451 error)

### Default Constructor

```rust
// Defaults to International
let feed = BinanceFeed::new(
    vec!["BTCUSDT".to_string()],
    "1m".to_string()
);
```

## Supported Intervals

The following kline intervals are supported:

| Interval | Description |
|----------|-------------|
| `1s` | 1 second |
| `1m` | 1 minute |
| `3m` | 3 minutes |
| `5m` | 5 minutes |
| `15m` | 15 minutes |
| `30m` | 30 minutes |
| `1h` | 1 hour |
| `2h` | 2 hours |
| `4h` | 4 hours |
| `6h` | 6 hours |
| `8h` | 8 hours |
| `12h` | 12 hours |
| `1d` | 1 day |
| `3d` | 3 days |
| `1w` | 1 week |
| `1M` | 1 month |

## Symbol Formats

Binance symbols follow this format:

- **Crypto:** `BTCUSDT`, `ETHUSDT`, `BNBUSDT`
- **Case insensitive:** The library converts to lowercase automatically
- **No separators:** No dashes or underscores

### Common Symbols

- `BTCUSDT` - Bitcoin vs USDT
- `ETHUSDT` - Ethereum vs USDT
- `BNBUSDT` - Binance Coin vs USDT
- `ADAUSDT` - Cardano vs USDT
- `SOLUSDT` - Solana vs USDT

## Multiple Symbols

Subscribe to multiple symbols simultaneously:

```rust
let symbols = vec![
    "BTCUSDT".to_string(),
    "ETHUSDT".to_string(),
    "BNBUSDT".to_string(),
];

let mut feed = BinanceFeed::new_with_region(
    symbols.clone(),
    "1m".to_string(),
    BinanceRegion::US
);

feed.connect().await?;
feed.subscribe(symbols).await?;

// Each call to next_tick() returns data from any symbol
loop {
    let data = feed.next_tick().await?;
    println!("{}: ${:.2}", data.symbol, data.close);
}
```

## Data Structure

Each market data point contains:

```rust
pub struct MarketData {
    pub symbol: String,      // e.g., "BTCUSDT"
    pub timestamp: i64,      // Unix timestamp in milliseconds
    pub open: f64,           // Opening price
    pub high: f64,           // Highest price
    pub low: f64,            // Lowest price
    pub close: f64,          // Closing price
    pub volume: u64,         // Volume traded
    pub bid: f64,            // Best bid price (from bookTicker)
    pub ask: f64,            // Best ask price (from bookTicker)
}
```

### Bid/Ask Prices

The bid and ask prices come from the real-time bookTicker stream:

- **No approximations** - actual market prices
- Updated in real-time as the order book changes
- Cached and attached to completed klines
- Reflects the true spread at kline completion

## Complete Example with Storage

```rust
use trading_engine::{
    MarketDataSource,
    MarketDataStorage,
    sources::{BinanceFeed, BinanceRegion},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create storage
    let storage = MarketDataStorage::new(1000);

    // Create feed
    let symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
    let mut feed = BinanceFeed::new_with_region(
        symbols.clone(),
        "5m".to_string(),
        BinanceRegion::US
    );

    // Connect
    feed.connect().await?;
    feed.subscribe(symbols).await?;

    // Collect data for 30 minutes (6 x 5-minute klines)
    for i in 0..6 {
        let data = feed.next_tick().await?;

        println!("Kline #{}: {} - Close: ${:.2}",
            i + 1, data.symbol, data.close);

        storage.push(data);
    }

    feed.disconnect().await?;

    // Analyze collected data
    if let Some(btc_window) = storage.get_window("BTCUSDT") {
        println!("\nBTC Analysis:");
        println!("  Data points: {}", btc_window.len());
        if let Some(high) = btc_window.high(6) {
            println!("  High: ${:.2}", high);
        }
        if let Some(low) = btc_window.low(6) {
            println!("  Low: ${:.2}", low);
        }
    }

    Ok(())
}
```

## Error Handling

Common errors and solutions:

### HTTP 451 - Unavailable For Legal Reasons

```
Error: WebSocket error: Failed to connect: HTTP error: 451 Unavailable For Legal Reasons
```

**Cause:** Regional restrictions (accessing Binance.com from US, or Binance.US from outside US)

**Solution:** Use the correct regional endpoint:
- In US: Use `BinanceRegion::US`
- Outside US: Use `BinanceRegion::International`

### Connection Timeout

```
Error: WebSocket error: No message received within 60s
```

**Cause:** Network issues or server unavailable

**Solution:**
- Check internet connection
- Verify Binance status: https://www.binance.com/en/support/announcement
- Retry connection after delay

### Parse Errors

```
Error: Parsing error: Failed to parse Binance kline: ...
```

**Cause:** Unexpected data format from Binance

**Solution:**
- Check symbol format (should be uppercase like "BTCUSDT")
- Verify interval is valid
- Report issue if persistent

## Connection Limits

Binance has rate limits:

- **5 messages/second** per connection
- **1024 streams** maximum per connection
- **300 connections** per 5 minutes per IP
- **24-hour** connection lifetime

The implementation handles these automatically:
- Ping/pong keepalive every 20 seconds
- Automatic pong responses
- 60-second timeout for server responses

## Best Practices

### 1. Use Appropriate Intervals

For different use cases:

- **High-frequency:** `1s`, `1m` (lots of data)
- **Day trading:** `1m`, `5m`, `15m`
- **Swing trading:** `1h`, `4h`, `1d`
- **Long-term:** `1d`, `1w`

### 2. Handle Partial Klines

The implementation only returns **completed** klines:

```rust
// This will wait for the kline to complete
let data = feed.next_tick().await?;
// data.close is the final close price
```

Partial klines (still forming) are filtered out automatically.

### 3. Graceful Shutdown

Always disconnect properly:

```rust
// Set up signal handler
tokio::select! {
    result = feed.next_tick() => {
        // Process data
    }
    _ = tokio::signal::ctrl_c() => {
        println!("Shutting down...");
        feed.disconnect().await?;
    }
}
```

### 4. Use Storage for Analysis

Combine feed with storage for historical analysis:

```rust
let storage = MarketDataStorage::new(100);

loop {
    let data = feed.next_tick().await?;
    storage.push(data);

    // Analyze last 20 periods
    if let Some(window) = storage.get_window("BTCUSDT") {
        if let Some(high) = window.high(20) {
            println!("20-period high: {}", high);
        }
    }
}
```

## Testing

### Integration Tests

Run the Binance integration tests (requires network):

```bash
# Run tests (they're ignored by default)
cargo test --test binance_integration -- --ignored --nocapture

# Run specific test
cargo test --test binance_integration test_binance_connection -- --ignored --nocapture
```

**Note:** These tests are slow (wait for completed klines) and require internet.

### Unit Tests

The feed creation is tested without network:

```bash
cargo test test_binance_feed_creation
```

## Troubleshooting

### Enable Debug Logging

```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

This shows:
- WebSocket connection details
- Kline completion events
- BookTicker updates
- Ping/pong activity

### Check Connection

Verify the WebSocket URL is correct:

```rust
feed.connect().await?;
// Check logs for: "Connecting to Binance WebSocket: wss://..."
```

### Verify Symbol Format

Symbols should be uppercase with no separators:

```rust
// Correct
"BTCUSDT"

// Incorrect
"btc-usdt"
"BTC/USDT"
"btc_usdt"
```

## See Also

- [Architecture Overview](../architecture/01-overview.md)
- [Getting Started Guide](getting-started.md)
- [API Documentation](../../engine-core/README.md) - Run `cargo doc --open`
- [Binance WebSocket API Docs](https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams)

## Next Steps

- Implement indicators using the collected data (Phase 2)
- Build trading strategies with Lua (Phase 4)
- Add backtesting capabilities (Phase 8)
