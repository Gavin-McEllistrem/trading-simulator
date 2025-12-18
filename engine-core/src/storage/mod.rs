//! Thread-safe market data storage for multiple symbols.
//!
//! This module provides [`MarketDataStorage`], a concurrent data structure for storing
//! market data across multiple trading symbols. It uses read-write locks to allow
//! multiple concurrent readers while ensuring safe writes.
//!
//! # Thread Safety
//!
//! The storage uses [`parking_lot::RwLock`] wrapped in [`Arc`] for efficient concurrent access:
//! - Multiple readers can access different symbols simultaneously
//! - Writes are synchronized with a write lock
//! - Cloning the storage is cheap (only clones the Arc, not the data)
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use trading_engine::{MarketDataStorage, MarketData};
//!
//! let storage = MarketDataStorage::new(100);
//!
//! // Push data for multiple symbols
//! let btc_data = MarketData {
//!     symbol: "BTCUSDT".to_string(),
//!     timestamp: 1234567890,
//!     open: 50000.0,
//!     high: 51000.0,
//!     low: 49500.0,
//!     close: 50500.0,
//!     volume: 1000,
//!     bid: 50499.0,
//!     ask: 50501.0,
//! };
//!
//! storage.push(btc_data);
//!
//! // Retrieve window for symbol
//! if let Some(window) = storage.get_window("BTCUSDT") {
//!     assert_eq!(window.len(), 1);
//! }
//! ```
//!
//! ## Multi-threaded Access
//!
//! ```
//! use trading_engine::{MarketDataStorage, MarketData};
//! use std::thread;
//!
//! let storage = MarketDataStorage::new(100);
//!
//! // Clone for thread (cheap Arc clone)
//! let storage_clone = storage.clone();
//!
//! let handle = thread::spawn(move || {
//!     let data = MarketData {
//!         symbol: "ETHUSDT".to_string(),
//!         timestamp: 1234567890,
//!         open: 3000.0,
//!         high: 3100.0,
//!         low: 2950.0,
//!         close: 3050.0,
//!         volume: 500,
//!         bid: 3049.0,
//!         ask: 3051.0,
//!     };
//!     storage_clone.push(data);
//! });
//!
//! handle.join().unwrap();
//!
//! // Can read from original reference
//! assert!(storage.get_window("ETHUSDT").is_some());
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::market_data::{MarketData, MarketDataWindow};

/// Thread-safe storage for market data across multiple symbols.
///
/// This structure manages a collection of [`MarketDataWindow`] instances, one per symbol,
/// with automatic window creation and thread-safe concurrent access.
///
/// # Thread Safety
///
/// Uses `Arc<RwLock<HashMap>>` for lock-free reads across threads and synchronized writes.
/// The [`parking_lot::RwLock`] provides better performance than the standard library version.
///
/// # Examples
///
/// ```
/// use trading_engine::{MarketDataStorage, MarketData};
///
/// let storage = MarketDataStorage::new(100);
///
/// let data = MarketData {
///     symbol: "BTCUSDT".to_string(),
///     timestamp: 1234567890,
///     open: 50000.0,
///     high: 51000.0,
///     low: 49500.0,
///     close: 50500.0,
///     volume: 1000,
///     bid: 50499.0,
///     ask: 50501.0,
/// };
///
/// storage.push(data);
/// assert_eq!(storage.symbols(), vec!["BTCUSDT".to_string()]);
/// ```
pub struct MarketDataStorage {
    windows: Arc<RwLock<HashMap<String, MarketDataWindow>>>,
    window_size: usize,
}

impl MarketDataStorage {
    /// Create a new storage with specified window size.
    ///
    /// # Arguments
    ///
    /// * `window_size` - Maximum number of data points to keep per symbol
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::MarketDataStorage;
    ///
    /// // Store up to 1000 data points per symbol
    /// let storage = MarketDataStorage::new(1000);
    /// ```
    pub fn new(window_size: usize) -> Self {
        Self {
            windows: Arc::new(RwLock::new(HashMap::new())),
            window_size,
        }
    }

    /// Push market data for a symbol.
    ///
    /// If this is the first data point for a symbol, a new window is created automatically.
    /// The data is added to the symbol's window, which maintains a circular buffer of the
    /// most recent `window_size` data points.
    ///
    /// # Arguments
    ///
    /// * `data` - Market data to store
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::{MarketDataStorage, MarketData};
    ///
    /// let storage = MarketDataStorage::new(100);
    ///
    /// let data = MarketData {
    ///     symbol: "BTCUSDT".to_string(),
    ///     timestamp: 1234567890,
    ///     open: 50000.0,
    ///     high: 51000.0,
    ///     low: 49500.0,
    ///     close: 50500.0,
    ///     volume: 1000,
    ///     bid: 50499.0,
    ///     ask: 50501.0,
    /// };
    ///
    /// storage.push(data);
    /// ```
    pub fn push(&self, data: MarketData) {
        let mut windows = self.windows.write();
        let window = windows
            .entry(data.symbol.clone())
            .or_insert_with(|| MarketDataWindow::new(self.window_size));
        window.push(data);
    }

