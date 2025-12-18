//! Technical Indicators Demo
//!
//! Demonstrates both Rust and OCaml indicator implementations.
//!
//! Run with: cargo run --example indicators_demo

use trading_engine::{
    MarketDataSource, MarketDataStorage, SimulatedFeed,
    indicators::*,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    tracing::info!("=== Technical Indicators Demo ===\n");

    // Create simulated feed and collect data
    let mut feed = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);
    let storage = MarketDataStorage::new(100);

    feed.connect().await?;
    feed.subscribe(vec!["BTCUSDT".to_string()]).await?;

    tracing::info!("Collecting 50 data points...");
    for _ in 0..50 {
        let data = feed.next_tick().await?;
        storage.push(data);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    feed.disconnect().await?;

    // Extract closing prices
    let window = storage.get_window("BTCUSDT").unwrap();
    let closes = window.closes(50);

    tracing::info!("\nPrice data collected: {} data points", closes.len());
    tracing::info!("Price range: ${:.2} - ${:.2}\n",
        closes.iter().cloned().fold(f64::INFINITY, f64::min),
        closes.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    );

    // Calculate SMA (Rust implementation)
    tracing::info!("--- Simple Moving Average (SMA) - Rust ---");
    let sma_10 = simple_moving_average(&closes, 10);
    let sma_20 = simple_moving_average(&closes, 20);
    tracing::info!("SMA(10): ${:.2}", sma_10.last().unwrap());
    tracing::info!("SMA(20): ${:.2}", sma_20.last().unwrap());

    // Calculate EMA (Rust implementation)
    tracing::info!("\n--- Exponential Moving Average (EMA) - Rust ---");
    let ema_10 = exponential_moving_average(&closes, 10);
    let ema_20 = exponential_moving_average(&closes, 20);
    tracing::info!("EMA(10): ${:.2}", ema_10.last().unwrap());
    tracing::info!("EMA(20): ${:.2}", ema_20.last().unwrap());

    // Calculate RSI (Rust implementation)
    tracing::info!("\n--- Relative Strength Index (RSI) - Rust ---");
    let rsi = relative_strength_index(&closes, 14);
    let current_rsi = rsi.last().unwrap();
    tracing::info!("RSI(14): {:.2}", current_rsi);
    if *current_rsi > 70.0 {
        tracing::info!("  → Overbought territory!");
    } else if *current_rsi < 30.0 {
        tracing::info!("  → Oversold territory!");
    } else {
        tracing::info!("  → Neutral territory");
    }

    // Calculate MACD (Rust implementation)
    tracing::info!("\n--- MACD (12, 26, 9) - Rust ---");
    let macd_result = macd(&closes, 12, 26, 9);
    tracing::info!("MACD Line: {:.2}", macd_result.macd_line.last().unwrap());
    tracing::info!("Signal Line: {:.2}", macd_result.signal_line.last().unwrap());
    tracing::info!("Histogram: {:.2}", macd_result.histogram.last().unwrap());
    if *macd_result.histogram.last().unwrap() > 0.0 {
        tracing::info!("  → Bullish signal (MACD above signal)");
    } else {
        tracing::info!("  → Bearish signal (MACD below signal)");
    }

    // Calculate Bollinger Bands (Rust implementation)
    tracing::info!("\n--- Bollinger Bands (20, 2.0) - Rust ---");
    let bb = bollinger_bands(&closes, 20, 2.0);
    let current_price = closes.last().unwrap();
    tracing::info!("Upper Band: ${:.2}", bb.upper.last().unwrap());
    tracing::info!("Middle Band: ${:.2}", bb.middle.last().unwrap());
    tracing::info!("Lower Band: ${:.2}", bb.lower.last().unwrap());
    tracing::info!("Current Price: ${:.2}", current_price);

    let band_width = bb.upper.last().unwrap() - bb.lower.last().unwrap();
    tracing::info!("Band Width: ${:.2}", band_width);

    // Determine position within bands
    let upper = bb.upper.last().unwrap();
    let lower = bb.lower.last().unwrap();
    if current_price > upper {
        tracing::info!("  → Price above upper band (potential reversal)");
    } else if current_price < lower {
        tracing::info!("  → Price below lower band (potential reversal)");
    } else {
        let middle = bb.middle.last().unwrap();
        if current_price > middle {
            tracing::info!("  → Price in upper half of bands");
        } else {
            tracing::info!("  → Price in lower half of bands");
        }
    }

    // Verify with OCaml implementation
    tracing::info!("\n--- Verification with OCaml Implementation ---");
    tracing::info!("Comparing Rust and OCaml results...");

    match ocaml::sma_ocaml(&closes, 10) {
        Ok(ocaml_sma) => {
            let rust_val = sma_10.last().unwrap();
            let ocaml_val = ocaml_sma.last().unwrap();
            let diff = (rust_val - ocaml_val).abs();
            tracing::info!("SMA(10): Rust=${:.2}, OCaml=${:.2}, diff={:.6}", rust_val, ocaml_val, diff);
            if diff < 0.001 {
                tracing::info!("  ✓ Implementations match!");
            }
        }
        Err(e) => tracing::warn!("OCaml SMA failed: {} (OCaml binary may not be built)", e),
    }

    tracing::info!("\n✅ Demo complete!\n");
    tracing::info!("Note: Rust implementation is used for production.");
    tracing::info!("OCaml implementation serves as a reference for correctness verification.");

    Ok(())
}
