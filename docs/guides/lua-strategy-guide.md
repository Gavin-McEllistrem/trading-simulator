# Lua Strategy Development Guide

This guide covers everything you need to know about creating, testing, and deploying Lua-based trading strategies in the trading simulator.

## Table of Contents

1. [Introduction](#introduction)
2. [Strategy Basics](#strategy-basics)
3. [API Reference](#api-reference)
4. [Creating Your First Strategy](#creating-your-first-strategy)
5. [Testing Strategies](#testing-strategies)
6. [Example Strategies](#example-strategies)
7. [Best Practices](#best-practices)
8. [Debugging](#debugging)
9. [Performance Optimization](#performance-optimization)

---

## Introduction

Strategies in this trading system are written in Lua and control the trading logic. They analyze market data, technical indicators, and decide when to enter/exit positions. Strategies are completely separate from the state machine - they simply return actions that the state machine executes.

### Why Lua?

- **Simple syntax**: Easy to learn and write
- **Fast execution**: <1ms overhead per tick
- **Hot reloading**: Modify strategies without recompiling Rust
- **Sandboxed**: Safe isolation from core engine
- **Type-safe integration**: Full validation at Rust boundary

### Architecture

```
┌──────────────────────────┐
│   Your Lua Strategy      │  User-defined logic
│   (3 required functions) │
└────────────┬─────────────┘
             │ Returns Actions
             ↓
┌──────────────────────────┐
│   State Machine (Rust)   │  Executes actions
│   - Idle                 │
│   - Analyzing            │
│   - InPosition           │
└──────────────────────────┘
```

---

## Strategy Basics

Every strategy must implement **exactly 3 functions**:

### 1. `detect_opportunity(market_data, context, indicators)`

**Called in:** Idle state
**Purpose:** Scan for trading opportunities
**Returns:**
- `nil` - No opportunity found
- `table` - Opportunity details (will transition to Analyzing state)

**Example:**
```lua
function detect_opportunity(market_data, context, indicators)
    local ema_fast = indicators.ema(10)
    local ema_slow = indicators.ema(20)

    if ema_fast and ema_slow and ema_fast > ema_slow then
        return {
            signal = "bullish",
            ema_fast = ema_fast,
            ema_slow = ema_slow,
            confidence = 0.8
        }
    end

    return nil
end
```

### 2. `filter_commitment(market_data, context, indicators)`

**Called in:** Analyzing state
**Purpose:** Decide whether to enter a trade
**Returns:**
- `nil` - No action
- `action table` - Entry action or cancel

**Example:**
```lua
function filter_commitment(market_data, context, indicators)
    if context.signal == "bullish" then
        return {
            action = "enter_long",
            price = market_data.close,
            quantity = 0.1
        }
    end

    return {
        action = "cancel_analysis",
        reason = "Signal conditions not met"
    }
end
```

### 3. `manage_position(market_data, context, indicators)`

**Called in:** InPosition state
**Purpose:** Manage the active trade
**Returns:**
- `nil` - No action needed
- `action table` - Exit, update stop/profit, etc.

**Example:**
```lua
function manage_position(market_data, context, indicators)
    local rsi = indicators.rsi(14)

    -- Exit on overbought
    if rsi and rsi > 70 then
        return {
            action = "exit",
            price = market_data.close,
            reason = "RSI overbought"
        }
    end

    return nil
end
```

---

## API Reference

### Market Data

The `market_data` table contains the latest candlestick data:

```lua
market_data.symbol      -- String: "BTCUSDT"
market_data.timestamp   -- Integer: Unix timestamp
market_data.open        -- Float: Opening price
market_data.high        -- Float: High price
market_data.low         -- Float: Low price
market_data.close       -- Float: Closing price
market_data.volume      -- Integer: Volume
market_data.bid         -- Float: Current bid price
market_data.ask         -- Float: Current ask price
market_data.mid_price   -- Float: (bid + ask) / 2
```

### Context

The `context` table stores strategy state between calls. Any values you set in one function will be available in subsequent calls:

```lua
-- Set values
context.signal = "bullish"
context.entry_price = 50000.0
context.confidence = 0.85
context.should_exit = true

-- Read values
if context.signal == "bullish" then
    -- ...
end
```

**Supported types:**
- `number` (f64 in Rust)
- `string`
- `integer` (i64 in Rust)
- `boolean`

### Indicators

The `indicators` table provides technical indicator functions:

```lua
-- Moving averages
local sma = indicators.sma(period)  -- Simple Moving Average
local ema = indicators.ema(period)  -- Exponential Moving Average

-- Oscillators
local rsi = indicators.rsi(period)  -- Relative Strength Index (0-100)

-- Window queries
local high = indicators.high          -- Highest high in window
local low = indicators.low            -- Lowest low in window
local range = indicators.range        -- Price range (high - low)
local avg_vol = indicators.avg_volume -- Average volume
```

**Returns:** `number` or `nil` (if not enough data)

### Actions

Action tables must have an `action` field with one of these values:

#### Entry Actions
```lua
-- Enter long position
{
    action = "enter_long",
    price = market_data.close,
    quantity = 0.1
}

-- Enter short position
{
    action = "enter_short",
    price = market_data.close,
    quantity = 0.1
}
```

#### Exit Actions
```lua
-- Exit position
{
    action = "exit",
    price = market_data.close,
    reason = "Optional reason string"
}
```

#### Position Management
```lua
-- Update stop loss
{
    action = "update_stop_loss",
    new_stop = 49000.0
}

-- Update take profit
{
    action = "update_take_profit",
    new_target = 52000.0
}
```

#### Analysis Control
```lua
-- Start analyzing (transition from Idle to Analyzing)
{
    action = "start_analyzing",
    reason = "Signal detected"
}

-- Cancel analysis (return to Idle)
{
    action = "cancel_analysis",
    reason = "Conditions not met"
}
```

---

## Creating Your First Strategy

Let's create a simple EMA crossover strategy from scratch.

### Step 1: Create the File

Create `lua-strategies/my_first_strategy.lua`:

```lua
--[[
    My First Strategy

    Simple EMA crossover with basic risk management.
    - Enter long when fast EMA crosses above slow EMA
    - Exit when fast EMA crosses below slow EMA
    - 2% stop loss, 5% take profit
]]

-- Configuration
local FAST_EMA = 10
local SLOW_EMA = 20
local STOP_LOSS_PCT = 0.02
local TAKE_PROFIT_PCT = 0.05
local POSITION_SIZE = 0.1

function detect_opportunity(market_data, context, indicators)
    -- Calculate EMAs
    local ema_fast = indicators.ema(FAST_EMA)
    local ema_slow = indicators.ema(SLOW_EMA)

    -- Need both values
    if not ema_fast or not ema_slow then
        return nil
    end

    -- Store current values for next call
    local prev_fast = context.prev_ema_fast
    local prev_slow = context.prev_ema_slow

    context.prev_ema_fast = ema_fast
    context.prev_ema_slow = ema_slow

    -- Detect crossover (need previous values)
    if not prev_fast or not prev_slow then
        return nil
    end

    -- Bullish crossover: fast crosses above slow
    if prev_fast <= prev_slow and ema_fast > ema_slow then
        return {
            signal = "bullish_crossover",
            ema_fast = ema_fast,
            ema_slow = ema_slow
        }
    end

    return nil
end

function filter_commitment(market_data, context, indicators)
    -- Only enter on bullish crossover
    if context.signal ~= "bullish_crossover" then
        return {
            action = "cancel_analysis",
            reason = "No valid signal"
        }
    end

    -- Calculate entry parameters
    local entry_price = market_data.close
    local stop_loss = entry_price * (1 - STOP_LOSS_PCT)
    local take_profit = entry_price * (1 + TAKE_PROFIT_PCT)

    -- Store for position management
    context.entry_price = entry_price
    context.stop_loss = stop_loss
    context.take_profit = take_profit

    return {
        action = "enter_long",
        price = entry_price,
        quantity = POSITION_SIZE
    }
end

function manage_position(market_data, context, indicators)
    -- Get current EMAs
    local ema_fast = indicators.ema(FAST_EMA)
    local ema_slow = indicators.ema(SLOW_EMA)

    if not ema_fast or not ema_slow then
        return nil
    end

    -- Exit on bearish crossover
    if ema_fast < ema_slow then
        return {
            action = "exit",
            price = market_data.close,
            reason = "Bearish crossover"
        }
    end

    return nil
end
```

### Step 2: Test the Strategy

Create a test in `engine-core/tests/lua_strategy_integration.rs`:

```rust
#[test]
fn test_my_first_strategy_loads() {
    let result = LuaStrategy::new("../lua-strategies/my_first_strategy.lua");
    assert!(result.is_ok(), "Failed to load strategy: {:?}", result.err());
}
```

Run the test:
```bash
cd engine-core
cargo test test_my_first_strategy_loads
```

### Step 3: Use in Demo

Modify `examples/lua_strategy_demo.rs` to load your strategy:

```rust
let strategy = LuaStrategy::new("../lua-strategies/my_first_strategy.lua")?;
```

Run the demo:
```bash
cargo run --example lua_strategy_demo
```

---

## Testing Strategies

### Unit Testing (Lua)

You can test individual functions by creating a separate test file:

```lua
-- test_my_strategy.lua
dofile("lua-strategies/my_first_strategy.lua")

-- Mock data
local market_data = {
    close = 50000.0,
    high = 50100.0,
    low = 49900.0
}

local context = {
    signal = "bullish_crossover"
}

local indicators = {
    ema = function(period)
        if period == 10 then return 50050.0 end
        if period == 20 then return 49950.0 end
    end
}

-- Test
local result = detect_opportunity(market_data, context, indicators)
print("Opportunity:", result ~= nil)
```

### Integration Testing (Rust)

Add tests to `tests/lua_strategy_integration.rs`:

```rust
#[test]
fn test_strategy_with_real_data() {
    let strategy = LuaStrategy::new("../lua-strategies/my_first_strategy.lua")
        .expect("Failed to load");

    // Create market data window
    let mut window = MarketDataWindow::new(50);
    for i in 0..30 {
        window.push(create_test_data(50000.0 + i as f64 * 10.0));
    }

    let market_data = window.latest().unwrap().clone();
    let context = Context::new();
    let indicator_api = IndicatorApi::new(window);

    let result = strategy.detect_opportunity(&market_data, &context, &indicator_api);
    assert!(result.is_ok());
}
```

### Backtesting

Create a backtest runner:

```rust
// examples/backtest.rs
let mut results = Vec::new();

for data in historical_data {
    window.push(data.clone());

    let action = match state_machine.current_state() {
        State::Idle => strategy.detect_opportunity(...),
        State::Analyzing => strategy.filter_commitment(...),
        State::InPosition => strategy.manage_position(...),
    };

    if let Some(action) = action {
        state_machine.execute(action)?;
    }

    state_machine.update(&data);

    // Record results
    if let Some(pos) = state_machine.position() {
        results.push(pos.unrealized_pnl());
    }
}

// Analyze results
println!("Total P&L: ${:.2}", results.iter().sum::<f64>());
```

---

## Example Strategies

### 1. EMA Crossover

Location: `lua-strategies/examples/ema_crossover.lua`

**Strategy:**
- Fast EMA (10) crosses above Slow EMA (20) → Enter long
- Fast EMA crosses below Slow EMA → Exit
- 2% stop loss, 5% take profit
- Volume confirmation

**Use case:** Trending markets

### 2. RSI Mean Reversion

Location: `lua-strategies/examples/rsi_mean_reversion.lua`

**Strategy:**
- RSI < 30 (oversold) → Enter long
- RSI >= 50 (neutral) → Exit
- 3% stop loss, 4% take profit
- Price bounce confirmation

**Use case:** Range-bound markets

### 3. Range Breakout

Location: `lua-strategies/examples/range_breakout.lua`

**Strategy:**
- Track 20-bar range (high/low)
- Price breaks above range + volume spike → Enter long
- Price falls back into range → Exit (failed breakout)
- Stop at range low, target 2x range size

**Use case:** Volatility breakouts

---

## Best Practices

### 1. Always Check for nil

Indicators return `nil` when there's insufficient data:

```lua
-- ❌ Bad: Will error if nil
local ema = indicators.ema(20)
if ema > 50000 then
    -- ...
end

-- ✅ Good: Check first
local ema = indicators.ema(20)
if ema and ema > 50000 then
    -- ...
end
```

### 2. Use Configuration Constants

Define parameters at the top of your file:

```lua
-- ✅ Good: Easy to adjust
local FAST_PERIOD = 10
local SLOW_PERIOD = 20
local STOP_LOSS_PCT = 0.02

function detect_opportunity(...)
    local fast = indicators.ema(FAST_PERIOD)
    -- ...
end
```

### 3. Store State in Context

Use context to maintain state between calls:

```lua
-- ✅ Good: State persists
function detect_opportunity(market_data, context, indicators)
    context.prev_price = context.current_price
    context.current_price = market_data.close

    if context.prev_price and context.current_price > context.prev_price then
        -- Price is rising
    end
end
```

### 4. Validate Signals Before Entry

Always double-check conditions in `filter_commitment`:

```lua
function filter_commitment(market_data, context, indicators)
    -- Recheck signal (market may have changed)
    local ema_fast = indicators.ema(10)
    local ema_slow = indicators.ema(20)

    if not ema_fast or not ema_slow or ema_fast <= ema_slow then
        return {
            action = "cancel_analysis",
            reason = "Signal no longer valid"
        }
    end

    -- Proceed with entry
end
```

### 5. Document Your Strategy

Add clear comments explaining the logic:

```lua
--[[
    Strategy: EMA Crossover with Volume Confirmation

    Entry Conditions:
    1. Fast EMA (10) crosses above Slow EMA (20)
    2. Volume > 150% of average volume
    3. Price not near recent high (avoid late entries)

    Exit Conditions:
    1. Fast EMA crosses below Slow EMA
    2. Stop loss hit (2%)
    3. Take profit hit (5%)

    Risk Management:
    - Position size: 10% of capital
    - Max loss per trade: 2%
    - Risk/Reward: 1:2.5
]]
```

### 6. Handle Edge Cases

Anticipate unusual market conditions:

```lua
function detect_opportunity(market_data, context, indicators)
    -- Check for stale data
    if market_data.volume == 0 then
        return nil  -- No trading on zero volume
    end

    -- Check for extreme moves (possible data error)
    if context.prev_price then
        local change = math.abs(market_data.close - context.prev_price) / context.prev_price
        if change > 0.10 then  -- 10% move
            return nil  -- Skip suspicious data
        end
    end

    -- Normal logic...
end
```

---

## Debugging

### 1. Print Debugging (Simple)

Lua `print()` statements will show up in the Rust output:

```lua
function detect_opportunity(market_data, context, indicators)
    local ema = indicators.ema(20)
    print(string.format("EMA(20): %s", tostring(ema)))

    if ema then
        print(string.format("Price: %.2f, EMA: %.2f", market_data.close, ema))
    end
end
```

### 2. Context Inspection

Store debug info in context:

```lua
function detect_opportunity(market_data, context, indicators)
    context.debug_calls = (context.debug_calls or 0) + 1
    context.last_ema = indicators.ema(20)

    if context.debug_calls % 10 == 0 then
        print(string.format("Called %d times, Last EMA: %s",
            context.debug_calls, tostring(context.last_ema)))
    end
end
```

### 3. Return Value Inspection

Check what your functions return:

```lua
function detect_opportunity(market_data, context, indicators)
    local result = {
        signal = "bullish",
        confidence = 0.8
    }

    print("Returning opportunity:", result.signal, result.confidence)
    return result
end
```

### 4. Common Errors

**Error: "Missing required function: XXX"**
- **Cause:** Function name misspelled or missing
- **Fix:** Ensure exact names: `detect_opportunity`, `filter_commitment`, `manage_position`

**Error: "Strategy error: action field must be a string"**
- **Cause:** Action table missing `action` field
- **Fix:** Always include `action = "enter_long"` etc.

**Error: "Lua error: attempt to compare nil with number"**
- **Cause:** Indicator returned nil, not checked
- **Fix:** Add nil checks: `if ema and ema > 50000 then`

---

## Performance Optimization

### 1. Minimize Indicator Calculations

Cache results when possible:

```lua
-- ❌ Bad: Calculates twice
if indicators.ema(20) > 50000 and indicators.ema(20) < 51000 then
    -- ...
end

-- ✅ Good: Calculate once
local ema = indicators.ema(20)
if ema and ema > 50000 and ema < 51000 then
    -- ...
end
```

### 2. Early Returns

Exit functions early when conditions aren't met:

```lua
function detect_opportunity(market_data, context, indicators)
    -- Check volume first (cheapest check)
    if market_data.volume < 1000 then
        return nil
    end

    -- Then check indicators (more expensive)
    local ema = indicators.ema(20)
    if not ema then
        return nil
    end

    -- Heavy logic only if basic checks pass
    -- ...
end
```

### 3. Avoid Complex Logic in manage_position

This function is called on every tick when in a position:

```lua
-- ❌ Bad: Heavy calculation every tick
function manage_position(market_data, context, indicators)
    -- Recalculate complex trailing stop every tick
    local trailing_stop = calculate_complex_trailing_stop(...)
end

-- ✅ Good: Update only when needed
function manage_position(market_data, context, indicators)
    -- Only update trailing stop on new highs
    if market_data.high > (context.highest_high or 0) then
        context.highest_high = market_data.high
        return {
            action = "update_stop_loss",
            new_stop = market_data.high * 0.98
        }
    end
    return nil
end
```

### 4. Performance Targets

Your strategy should aim for:
- `detect_opportunity`: <0.1ms per call
- `filter_commitment`: <0.2ms per call
- `manage_position`: <0.05ms per call

Monitor with:
```rust
let start = std::time::Instant::now();
let result = strategy.detect_opportunity(...);
println!("Took: {:?}", start.elapsed());
```

---

## Advanced Topics

### Multiple Timeframes

Track different EMA periods:

```lua
function detect_opportunity(market_data, context, indicators)
    -- Short-term: 5-minute trend
    local ema_short = indicators.ema(5)

    -- Medium-term: 20-minute trend
    local ema_medium = indicators.ema(20)

    -- Long-term: 50-minute trend
    local ema_long = indicators.ema(50)

    -- All aligned bullish?
    if ema_short and ema_medium and ema_long then
        if ema_short > ema_medium and ema_medium > ema_long then
            return { signal = "strong_bullish" }
        end
    end
end
```

### Dynamic Position Sizing

Adjust position size based on confidence:

```lua
function filter_commitment(market_data, context, indicators)
    local confidence = context.confidence or 0.5

    -- Base size: 10%
    -- Scale by confidence (0.5 - 1.0 → 5% - 10%)
    local position_size = 0.10 * confidence

    return {
        action = "enter_long",
        price = market_data.close,
        quantity = position_size
    }
end
```

### Trailing Stops

Implement a trailing stop loss:

```lua
function manage_position(market_data, context, indicators)
    local current_high = context.highest_since_entry or 0

    -- Update highest high
    if market_data.high > current_high then
        context.highest_since_entry = market_data.high

        -- Trail stop 2% below highest high
        local new_stop = market_data.high * 0.98

        return {
            action = "update_stop_loss",
            new_stop = new_stop
        }
    end

    return nil
end
```

---

## Next Steps

1. **Study Examples:** Read through the 3 example strategies in `lua-strategies/examples/`
2. **Create Your Own:** Start with a simple strategy and iterate
3. **Test Thoroughly:** Use integration tests and backtesting
4. **Optimize:** Profile and improve performance
5. **Deploy:** Integrate with SymbolRunner (Phase 5) for live trading

## Resources

- [Lua 5.4 Reference Manual](https://www.lua.org/manual/5.4/)
- [Example Strategies](../../lua-strategies/examples/)
- [API Documentation](../../engine-core/target/doc/trading_engine/strategy/index.html)
- [Phase 4 Completion Report](../../changes/2025-12-18-phase4-completion.md)

---

**Happy Trading! 🚀**
