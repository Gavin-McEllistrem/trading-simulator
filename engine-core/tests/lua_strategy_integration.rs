use trading_engine::{
    market_data::{MarketData, MarketDataWindow},
    state_machine::Context,
    strategy::{IndicatorApi, LuaStrategy},
};

#[test]
fn test_strategy_loading() {
    let result = LuaStrategy::new("../lua-strategies/test_strategy.lua");
    assert!(result.is_ok(), "Failed to load strategy: {:?}", result.err());

    let strategy = result.unwrap();
    assert_eq!(strategy.name(), "test_strategy");
}

#[test]
fn test_strategy_missing_file() {
    let result = LuaStrategy::new("../lua-strategies/nonexistent.lua");
    assert!(result.is_err());
}

#[test]
fn test_detect_opportunity() {
    let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load strategy");

    // Create market data with 20 data points for indicator calculation
    let mut window = MarketDataWindow::new(50);
    for i in 0..20 {
        window.push(MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1000 + i as i64,
            open: 50000.0 + i as f64 * 100.0,
            high: 50100.0 + i as f64 * 100.0,
            low: 49900.0 + i as f64 * 100.0,
            close: 50000.0 + i as f64 * 100.0,
            volume: 1000,
            bid: 49950.0,
            ask: 50050.0,
        });
    }

    let market_data = window.latest().unwrap().clone();
    let context = Context::new();
    let indicator_api = IndicatorApi::new(window);

    let result = strategy.detect_opportunity(&market_data, &context, &indicator_api);
    assert!(result.is_ok());
}

#[test]
fn test_filter_commitment() {
    let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load strategy");

    let mut window = MarketDataWindow::new(50);
    for i in 0..20 {
        window.push(MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1000 + i as i64,
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50000.0,
            volume: 1000,
            bid: 49950.0,
            ask: 50050.0,
        });
    }

    let market_data = window.latest().unwrap().clone();
    let mut context = Context::new();
    context.set("signal", "bullish".to_string());

    let indicator_api = IndicatorApi::new(window);

    let result = strategy.filter_commitment(&market_data, &context, &indicator_api);
    assert!(result.is_ok());

    let action = result.unwrap();
    assert!(action.is_some(), "Expected an action to be returned");
}

#[test]
fn test_manage_position() {
    let strategy = LuaStrategy::new("../lua-strategies/test_strategy.lua")
        .expect("Failed to load strategy");

    let mut window = MarketDataWindow::new(50);
    for i in 0..20 {
        window.push(MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1000 + i as i64,
            open: 44000.0,  // Below exit threshold
            high: 44100.0,
            low: 43900.0,
            close: 44000.0,
            volume: 1000,
            bid: 43950.0,
            ask: 44050.0,
        });
    }

    let market_data = window.latest().unwrap().clone();
    let context = Context::new();
    let indicator_api = IndicatorApi::new(window);

    let result = strategy.manage_position(&market_data, &context, &indicator_api);
    assert!(result.is_ok());

    // Should return exit action since price < 45000
    let action = result.unwrap();
    assert!(action.is_some(), "Expected exit action");
}

#[test]
fn test_ema_crossover_strategy_loads() {
    let result = LuaStrategy::new("../lua-strategies/examples/ema_crossover.lua");
    assert!(result.is_ok(), "Failed to load EMA crossover strategy");
}

#[test]
fn test_rsi_mean_reversion_strategy_loads() {
    let result = LuaStrategy::new("../lua-strategies/examples/rsi_mean_reversion.lua");
    assert!(result.is_ok(), "Failed to load RSI mean reversion strategy");
}

#[test]
fn test_range_breakout_strategy_loads() {
    let result = LuaStrategy::new("../lua-strategies/examples/range_breakout.lua");
    assert!(result.is_ok(), "Failed to load range breakout strategy");
}
