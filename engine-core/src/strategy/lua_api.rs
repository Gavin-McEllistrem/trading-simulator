//! Lua API for strategy scripts
//!
//! This module provides the bridge between Rust and Lua, converting
//! Rust types to Lua tables and vice versa.

use crate::error::{Result, TradingEngineError};
use crate::market_data::{MarketData, MarketDataWindow};
use crate::state_machine::{Action, Context};
use mlua::{Lua, Table, Value};

/// API for accessing indicators from Lua
///
/// This struct wraps a MarketDataWindow and provides methods
/// that can be called from Lua scripts to calculate indicators.
pub struct IndicatorApi {
    window: MarketDataWindow,
}

impl IndicatorApi {
    /// Create a new indicator API from a market data window
    pub fn new(window: MarketDataWindow) -> Self {
        Self { window }
    }

    /// Get the close prices from the window
    pub fn closes(&self) -> Vec<f64> {
        let len = self.window.len();
        self.window.closes(len)
    }

    /// Calculate SMA
    pub fn sma(&self, period: usize) -> Option<f64> {
        let closes = self.closes();
        if closes.len() < period {
            return None;
        }
        crate::indicators::simple_moving_average(&closes, period)
            .last()
            .copied()
    }

    /// Calculate EMA
    pub fn ema(&self, period: usize) -> Option<f64> {
        let closes = self.closes();
        if closes.len() < period {
            return None;
        }
        crate::indicators::exponential_moving_average(&closes, period)
            .last()
            .copied()
    }

    /// Calculate RSI
    pub fn rsi(&self, period: usize) -> Option<f64> {
        let closes = self.closes();
        if closes.len() < period + 1 {
            return None;
        }
        crate::indicators::relative_strength_index(&closes, period)
            .last()
            .copied()
    }

    /// Get the highest high over the full window
    pub fn high(&self) -> Option<f64> {
        let len = self.window.len();
        self.window.high(len)
    }

    /// Get the lowest low over the full window
    pub fn low(&self) -> Option<f64> {
        let len = self.window.len();
        self.window.low(len)
    }

    /// Get the price range (high - low)
    pub fn range(&self) -> Option<f64> {
        let high = self.high()?;
        let low = self.low()?;
        Some(high - low)
    }

    /// Get the average volume over the full window
    pub fn avg_volume(&self) -> Option<f64> {
        let len = self.window.len();
        self.window.avg_volume(len)
    }
}

/// Convert MarketData to a Lua table
pub fn market_data_to_lua<'lua>(lua: &'lua Lua, data: &MarketData) -> Result<Table<'lua>> {
    let table = lua.create_table()?;
    table.set("symbol", data.symbol.clone())?;
    table.set("timestamp", data.timestamp)?;
    table.set("open", data.open)?;
    table.set("high", data.high)?;
    table.set("low", data.low)?;
    table.set("close", data.close)?;
    table.set("volume", data.volume)?;
    table.set("bid", data.bid)?;
    table.set("ask", data.ask)?;
    table.set("mid_price", data.mid_price())?;
    Ok(table)
}

/// Convert Context to a Lua table
pub fn context_to_lua<'lua>(lua: &'lua Lua, context: &Context) -> Result<Table<'lua>> {
    let table = lua.create_table()?;

    // Convert all context values to Lua
    // Numbers
    for (key, value) in context.iter_numbers() {
        table.set(key.clone(), *value)?;
    }

    // Strings
    for (key, value) in context.iter_strings() {
        table.set(key.clone(), value.clone())?;
    }

    // Integers
    for (key, value) in context.iter_integers() {
        table.set(key.clone(), *value)?;
    }

    // Booleans
    for (key, value) in context.iter_booleans() {
        table.set(key.clone(), *value)?;
    }

    Ok(table)
}

/// Convert IndicatorApi to a Lua table with callable functions
pub fn indicators_to_lua<'lua>(lua: &'lua Lua, api: &IndicatorApi) -> Result<Table<'lua>> {
    let table = lua.create_table()?;

    // Create closures for each indicator function
    let closes = api.closes();

    // SMA
    let sma_closes = closes.clone();
    let sma_fn = lua.create_function(move |_, period: usize| {
        if sma_closes.len() < period {
            return Ok(Value::Nil);
        }
        match crate::indicators::simple_moving_average(&sma_closes, period).last() {
            Some(&value) => Ok(Value::Number(value)),
            None => Ok(Value::Nil),
        }
    })?;
    table.set("sma", sma_fn)?;

    // EMA
    let ema_closes = closes.clone();
    let ema_fn = lua.create_function(move |_, period: usize| {
        if ema_closes.len() < period {
            return Ok(Value::Nil);
        }
        match crate::indicators::exponential_moving_average(&ema_closes, period).last() {
            Some(&value) => Ok(Value::Number(value)),
            None => Ok(Value::Nil),
        }
    })?;
    table.set("ema", ema_fn)?;

    // RSI
    let rsi_closes = closes.clone();
    let rsi_fn = lua.create_function(move |_, period: usize| {
        if rsi_closes.len() < period + 1 {
            return Ok(Value::Nil);
        }
        match crate::indicators::relative_strength_index(&rsi_closes, period).last() {
            Some(&value) => Ok(Value::Number(value)),
            None => Ok(Value::Nil),
        }
    })?;
    table.set("rsi", rsi_fn)?;

    // Window query functions
    table.set("high", api.high().unwrap_or(0.0))?;
    table.set("low", api.low().unwrap_or(0.0))?;
    table.set("range", api.range().unwrap_or(0.0))?;
    table.set("avg_volume", api.avg_volume().unwrap_or(0.0))?;

    Ok(table)
}

