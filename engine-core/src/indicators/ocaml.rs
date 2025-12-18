//! OCaml Indicator Bridge
//!
//! This module provides a bridge to call OCaml indicator implementations
//! via subprocess for verification and testing purposes.

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Command, Stdio};
use crate::Result;

/// Path to the OCaml indicators CLI binary
const OCAML_CLI_PATH: &str = "../ocaml-indicators/_build/default/bin/main.exe";

/// Request structure for OCaml indicator calculations
#[derive(Debug, Serialize)]
struct IndicatorRequest {
    indicator: String,
    data: Vec<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    period: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fast_period: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    slow_period: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signal_period: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_std_dev: Option<f64>,
}

/// SMA response from OCaml
#[derive(Debug, Deserialize)]
struct SmaResponse {
    values: Vec<f64>,
}

/// EMA response from OCaml
#[derive(Debug, Deserialize)]
struct EmaResponse {
    values: Vec<f64>,
}

/// RSI response from OCaml
#[derive(Debug, Deserialize)]
struct RsiResponse {
    values: Vec<f64>,
}

/// MACD response from OCaml
#[derive(Debug, Deserialize)]
struct MacdResponse {
    macd_line: Vec<f64>,
    signal_line: Vec<f64>,
    histogram: Vec<f64>,
}

/// Bollinger Bands response from OCaml
#[derive(Debug, Deserialize)]
struct BollingerBandsResponse {
    upper: Vec<f64>,
    middle: Vec<f64>,
    lower: Vec<f64>,
}

/// Error response from OCaml
#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
}

/// Call OCaml CLI and get JSON response
fn call_ocaml(request: &IndicatorRequest) -> Result<serde_json::Value> {
    let json_input = serde_json::to_string(request)?;

    let mut child = Command::new(OCAML_CLI_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| crate::TradingEngineError::InvalidData(
            format!("Failed to spawn OCaml process: {}. Make sure to build ocaml-indicators first (cd ../ocaml-indicators && dune build)", e)
        ))?;

    // Write request to stdin
    {
        let stdin = child.stdin.as_mut()
            .ok_or_else(|| crate::TradingEngineError::InvalidData("Failed to open stdin".to_string()))?;
        stdin.write_all(json_input.as_bytes())?;
    }

    // Read response
    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::TradingEngineError::InvalidData(
            format!("OCaml process failed: {}", stderr)
        ));
    }

    let response_str = String::from_utf8_lossy(&output.stdout);
    let response: serde_json::Value = serde_json::from_str(&response_str)?;

    // Check for error in response
    if let Some(error) = response.get("error") {
        return Err(crate::TradingEngineError::InvalidData(
            format!("OCaml indicator error: {}", error)
        ));
    }

    Ok(response)
}

/// Calculate SMA using OCaml implementation
pub fn sma_ocaml(data: &[f64], period: usize) -> Result<Vec<f64>> {
    let request = IndicatorRequest {
        indicator: "sma".to_string(),
        data: data.to_vec(),
        period: Some(period),
        fast_period: None,
        slow_period: None,
        signal_period: None,
        num_std_dev: None,
    };

    let response = call_ocaml(&request)?;
    let result: SmaResponse = serde_json::from_value(response)?;
    Ok(result.values)
}

/// Calculate EMA using OCaml implementation
pub fn ema_ocaml(data: &[f64], period: usize) -> Result<Vec<f64>> {
    let request = IndicatorRequest {
        indicator: "ema".to_string(),
        data: data.to_vec(),
        period: Some(period),
        fast_period: None,
        slow_period: None,
        signal_period: None,
        num_std_dev: None,
    };

    let response = call_ocaml(&request)?;
    let result: EmaResponse = serde_json::from_value(response)?;
    Ok(result.values)
}

/// Calculate RSI using OCaml implementation
pub fn rsi_ocaml(data: &[f64], period: usize) -> Result<Vec<f64>> {
    let request = IndicatorRequest {
        indicator: "rsi".to_string(),
        data: data.to_vec(),
        period: Some(period),
        fast_period: None,
        slow_period: None,
        signal_period: None,
        num_std_dev: None,
    };

    let response = call_ocaml(&request)?;
    let result: RsiResponse = serde_json::from_value(response)?;
    Ok(result.values)
}

/// Calculate MACD using OCaml implementation
pub fn macd_ocaml(data: &[f64], fast: usize, slow: usize, signal: usize)
    -> Result<(Vec<f64>, Vec<f64>, Vec<f64>)> {
    let request = IndicatorRequest {
        indicator: "macd".to_string(),
        data: data.to_vec(),
        period: None,
        fast_period: Some(fast),
        slow_period: Some(slow),
        signal_period: Some(signal),
        num_std_dev: None,
    };

    let response = call_ocaml(&request)?;
    let result: MacdResponse = serde_json::from_value(response)?;
    Ok((result.macd_line, result.signal_line, result.histogram))
}

/// Calculate Bollinger Bands using OCaml implementation
pub fn bollinger_bands_ocaml(data: &[f64], period: usize, num_std_dev: f64)
    -> Result<(Vec<f64>, Vec<f64>, Vec<f64>)> {
    let request = IndicatorRequest {
        indicator: "bollinger_bands".to_string(),
        data: data.to_vec(),
        period: Some(period),
        fast_period: None,
        slow_period: None,
        signal_period: None,
        num_std_dev: Some(num_std_dev),
    };

    let response = call_ocaml(&request)?;
    let result: BollingerBandsResponse = serde_json::from_value(response)?;
    Ok((result.upper, result.middle, result.lower))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_ocaml() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = sma_ocaml(&data, 3).unwrap();
        assert_eq!(result.len(), 3);
        assert!((result[0] - 2.0).abs() < 0.001);
        assert!((result[1] - 3.0).abs() < 0.001);
        assert!((result[2] - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_rsi_ocaml() {
        let data = vec![
            44.0, 44.5, 45.0, 45.5, 46.0, 46.5, 47.0,
            46.5, 46.0, 45.5, 45.0, 44.5, 44.0, 43.5,
        ];
        let result = rsi_ocaml(&data, 6).unwrap();
        assert_eq!(result.len(), 14);
        // Warmup period should be neutral
        for i in 0..6 {
            assert!((result[i] - 50.0).abs() < 0.001);
        }
        // After uptrend, RSI > 50
        assert!(result[6] > 50.0);
    }
}