    /// Get a clone of the market data window for a symbol.
    ///
    /// Returns `None` if no data has been stored for this symbol.
    /// The returned window is a clone, so modifications won't affect the storage.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading symbol to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(MarketDataWindow)` - Clone of the symbol's window
    /// * `None` - No data exists for this symbol
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::{MarketDataStorage, MarketData};
    ///
    /// let storage = MarketDataStorage::new(100);
    ///
    /// // No data yet
    /// assert!(storage.get_window("BTCUSDT").is_none());
    ///
    /// // Add data
    /// let data = MarketData {
    ///     symbol: "BTCUSDT".to_string(),
    ///     timestamp: 1234567890,
    ///     open: 50000.0,
    ///     high: 51000.0,
    ///     low: 49500.0,
    ///     close: 50500.0,
    ///     volume: 1000,
    ///     bid: 50499.0,
    ///     ask: 50501.0,
    /// };
    /// storage.push(data);
    ///
    /// // Now window exists
    /// let window = storage.get_window("BTCUSDT").unwrap();
    /// assert_eq!(window.len(), 1);
    /// ```
    pub fn get_window(&self, symbol: &str) -> Option<MarketDataWindow> {
        let windows = self.windows.read();
        windows.get(symbol).cloned()
    }

    /// Get list of all symbols that have data stored.
    ///
    /// # Returns
    ///
    /// Vector of symbol names
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::{MarketDataStorage, MarketData};
    ///
    /// let storage = MarketDataStorage::new(100);
    ///
    /// let btc_data = MarketData {
    ///     symbol: "BTCUSDT".to_string(),
    ///     timestamp: 1234567890,
    ///     open: 50000.0,
    ///     high: 51000.0,
    ///     low: 49500.0,
    ///     close: 50500.0,
    ///     volume: 1000,
    ///     bid: 50499.0,
    ///     ask: 50501.0,
    /// };
    ///
    /// let eth_data = MarketData {
    ///     symbol: "ETHUSDT".to_string(),
    ///     timestamp: 1234567890,
    ///     open: 3000.0,
    ///     high: 3100.0,
    ///     low: 2950.0,
    ///     close: 3050.0,
    ///     volume: 500,
    ///     bid: 3049.0,
    ///     ask: 3051.0,
    /// };
    ///
    /// storage.push(btc_data);
    /// storage.push(eth_data);
    ///
    /// let symbols = storage.symbols();
    /// assert_eq!(symbols.len(), 2);
    /// assert!(symbols.contains(&"BTCUSDT".to_string()));
    /// assert!(symbols.contains(&"ETHUSDT".to_string()));
    /// ```
    pub fn symbols(&self) -> Vec<String> {
        let windows = self.windows.read();
        windows.keys().cloned().collect()
    }

    /// Get a clone of the underlying storage Arc.
    ///
    /// This is useful for advanced use cases where you need direct access to the
    /// underlying `Arc<RwLock<HashMap>>` for custom operations.
    ///
    /// # Returns
    ///
    /// Cloned Arc reference to the internal storage
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::MarketDataStorage;
    ///
    /// let storage = MarketDataStorage::new(100);
    /// let arc_storage = storage.clone_storage();
    ///
    /// // Can read from the Arc directly
    /// let windows = arc_storage.read();
    /// assert_eq!(windows.len(), 0);
    /// ```
    pub fn clone_storage(&self) -> Arc<RwLock<HashMap<String, MarketDataWindow>>> {
        Arc::clone(&self.windows)
    }
}

impl Clone for MarketDataStorage {
    /// Clone the storage (cheap Arc clone).
    ///
    /// This creates a new `MarketDataStorage` that shares the same underlying data
    /// via Arc. Both instances will see the same data and updates.
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::{MarketDataStorage, MarketData};
    ///
    /// let storage1 = MarketDataStorage::new(100);
    /// let storage2 = storage1.clone();
    ///
    /// // Push to one
    /// let data = MarketData {
    ///     symbol: "BTCUSDT".to_string(),
    ///     timestamp: 1234567890,
    ///     open: 50000.0,
    ///     high: 51000.0,
    ///     low: 49500.0,
    ///     close: 50500.0,
    ///     volume: 1000,
    ///     bid: 50499.0,
    ///     ask: 50501.0,
    /// };
    /// storage1.push(data);
    ///
    /// // Visible from the other
    /// assert_eq!(storage2.symbols(), vec!["BTCUSDT".to_string()]);
    /// ```
    fn clone(&self) -> Self {
        Self {
            windows: Arc::clone(&self.windows),
            window_size: self.window_size,
        }
    }
}