/// Convert a Lua table to an Action
pub fn table_to_action(table: &Table) -> Result<Option<Action>> {
    let action_type: String = match table.get("action")? {
        Value::String(s) => s.to_str()?.to_string(),
        Value::Nil => return Ok(None),
        _ => {
            return Err(TradingEngineError::StrategyError(
                "action field must be a string".to_string(),
            ))
        }
    };

    match action_type.as_str() {
        "enter_long" => {
            let price: f64 = table.get("price")?;
            let quantity: f64 = table.get("quantity")?;
            Ok(Some(Action::EnterLong { price, quantity }))
        }
        "enter_short" => {
            let price: f64 = table.get("price")?;
            let quantity: f64 = table.get("quantity")?;
            Ok(Some(Action::EnterShort { price, quantity }))
        }
        "exit" => {
            let price: f64 = table.get("price")?;
            Ok(Some(Action::ExitPosition { price }))
        }
        "update_stop_loss" => {
            let new_stop: f64 = table.get("new_stop")?;
            Ok(Some(Action::UpdateStopLoss { new_stop }))
        }
        "update_take_profit" => {
            let new_target: f64 = table.get("new_target")?;
            Ok(Some(Action::UpdateTakeProfit { new_target }))
        }
        "start_analyzing" => {
            let reason: String = table
                .get::<_, Option<String>>("reason")?
                .unwrap_or_else(|| "Strategy signal".to_string());
            Ok(Some(Action::StartAnalyzing { reason }))
        }
        "cancel_analysis" => {
            let reason: String = table
                .get::<_, Option<String>>("reason")?
                .unwrap_or_else(|| "Conditions not met".to_string());
            Ok(Some(Action::CancelAnalysis { reason }))
        }
        _ => Err(TradingEngineError::StrategyError(format!(
            "Unknown action type: {}",
            action_type
        ))),
    }
}

/// Update context from a Lua table
///
/// This allows Lua scripts to set context values that will be
/// preserved across strategy calls
pub fn update_context_from_lua(context: &mut Context, table: &Table) -> Result<()> {
    // Iterate through all keys in the table
    for pair in table.clone().pairs::<Value, Value>() {
        let (key, value) = pair?;

        if let Value::String(key_str) = key {
            let key_string = key_str.to_str()?.to_string();

            match value {
                Value::Number(n) => {
                    context.set(&key_string, n);
                }
                Value::String(s) => {
                    context.set(&key_string, s.to_str()?.to_string());
                }
                Value::Integer(i) => {
                    context.set(&key_string, i);
                }
                Value::Boolean(b) => {
                    context.set(&key_string, b);
                }
                _ => {
                    // Ignore other types (tables, functions, etc.)
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_data_to_lua() {
        let lua = Lua::new();
        let data = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1234567890,
            open: 50000.0,
            high: 51000.0,
            low: 49000.0,
            close: 50500.0,
            volume: 1000,
            bid: 50450.0,
            ask: 50550.0,
        };

        let table = market_data_to_lua(&lua, &data).unwrap();
        assert_eq!(table.get::<_, String>("symbol").unwrap(), "BTCUSDT");
        assert_eq!(table.get::<_, f64>("close").unwrap(), 50500.0);
        assert_eq!(table.get::<_, f64>("mid_price").unwrap(), 50500.0);
    }

    #[test]
    fn test_table_to_action_enter_long() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        table.set("action", "enter_long").unwrap();
        table.set("price", 50000.0).unwrap();
        table.set("quantity", 0.1).unwrap();

        let action = table_to_action(&table).unwrap();
        assert!(matches!(action, Some(Action::EnterLong { .. })));
    }

    #[test]
    fn test_table_to_action_exit() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        table.set("action", "exit").unwrap();
        table.set("price", 51000.0).unwrap();

        let action = table_to_action(&table).unwrap();
        assert!(matches!(action, Some(Action::ExitPosition { .. })));
    }

    #[test]
    fn test_context_to_lua() {
        let lua = Lua::new();
        let mut context = Context::new();
        context.set("confidence", 0.85);
        context.set("signal", "bullish".to_string());
        context.set("active", true);

        let table = context_to_lua(&lua, &context).unwrap();
        assert_eq!(table.get::<_, f64>("confidence").unwrap(), 0.85);
        assert_eq!(table.get::<_, String>("signal").unwrap(), "bullish");
        assert_eq!(table.get::<_, bool>("active").unwrap(), true);
    }
}
