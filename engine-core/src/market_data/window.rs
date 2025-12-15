//! Market data window implementation using a circular buffer.
//!
//! This module provides [`MarketDataWindow`], a fixed-size circular buffer
//! for storing and querying recent market data efficiently.

use std::collections::VecDeque;
use super::MarketData;

/// A circular buffer for storing recent market data.
///
/// `MarketDataWindow` maintains a fixed-size buffer of recent [`MarketData`] points,
/// automatically removing the oldest data when the buffer is full. This design
/// prevents unbounded memory growth in long-running processes while providing
/// efficient access to recent data for indicator calculations.
///
/// # Design
///
/// - **Fixed Size**: Capacity set at creation, prevents memory leaks
/// - **FIFO**: Oldest data removed first (First-In-First-Out)
/// - **Circular**: Uses `VecDeque` for efficient push/pop operations
/// - **Recent Access**: Most queries operate on recent N bars
///
/// # Performance
///
/// - `push()`: O(1) amortized
/// - `high(n)`, `low(n)`, `avg_volume(n)`: O(n)
/// - Memory: O(max_size)
///
/// # Thread Safety
///
/// This type is **not** thread-safe. For concurrent access, use
/// [`MarketDataStorage`](crate::storage::MarketDataStorage) which provides
/// thread-safe access via `RwLock`.
///
/// # Examples
///
/// ```
/// use trading_engine::{MarketData, MarketDataWindow};
///
/// let mut window = MarketDataWindow::new(100);
///
/// // Add data points
/// for i in 0..150 {
///     let data = MarketData {
///         symbol: "BTCUSDT".to_string(),
///         timestamp: i,
///         open: 50000.0,
///         high: 51000.0,
///         low: 49000.0,
///         close: 50500.0,
///         volume: 1000,
///         bid: 50499.0,
///         ask: 50501.0,
///     };
///     window.push(data);
/// }
///
/// // Window only keeps last 100
/// assert_eq!(window.len(), 100);
///
/// // Query recent highs
/// let high_20 = window.high(20).unwrap();
/// assert!(high_20 > 0.0);
/// ```
pub struct MarketDataWindow {
    data: VecDeque<MarketData>,
    max_size: usize,
}

impl MarketDataWindow {
    /// Creates a new market data window with the specified maximum size.
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum number of data points to store
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::MarketDataWindow;
    ///
    /// // Create a window that holds 1000 bars
    /// let window = MarketDataWindow::new(1000);
    /// assert_eq!(window.len(), 0);
    /// assert!(window.is_empty());
    /// ```
    pub fn new(max_size: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Adds a new market data point to the window.
    ///
    /// If the window is at capacity, the oldest data point is removed first.
    /// This maintains a sliding window of the most recent N data points.
    ///
    /// # Performance
    ///
    /// O(1) amortized time complexity.
    ///
    /// # Arguments
    ///
    /// * `market_data` - The market data point to add
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::{MarketData, MarketDataWindow};
    ///
    /// let mut window = MarketDataWindow::new(3);
    ///
    /// for i in 0..5 {
    ///     let data = MarketData {
    ///         symbol: "BTC".to_string(),
    ///         timestamp: i,
    ///         open: 0.0, high: 0.0, low: 0.0, close: 0.0,
    ///         volume: 0, bid: 0.0, ask: 0.0,
    ///     };
    ///     window.push(data);
    /// }
    ///
    /// // Only keeps last 3 despite adding 5
    /// assert_eq!(window.len(), 3);
    /// ```
    pub fn push(&mut self, market_data: MarketData) {
        if self.data.len() >= self.max_size {
            self.data.pop_front();
        }
        self.data.push_back(market_data);
    }

    /// Returns the highest high price over the last `period` bars.
    ///
    /// Searches the most recent `period` bars and returns the maximum
    /// high price. If `period` exceeds the window size, searches all
    /// available bars.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of recent bars to search
    ///
    /// # Returns
    ///
    /// - `Some(f64)` - The highest high price in the period
    /// - `None` - If the window is empty
    ///
    /// # Performance
    ///
    /// O(min(period, window.len()))
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::{MarketData, MarketDataWindow};
    ///
    /// let mut window = MarketDataWindow::new(100);
    ///
    /// // Add data with increasing highs
    /// for i in 0..10 {
    ///     let data = MarketData {
    ///         symbol: "BTC".to_string(),
    ///         timestamp: i,
    ///         open: 0.0,
    ///         high: 100.0 + i as f64,
    ///         low: 0.0,
    ///         close: 0.0,
    ///         volume: 0, bid: 0.0, ask: 0.0,
    ///     };
    ///     window.push(data);
    /// }
    ///
    /// // Most recent high is 109.0 (100 + 9)
    /// assert_eq!(window.high(10).unwrap(), 109.0);
    ///
    /// // Last 5 bars: highest is 109.0
    /// assert_eq!(window.high(5).unwrap(), 109.0);
    /// ```
    pub fn high(&self, period: usize) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }

        self.data
            .iter()
            .rev()
            .take(period)
            .map(|d| d.high)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn low(&self, period: usize) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }

        self.data
            .iter()
            .rev()
            .take(period)
            .map(|d| d.low)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn avg_volume(&self, period: usize) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }

        let values: Vec<u64> = self.data
            .iter()
            .rev()
            .take(period)
            .map(|d| d.volume)
            .collect();

        if values.is_empty() {
            return None;
        }

        Some(values.iter().sum::<u64>() as f64 / values.len() as f64)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&MarketData> {
        self.data.get(index)
    }

    pub fn latest(&self) -> Option<&MarketData> {
        self.data.back()
    }

    /// Returns the oldest data point in the window.
    ///
    /// # Returns
    ///
    /// - `Some(&MarketData)` - Reference to the oldest data point
    /// - `None` - If the window is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::{MarketData, MarketDataWindow};
    ///
    /// let mut window = MarketDataWindow::new(3);
    ///
    /// for i in 0..5 {
    ///     let data = MarketData {
    ///         symbol: "BTC".to_string(),
    ///         timestamp: i,
    ///         open: 0.0, high: 0.0, low: 0.0, close: 0.0,
    ///         volume: 0, bid: 0.0, ask: 0.0,
    ///     };
    ///     window.push(data);
    /// }
    ///
    /// // Window holds last 3 (timestamps 2, 3, 4)
    /// assert_eq!(window.oldest().unwrap().timestamp, 2);
    /// ```
    pub fn oldest(&self) -> Option<&MarketData> {
        self.data.front()
    }

    /// Returns an iterator over the data points from oldest to newest.
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::{MarketData, MarketDataWindow};
    ///
    /// let mut window = MarketDataWindow::new(100);
    ///
    /// for i in 0..10 {
    ///     let data = MarketData {
    ///         symbol: "BTC".to_string(),
    ///         timestamp: i,
    ///         open: 0.0, high: 0.0, low: 0.0, close: 0.0,
    ///         volume: 0, bid: 0.0, ask: 0.0,
    ///     };
    ///     window.push(data);
    /// }
    ///
    /// let timestamps: Vec<i64> = window.iter().map(|d| d.timestamp).collect();
    /// assert_eq!(timestamps, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &MarketData> {
        self.data.iter()
    }

    /// Returns the closing prices for the last `period` bars.
    ///
    /// Returns the most recent `period` closing prices, ordered from oldest to newest.
    /// If `period` exceeds the window size, returns all available closes.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of recent bars to retrieve
    ///
    /// # Returns
    ///
    /// A vector of closing prices, or empty vector if window is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::{MarketData, MarketDataWindow};
    ///
    /// let mut window = MarketDataWindow::new(100);
    ///
    /// for i in 0..5 {
    ///     let data = MarketData {
    ///         symbol: "BTC".to_string(),
    ///         timestamp: i,
    ///         open: 0.0, high: 0.0, low: 0.0,
    ///         close: 100.0 + i as f64,
    ///         volume: 0, bid: 0.0, ask: 0.0,
    ///     };
    ///     window.push(data);
    /// }
    ///
    /// let closes = window.closes(3);
    /// assert_eq!(closes, vec![102.0, 103.0, 104.0]);
    /// ```
    pub fn closes(&self, period: usize) -> Vec<f64> {
        self.data
            .iter()
            .rev()
            .take(period)
            .map(|d| d.close)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Returns the price range (high - low) for the last `period` bars.
    ///
    /// Calculates the difference between the highest high and lowest low
    /// over the specified period.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of recent bars to analyze
    ///
    /// # Returns
    ///
    /// - `Some(f64)` - The price range (high - low)
    /// - `None` - If the window is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::{MarketData, MarketDataWindow};
    ///
    /// let mut window = MarketDataWindow::new(100);
    ///
    /// for i in 0..5 {
    ///     let data = MarketData {
    ///         symbol: "BTC".to_string(),
    ///         timestamp: i,
    ///         open: 0.0,
    ///         high: 110.0 + i as f64,
    ///         low: 90.0 - i as f64,
    ///         close: 0.0,
    ///         volume: 0, bid: 0.0, ask: 0.0,
    ///     };
    ///     window.push(data);
    /// }
    ///
    /// // High: 114 (110 + 4), Low: 86 (90 - 4)
    /// // Range: 114 - 86 = 28
    /// assert_eq!(window.range(5).unwrap(), 28.0);
    /// ```
    pub fn range(&self, period: usize) -> Option<f64> {
        let high = self.high(period)?;
        let low = self.low(period)?;
        Some(high - low)
    }

    /// Clears all data from the window.
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::{MarketData, MarketDataWindow};
    ///
    /// let mut window = MarketDataWindow::new(100);
    ///
    /// let data = MarketData {
    ///     symbol: "BTC".to_string(),
    ///     timestamp: 0,
    ///     open: 0.0, high: 0.0, low: 0.0, close: 0.0,
    ///     volume: 0, bid: 0.0, ask: 0.0,
    /// };
    /// window.push(data);
    ///
    /// assert_eq!(window.len(), 1);
    /// window.clear();
    /// assert_eq!(window.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Clone for MarketDataWindow {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            max_size: self.max_size,
        }
    }
}
