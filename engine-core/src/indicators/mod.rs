//! Technical Indicators Module
//!
//! This module provides technical indicator calculations, with implementations
//! that mirror the pure functional OCaml reference implementation in `ocaml-indicators/`.
//!
//! # Available Indicators
//!
//! - **Moving Averages**: SMA, EMA
//! - **Momentum**: RSI, MACD
//! - **Volatility**: Bollinger Bands
//!
//! # Examples
//!
//! ```
//! use trading_engine::indicators::*;
//!
//! let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
//! let sma = simple_moving_average(&prices, 3);
//! assert_eq!(sma, vec![2.0, 3.0, 4.0]);
//! ```

/// OCaml indicator bridge (for verification/testing)
pub mod ocaml;

/// Calculate Simple Moving Average (SMA)
///
/// Returns a vector of averages for each window of size `period`.
/// Output length = `data.len() - period + 1`
///
/// # Arguments
///
/// * `data` - Price data
/// * `period` - Window size for averaging
///
/// # Returns
///
/// Vector of SMA values
///
/// # Examples
///
/// ```
/// use trading_engine::indicators::simple_moving_average;
///
/// let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let sma = simple_moving_average(&prices, 3);
/// assert_eq!(sma, vec![2.0, 3.0, 4.0]);
/// ```
pub fn simple_moving_average(data: &[f64], period: usize) -> Vec<f64> {
    if period == 0 || period > data.len() {
        return vec![];
    }

    data.windows(period)
        .map(|window| window.iter().sum::<f64>() / period as f64)
        .collect()
}

/// Calculate Exponential Moving Average (EMA)
///
/// Uses smoothing factor: alpha = 2 / (period + 1)
/// Returns a vector of the same length as input data.
///
/// # Arguments
///
/// * `data` - Price data
/// * `period` - Period for EMA calculation
///
/// # Examples
///
/// ```
/// use trading_engine::indicators::exponential_moving_average;
///
/// let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let ema = exponential_moving_average(&prices, 3);
/// assert_eq!(ema.len(), 5);
/// ```
pub fn exponential_moving_average(data: &[f64], period: usize) -> Vec<f64> {
    if period == 0 || period > data.len() {
        return vec![];
    }

    let alpha = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::with_capacity(data.len());

    // Initialize with SMA of first 'period' elements
    let seed: f64 = data[..period].iter().sum::<f64>() / period as f64;

    // Fill warmup period with seed
    for _ in 0..period {
        result.push(seed);
    }

    // Calculate EMA for remaining elements
    for &price in &data[period..] {
        let prev = *result.last().unwrap();
        result.push(alpha * price + (1.0 - alpha) * prev);
    }

    result
}

/// Calculate Relative Strength Index (RSI)
///
/// Returns RSI values in range 0.0-100.0.
/// First `period` values will be 50.0 (neutral) as warmup.
///
/// # Arguments
///
/// * `data` - Price data
/// * `period` - Period for RSI calculation (typically 14)
///
/// # Examples
///
/// ```
/// use trading_engine::indicators::relative_strength_index;
///
/// let prices = vec![44.0, 44.5, 45.0, 45.5, 46.0, 46.5, 47.0,
///                   46.5, 46.0, 45.5, 45.0, 44.5, 44.0, 43.5];
/// let rsi = relative_strength_index(&prices, 6);
/// assert_eq!(rsi.len(), 14);
/// ```
pub fn relative_strength_index(data: &[f64], period: usize) -> Vec<f64> {
    if period == 0 || period >= data.len() {
        return vec![];
    }

    let mut result = vec![50.0; data.len()]; // Neutral RSI during warmup

    // Calculate price changes
    let changes: Vec<f64> = data.windows(2).map(|w| w[1] - w[0]).collect();

    // Separate gains and losses
    let gains: Vec<f64> = changes.iter().map(|&x| if x > 0.0 { x } else { 0.0 }).collect();
    let losses: Vec<f64> = changes.iter().map(|&x| if x < 0.0 { x.abs() } else { 0.0 }).collect();

    // Calculate initial averages
    let mut avg_gain = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss = losses[..period].iter().sum::<f64>() / period as f64;

    // Calculate RSI using smoothed averages
    for i in period..data.len() {
        let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
        result[i] = 100.0 - (100.0 / (1.0 + rs));

        // Update smoothed averages for next iteration
        if i < data.len() - 1 {
            avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
            avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;
        }
    }

    result
}

/// MACD result containing the three components
pub struct MacdResult {
    pub macd_line: Vec<f64>,
    pub signal_line: Vec<f64>,
    pub histogram: Vec<f64>,
}

