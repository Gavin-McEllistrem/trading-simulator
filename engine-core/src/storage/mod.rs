// Market data storage with thread-safe access

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::market_data::{MarketData, MarketDataWindow};

pub struct MarketDataStorage {
    windows: Arc<RwLock<HashMap<String, MarketDataWindow>>>,
    window_size: usize,
}

impl MarketDataStorage {
    pub fn new(window_size: usize) -> Self {
        Self {
            windows: Arc::new(RwLock::new(HashMap::new())),
            window_size,
        }
    }

    pub fn push(&self, data: MarketData) {
        let mut windows = self.windows.write();
        let window = windows
            .entry(data.symbol.clone())
            .or_insert_with(|| MarketDataWindow::new(self.window_size));
        window.push(data);
    }

    pub fn get_window(&self, symbol: &str) -> Option<MarketDataWindow> {
        let windows = self.windows.read();
        windows.get(symbol).cloned()
    }

    pub fn symbols(&self) -> Vec<String> {
        let windows = self.windows.read();
        windows.keys().cloned().collect()
    }

    pub fn clone_storage(&self) -> Arc<RwLock<HashMap<String, MarketDataWindow>>> {
        Arc::clone(&self.windows)
    }
}

impl Clone for MarketDataStorage {
    fn clone(&self) -> Self {
        Self {
            windows: Arc::clone(&self.windows),
            window_size: self.window_size,
        }
    }
}
