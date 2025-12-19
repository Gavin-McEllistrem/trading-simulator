//! Strategy execution layer
//!
//! This module provides the Lua-based strategy system that drives the state machine.
//! Strategies are user-defined scripts that implement trading logic using market data,
//! technical indicators, and state machine context.
//!
//! # Architecture
//!
//! ```text
//! Lua Strategy Script
//!     ↓
//! LuaStrategy (Rust wrapper)
//!     ↓
//! StateMachine (executes actions)
//!     ↓
//! Position Management
//! ```
//!
//! # Example Strategy
//!
//! ```lua
//! -- strategies/ema_crossover.lua
//! function detect_opportunity(market_data, context, indicators)
//!     local ema_10 = indicators.ema(10)
//!     local ema_20 = indicators.ema(20)
//!
//!     if ema_10 > ema_20 then
//!         return { signal = "bullish", confidence = 0.8 }
//!     elseif ema_10 < ema_20 then
//!         return { signal = "bearish", confidence = 0.8 }
//!     end
//!     return nil
//! end
//!
//! function filter_commitment(market_data, context, indicators)
//!     if context.signal == "bullish" then
//!         return {
//!             action = "enter_long",
//!             price = market_data.close,
//!             quantity = 0.1,
//!             stop_loss = market_data.close * 0.98,
//!             take_profit = market_data.close * 1.05
//!         }
//!     end
//!     return nil
//! end
//!
//! function manage_position(market_data, context, indicators)
//!     -- Optional: trailing stop, position adjustments
//!     return nil
//! end
//! ```

use crate::error::Result;
use crate::market_data::MarketData;
use crate::state_machine::{Action, Context};
use mlua::{Lua, Table, Value};
use std::path::PathBuf;

mod lua_api;

pub use lua_api::IndicatorApi;

/// A Lua-based trading strategy
///
/// LuaStrategy loads and executes user-defined Lua scripts that implement
/// the three core strategy methods:
/// - `detect_opportunity`: Analyzes market conditions (Idle → Analyzing)
/// - `filter_commitment`: Decides on trade entry (Analyzing → InPosition)
/// - `manage_position`: Manages active trades (InPosition updates)
pub struct LuaStrategy {
    lua: Lua,
    script_path: PathBuf,
    strategy_name: String,
}

impl LuaStrategy {
    /// Create a new Lua strategy from a script file
    ///
    /// # Arguments
    ///
    /// * `script_path` - Path to the Lua strategy file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use trading_engine::strategy::LuaStrategy;
    ///
    /// let strategy = LuaStrategy::new("strategies/ema_crossover.lua")
    ///     .expect("Failed to load strategy");
    /// ```
    pub fn new(script_path: impl Into<PathBuf>) -> Result<Self> {
        let script_path = script_path.into();
        let strategy_name = script_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let lua = Lua::new();

        // Load the strategy script
        let script_content = std::fs::read_to_string(&script_path)?;
        lua.load(&script_content).exec()?;

        // Validate required functions exist
        Self::validate_strategy(&lua)?;

        Ok(Self {
            lua,
            script_path,
            strategy_name,
        })
    }

    /// Validate that the Lua script contains required functions
    fn validate_strategy(lua: &Lua) -> Result<()> {
        let globals = lua.globals();

        // Check for required functions
        let required_functions = [
            "detect_opportunity",
            "filter_commitment",
            "manage_position",
        ];

        for func_name in &required_functions {
            match globals.get::<_, Value>(*func_name)? {
                Value::Function(_) => {}
                _ => {
                    return Err(crate::error::TradingEngineError::StrategyError(
                        format!("Missing required function: {}", func_name),
                    ))
                }
            }
        }

        Ok(())
    }

    /// Get the strategy name
    pub fn name(&self) -> &str {
        &self.strategy_name
    }

    /// Get the script path
    pub fn script_path(&self) -> &PathBuf {
        &self.script_path
    }

    /// Call detect_opportunity function
    ///
    /// This is called in the Idle state to scan for trading opportunities.
    /// If the function returns a non-nil value, the state machine transitions
    /// to Analyzing state.
    pub fn detect_opportunity(
        &self,
        market_data: &MarketData,
        context: &Context,
        indicator_api: &IndicatorApi,
    ) -> Result<Option<Table>> {
        let globals = self.lua.globals();
        let func: mlua::Function = globals.get("detect_opportunity")?;

        // Convert inputs to Lua tables
        let market_table = lua_api::market_data_to_lua(&self.lua, market_data)?;
        let context_table = lua_api::context_to_lua(&self.lua, context)?;
        let indicator_table = lua_api::indicators_to_lua(&self.lua, indicator_api)?;

        // Call the function
        let result: Value = func.call((market_table, context_table, indicator_table))?;

        match result {
            Value::Nil => Ok(None),
            Value::Table(t) => Ok(Some(t)),
            _ => Err(crate::error::TradingEngineError::StrategyError(
                "detect_opportunity must return nil or a table".to_string(),
            )),
        }
    }

    /// Call filter_commitment function
    ///
    /// This is called in the Analyzing state to decide whether to enter a trade.
    /// The function should return nil or an action table describing the trade entry.
    pub fn filter_commitment(
        &self,
        market_data: &MarketData,
        context: &Context,
        indicator_api: &IndicatorApi,
    ) -> Result<Option<Action>> {
        let globals = self.lua.globals();
        let func: mlua::Function = globals.get("filter_commitment")?;

        let market_table = lua_api::market_data_to_lua(&self.lua, market_data)?;
        let context_table = lua_api::context_to_lua(&self.lua, context)?;
        let indicator_table = lua_api::indicators_to_lua(&self.lua, indicator_api)?;

        let result: Value = func.call((market_table, context_table, indicator_table))?;

        match result {
            Value::Nil => Ok(None),
            Value::Table(t) => lua_api::table_to_action(&t),
            _ => Err(crate::error::TradingEngineError::StrategyError(
                "filter_commitment must return nil or an action table".to_string(),
            )),
        }
    }

    /// Call manage_position function
    ///
    /// This is called in the InPosition state on every update to allow the strategy
    /// to manage the active position (trailing stops, partial exits, etc.)
    pub fn manage_position(
        &self,
        market_data: &MarketData,
        context: &Context,
        indicator_api: &IndicatorApi,
    ) -> Result<Option<Action>> {
        let globals = self.lua.globals();
        let func: mlua::Function = globals.get("manage_position")?;

        let market_table = lua_api::market_data_to_lua(&self.lua, market_data)?;
        let context_table = lua_api::context_to_lua(&self.lua, context)?;
        let indicator_table = lua_api::indicators_to_lua(&self.lua, indicator_api)?;

        let result: Value = func.call((market_table, context_table, indicator_table))?;

        match result {
            Value::Nil => Ok(None),
            Value::Table(t) => lua_api::table_to_action(&t),
            _ => Err(crate::error::TradingEngineError::StrategyError(
                "manage_position must return nil or an action table".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_validation_missing_function() {
        let lua = Lua::new();
        lua.load(
            r#"
            function detect_opportunity() end
            function filter_commitment() end
            -- missing manage_position
        "#,
        )
        .exec()
        .unwrap();

        let result = LuaStrategy::validate_strategy(&lua);
        assert!(result.is_err());
    }

    #[test]
    fn test_strategy_validation_success() {
        let lua = Lua::new();
        lua.load(
            r#"
            function detect_opportunity() end
            function filter_commitment() end
            function manage_position() end
        "#,
        )
        .exec()
        .unwrap();

        let result = LuaStrategy::validate_strategy(&lua);
        assert!(result.is_ok());
    }
}