/// Calculate MACD (Moving Average Convergence Divergence)
///
/// Returns MACD line, signal line, and histogram.
///
/// # Arguments
///
/// * `data` - Price data
/// * `fast_period` - Fast EMA period (typically 12)
/// * `slow_period` - Slow EMA period (typically 26)
/// * `signal_period` - Signal line EMA period (typically 9)
///
/// # Examples
///
/// ```
/// use trading_engine::indicators::macd;
///
/// let prices: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
/// let result = macd(&prices, 12, 26, 9);
/// assert_eq!(result.macd_line.len(), 50);
/// ```
pub fn macd(data: &[f64], fast_period: usize, slow_period: usize, signal_period: usize) -> MacdResult {
    let fast_ema = exponential_moving_average(data, fast_period);
    let slow_ema = exponential_moving_average(data, slow_period);

    // MACD Line = Fast EMA - Slow EMA
    let macd_line: Vec<f64> = fast_ema.iter()
        .zip(slow_ema.iter())
        .map(|(f, s)| f - s)
        .collect();

    // Signal Line = EMA of MACD Line
    let signal_line = exponential_moving_average(&macd_line, signal_period);

    // Histogram = MACD Line - Signal Line
    let histogram: Vec<f64> = macd_line.iter()
        .zip(signal_line.iter())
        .map(|(m, s)| m - s)
        .collect();

    MacdResult {
        macd_line,
        signal_line,
        histogram,
    }
}

/// Bollinger Bands result
pub struct BollingerBands {
    pub upper: Vec<f64>,
    pub middle: Vec<f64>,
    pub lower: Vec<f64>,
}

/// Calculate Bollinger Bands
///
/// Returns upper band, middle band (SMA), and lower band.
///
/// # Arguments
///
/// * `data` - Price data
/// * `period` - Period for SMA and std dev calculation
/// * `num_std_dev` - Number of standard deviations for bands (typically 2.0)
///
/// # Examples
///
/// ```
/// use trading_engine::indicators::bollinger_bands;
///
/// let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0,
///                   105.0, 106.0, 107.0, 108.0, 109.0];
/// let bb = bollinger_bands(&prices, 5, 2.0);
/// assert_eq!(bb.middle.len(), 10);
/// ```
pub fn bollinger_bands(data: &[f64], period: usize, num_std_dev: f64) -> BollingerBands {
    let mut upper = vec![0.0; data.len()];
    let mut middle = vec![0.0; data.len()];
    let mut lower = vec![0.0; data.len()];

    if period == 0 || period > data.len() {
        return BollingerBands { upper, middle, lower };
    }

    // Fill warmup period with actual prices
    for i in 0..period-1 {
        middle[i] = data[i];
        upper[i] = data[i];
        lower[i] = data[i];
    }

    // Calculate Bollinger Bands for valid windows
    for (i, window) in data.windows(period).enumerate() {
        let idx = i + period - 1;
        let mean = window.iter().sum::<f64>() / period as f64;
        let variance = window.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / period as f64;
        let std = variance.sqrt();

        middle[idx] = mean;
        upper[idx] = mean + num_std_dev * std;
        lower[idx] = mean - num_std_dev * std;
    }

    BollingerBands { upper, middle, lower }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_float_eq(a: f64, b: f64, epsilon: f64) {
        assert!((a - b).abs() < epsilon, "Expected {}, got {}", b, a);
    }

    #[test]
    fn test_sma() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = simple_moving_average(&data, 3);
        assert_eq!(result.len(), 3);
        assert_float_eq(result[0], 2.0, 0.001);
        assert_float_eq(result[1], 3.0, 0.001);
        assert_float_eq(result[2], 4.0, 0.001);
    }

    #[test]
    fn test_ema() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = exponential_moving_average(&data, 3);
        assert_eq!(result.len(), 10);
        // First elements should be seed (SMA of first 3)
        assert_float_eq(result[0], 2.0, 0.001);
        assert_float_eq(result[1], 2.0, 0.001);
        assert_float_eq(result[2], 2.0, 0.001);
        // EMA should increase for monotonic data
        assert!(result[9] > result[3]);
    }

    #[test]
    fn test_rsi() {
        let data = vec![
            44.0, 44.5, 45.0, 45.5, 46.0, 46.5, 47.0,
            46.5, 46.0, 45.5, 45.0, 44.5, 44.0, 43.5,
        ];
        let result = relative_strength_index(&data, 6);
        assert_eq!(result.len(), 14);
        // During warmup, RSI should be neutral
        for i in 0..6 {
            assert_float_eq(result[i], 50.0, 0.001);
        }
        // After uptrend, RSI should be > 50
        assert!(result[6] > 50.0);
        // After downtrend, RSI should be < 50
        assert!(result[13] < 50.0);
    }

    #[test]
    fn test_macd() {
        let data: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let result = macd(&data, 12, 26, 9);
        assert_eq!(result.macd_line.len(), 50);
        assert_eq!(result.signal_line.len(), 50);
        assert_eq!(result.histogram.len(), 50);
        // For uptrending data, MACD should be positive
        assert!(result.macd_line[49] > 0.0);
    }

    #[test]
    fn test_bollinger_bands() {
        let data = vec![
            100.0, 101.0, 102.0, 103.0, 104.0,
            105.0, 106.0, 107.0, 108.0, 109.0,
        ];
        let result = bollinger_bands(&data, 5, 2.0);
        assert_eq!(result.middle.len(), 10);
        // Upper should be above middle, lower should be below
        for i in 4..10 {
            assert!(result.upper[i] > result.middle[i]);
            assert!(result.lower[i] < result.middle[i]);
        }
    }
}
