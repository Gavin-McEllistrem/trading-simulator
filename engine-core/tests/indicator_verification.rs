//! Indicator Verification Tests
//!
//! These tests compare Rust and OCaml implementations of indicators
//! to ensure they produce identical results.

use trading_engine::indicators::*;
use trading_engine::indicators::ocaml::*;

fn assert_vec_eq(rust_result: &[f64], ocaml_result: &[f64], epsilon: f64) {
    assert_eq!(rust_result.len(), ocaml_result.len(),
        "Length mismatch: Rust={}, OCaml={}", rust_result.len(), ocaml_result.len());

    for (i, (r, o)) in rust_result.iter().zip(ocaml_result.iter()).enumerate() {
        assert!((r - o).abs() < epsilon,
            "Value mismatch at index {}: Rust={}, OCaml={}, diff={}",
            i, r, o, (r - o).abs());
    }
}

#[test]
fn verify_sma_matches_ocaml() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    let rust_result = simple_moving_average(&data, 3);
    let ocaml_result = sma_ocaml(&data, 3).unwrap();

    assert_vec_eq(&rust_result, &ocaml_result, 0.001);
}

#[test]
fn verify_ema_matches_ocaml() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    let rust_result = exponential_moving_average(&data, 3);
    let ocaml_result = ema_ocaml(&data, 3).unwrap();

    assert_vec_eq(&rust_result, &ocaml_result, 0.001);
}

#[test]
fn verify_rsi_matches_ocaml() {
    let data = vec![
        44.0, 44.5, 45.0, 45.5, 46.0, 46.5, 47.0,
        46.5, 46.0, 45.5, 45.0, 44.5, 44.0, 43.5,
    ];

    let rust_result = relative_strength_index(&data, 6);
    let ocaml_result = rsi_ocaml(&data, 6).unwrap();

    assert_vec_eq(&rust_result, &ocaml_result, 0.001);
}

#[test]
fn verify_macd_matches_ocaml() {
    let data: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();

    let rust_result = macd(&data, 12, 26, 9);
    let ocaml_result = macd_ocaml(&data, 12, 26, 9).unwrap();

    assert_vec_eq(&rust_result.macd_line, &ocaml_result.0, 0.001);
    assert_vec_eq(&rust_result.signal_line, &ocaml_result.1, 0.001);
    assert_vec_eq(&rust_result.histogram, &ocaml_result.2, 0.001);
}

#[test]
fn verify_bollinger_bands_matches_ocaml() {
    let data = vec![
        100.0, 101.0, 102.0, 103.0, 104.0,
        105.0, 106.0, 107.0, 108.0, 109.0,
    ];

    let rust_result = bollinger_bands(&data, 5, 2.0);
    let ocaml_result = bollinger_bands_ocaml(&data, 5, 2.0).unwrap();

    assert_vec_eq(&rust_result.upper, &ocaml_result.0, 0.001);
    assert_vec_eq(&rust_result.middle, &ocaml_result.1, 0.001);
    assert_vec_eq(&rust_result.lower, &ocaml_result.2, 0.001);
}

#[test]
fn verify_large_dataset() {
    // Test with larger dataset to ensure both implementations scale
    let data: Vec<f64> = (0..1000).map(|i| 100.0 + (i as f64 * 0.1)).collect();

    let rust_sma = simple_moving_average(&data, 50);
    let ocaml_sma = sma_ocaml(&data, 50).unwrap();

    assert_vec_eq(&rust_sma, &ocaml_sma, 0.001);
}
