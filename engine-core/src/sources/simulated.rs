// Simulated market data feed for testing

use super::*;
use async_trait::async_trait;
use rand::Rng;
use std::time::Duration;

pub struct SimulatedFeed {
    symbol: String,
    current_price: f64,
    tick_count: u64,
}

impl SimulatedFeed {
    pub fn new(symbol: String, starting_price: f64) -> Self {
        Self {
            symbol,
            current_price: starting_price,
            tick_count: 0,
        }
    }
}

#[async_trait]
impl MarketDataSource for SimulatedFeed {
    async fn connect(&mut self) -> Result<()> {
        tracing::info!("Simulated feed connected for {}", self.symbol);
        Ok(())
    }

    async fn subscribe(&mut self, symbols: Vec<String>) -> Result<()> {
        tracing::info!("Simulated feed subscribed to: {:?}", symbols);
        Ok(())
    }

    async fn next_tick(&mut self) -> Result<MarketData> {
        // Simulate delay between ticks
        tokio::time::sleep(Duration::from_millis(100)).await;

        let mut rng = rand::thread_rng();

        // Simulate price movement (random walk)
        let change_percent = rng.gen_range(-0.02..0.02); // +/- 2%
        let change = self.current_price * change_percent;
        self.current_price += change;

        // Generate OHLC data
        let volatility = self.current_price * 0.005; // 0.5% volatility
        let high = self.current_price + rng.gen_range(0.0..volatility);
        let low = self.current_price - rng.gen_range(0.0..volatility);
        let open = rng.gen_range(low..=high);
        let close = rng.gen_range(low..=high);

        // Generate volume
        let base_volume = 1000;
        let volume = base_volume + rng.gen_range(0..500);

        // Generate bid/ask spread
        let spread = self.current_price * 0.001; // 0.1% spread
        let bid = close - spread / 2.0;
        let ask = close + spread / 2.0;

        self.tick_count += 1;

        let data = MarketData {
            symbol: self.symbol.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            open,
            high,
            low,
            close,
            volume,
            bid,
            ask,
        };

        Ok(data)
    }

    async fn disconnect(&mut self) -> Result<()> {
        tracing::info!("Simulated feed disconnected");
        Ok(())
    }

    fn source_name(&self) -> &str {
        "simulated"
    }
}
