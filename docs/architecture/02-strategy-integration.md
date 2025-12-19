# Architecture: Strategy Integration

This document explains how the trading system architecture works and how Lua strategies integrate with the state machine and other components.

## Table of Contents

1. [Overview](#overview)
2. [Component Architecture](#component-architecture)
3. [Data Flow](#data-flow)
4. [Strategy Lifecycle](#strategy-lifecycle)
5. [State Machine Integration](#state-machine-integration)
6. [Type Conversion Layer](#type-conversion-layer)
7. [Error Handling](#error-handling)
8. [Performance Characteristics](#performance-characteristics)

---

## Overview

The trading system uses a **layered architecture** where each component has a specific responsibility. Strategies sit at the top of this stack and drive the trading logic, while lower layers handle data collection, analysis, and execution.

### Key Design Principles

1. **Separation of Concerns**: Each layer has a single responsibility
2. **Strategy Agnostic**: The state machine doesn't know about strategy implementation
3. **Type Safety**: All boundaries between Rust and Lua are validated
4. **Immutable Data Flow**: Market data flows down, actions flow up
5. **Zero Coupling**: Strategies can be swapped without changing core engine

---

## Component Architecture

### Layer Diagram

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 5: Lua Strategy Scripts (User Code)                  │
│  - detect_opportunity()                                      │
│  - filter_commitment()                                       │
│  - manage_position()                                         │
└────────────────────────┬────────────────────────────────────┘
                         │ Returns: Action tables
                         ↓
┌─────────────────────────────────────────────────────────────┐
│  Layer 4: LuaStrategy (Rust Wrapper)                        │
│  - VM management                                             │
│  - Type conversion (Rust ↔ Lua)                             │
│  - Function validation                                       │
│  - Error handling                                            │
└────────────────────────┬────────────────────────────────────┘
                         │ Returns: Action enum
                         ↓
┌─────────────────────────────────────────────────────────────┐
│  Layer 3: StateMachine (State Management)                   │
│  - Current state: Idle / Analyzing / InPosition             │
│  - Position tracking & P&L                                   │
│  - Action execution                                          │
│  - Auto-exit on stop/profit                                  │
│  - Transition history                                        │
└────────────────────────┬────────────────────────────────────┘
                         │ Queries for data
                         ↓
┌─────────────────────────────────────────────────────────────┐
│  Layer 2: Technical Indicators                              │
│  - Rust: SMA, EMA, RSI, MACD, Bollinger Bands              │
│  - OCaml: Verification implementation                        │
│  - Subprocess bridge (~1-2ms latency)                        │
└────────────────────────┬────────────────────────────────────┘
                         │ Calculates from
                         ↓
┌─────────────────────────────────────────────────────────────┐
│  Layer 1: Market Data Infrastructure                        │
│  - MarketDataWindow (circular buffer)                        │
│  - MarketDataStorage (thread-safe)                           │
│  - Data sources (Binance WebSocket, Simulated)              │
└─────────────────────────────────────────────────────────────┘
```

### Component Details

#### 1. Market Data Infrastructure (Layer 1)

**Purpose:** Collect and store real-time market data

**Components:**
- `MarketData`: OHLCV candlestick + bid/ask prices
- `MarketDataWindow`: Circular buffer (fixed size, O(1) insertion)
- `MarketDataStorage`: Thread-safe HashMap of windows per symbol
- `BinanceFeed`: WebSocket connection to Binance
- `SimulatedFeed`: Random walk generator for testing

**Key Methods:**
```rust
// MarketDataWindow
pub fn push(&mut self, data: MarketData)
pub fn latest(&self) -> Option<&MarketData>
pub fn closes(&self, period: usize) -> Vec<f64>
pub fn high(&self, period: usize) -> Option<f64>
pub fn low(&self, period: usize) -> Option<f64>
```

**Characteristics:**
- Bounded memory (configurable window size)
- No blocking operations
- Clone-friendly for multi-threading (Arc wrapper)

#### 2. Technical Indicators (Layer 2)

**Purpose:** Calculate technical indicators from market data

**Components:**
- Rust implementations (fast, production use)
- OCaml implementations (correctness verification)
- Subprocess bridge for OCaml communication

**Key Functions:**
```rust
pub fn simple_moving_average(data: &[f64], period: usize) -> Vec<f64>
pub fn exponential_moving_average(data: &[f64], period: usize) -> Vec<f64>
pub fn relative_strength_index(data: &[f64], period: usize) -> Vec<f64>
pub fn macd(data: &[f64], fast: usize, slow: usize, signal: usize) -> MacdResult
pub fn bollinger_bands(data: &[f64], period: usize, std_dev: f64) -> BollingerBands
```

**Characteristics:**
- Stateless (pure functions)
- Dual implementation for verification
- <1ms calculation time
- OCaml bridge adds 1-2ms (acceptable overhead)

#### 3. StateMachine (Layer 3)

**Purpose:** Manage trading states and positions

**States:**
```rust
pub enum State {
    Idle,        // Scanning for opportunities
    Analyzing,   // Evaluating potential entry
    InPosition,  // Actively trading
}
```

**Components:**
- `State`: Current state enum
- `Context`: Type-safe key-value storage for strategy state
- `Position`: Entry/exit tracking with P&L calculation
- `Action`: Enum representing trading actions
- `Transition`: History of state changes

**Key Methods:**
```rust
pub fn update(&mut self, market_data: &MarketData)
pub fn execute(&mut self, action: Action) -> Result<()>
pub fn transition_to(&mut self, new_state: State, reason: String)
pub fn current_state(&self) -> State
pub fn position(&self) -> Option<&Position>
pub fn context(&self) -> &Context
pub fn context_mut(&mut self) -> &mut Context
```

**Characteristics:**
- Generic (not strategy-specific)
- Auto-exit protection (checks stop/profit on every update)
- Bounded transition history (last 100)
- ~10-15KB memory per instance
- Microsecond-level state transitions

#### 4. LuaStrategy (Layer 4)

**Purpose:** Bridge between Lua scripts and Rust engine

**Components:**
```rust
pub struct LuaStrategy {
    lua: Lua,                    // Lua VM instance
    script_path: PathBuf,        // Path to .lua file
    strategy_name: String,       // Extracted from filename
}
```

**Key Methods:**
```rust
pub fn new(script_path: impl Into<PathBuf>) -> Result<Self>
pub fn detect_opportunity(&self, ...) -> Result<Option<Table>>
pub fn filter_commitment(&self, ...) -> Result<Option<Action>>
pub fn manage_position(&self, ...) -> Result<Option<Action>>
```

**Type Conversion:**
- `market_data_to_lua()`: MarketData → Lua table
- `context_to_lua()`: Context → Lua table
- `indicators_to_lua()`: IndicatorApi → Lua functions
- `table_to_action()`: Lua table → Action enum

**Characteristics:**
- One Lua VM per strategy instance
- Script validation at load time
- ~100-200KB memory per strategy
- <1ms overhead per function call

#### 5. Lua Strategy Scripts (Layer 5)

**Purpose:** User-defined trading logic

**Interface:**
```lua
function detect_opportunity(market_data, context, indicators)
    -- Returns: nil or opportunity table
end

function filter_commitment(market_data, context, indicators)
    -- Returns: nil or action table
end

function manage_position(market_data, context, indicators)
    -- Returns: nil or action table
end
```

**Characteristics:**
- Pure Lua (no Rust knowledge required)
- Sandboxed execution
- Hot-reloadable (Phase 5+)
- Zero coupling to engine internals

---

## Data Flow

### Flow Diagram

```
┌──────────────┐
│ Market Data  │ (Binance WebSocket or Simulated)
│   Source     │
└──────┬───────┘
       │ pushes
       ↓
┌──────────────────────┐
│ MarketDataWindow     │ (stores last N candles)
└──────┬───────────────┘
       │ reads
       ↓
┌──────────────────────┐
│ IndicatorApi         │ (calculates SMA, EMA, RSI, etc.)
└──────┬───────────────┘
       │ provides to
       ↓
┌──────────────────────┐
│ Lua Strategy         │ (user logic)
│ - analyze data       │
│ - generate action    │
└──────┬───────────────┘
       │ returns Action
       ↓
┌──────────────────────┐
│ StateMachine         │ (executes action)
│ - update state       │
│ - manage position    │
└──────┬───────────────┘
       │ auto-checks
       ↓
┌──────────────────────┐
│ Position             │ (check stop/profit)
└──────────────────────┘
```

### Detailed Flow by State

#### Idle State Flow

```
1. Market data arrives → pushed to window
2. StateMachine.update(market_data) called
3. StateMachine asks strategy: detect_opportunity()
4. Strategy receives:
   - market_data: Latest candle
   - context: Previous state
   - indicators: API with window data
5. Strategy analyzes and returns:
   - nil: No opportunity → stay in Idle
   - table: Opportunity found → transition to Analyzing
6. If table returned:
   - Context updated with opportunity data
   - State transitions: Idle → Analyzing
   - Transition recorded in history
```

#### Analyzing State Flow

```
1. New market data arrives
2. StateMachine asks strategy: filter_commitment()
3. Strategy receives same inputs as detect_opportunity
4. Strategy decides:
   - nil: Do nothing
   - { action = "cancel_analysis" }: Back to Idle
   - { action = "enter_long" }: Create position, go to InPosition
5. If entering position:
   - Position created with entry price, quantity
   - Stop loss and take profit set
   - Context updated
   - State transitions: Analyzing → InPosition
```

#### InPosition State Flow

```
1. New market data arrives
2. StateMachine.update() performs auto-checks:
   - Stop loss hit? → Auto-exit
   - Take profit hit? → Auto-exit
3. If still in position, ask strategy: manage_position()
4. Strategy can return:
   - nil: No action needed
   - { action = "exit" }: Close position
   - { action = "update_stop_loss" }: Adjust stop
   - { action = "update_take_profit" }: Adjust target
5. If position exits:
   - P&L calculated and recorded
   - State transitions: InPosition → Idle
```

---

## Strategy Lifecycle

### 1. Loading

```rust
// Load strategy from filesystem
let strategy = LuaStrategy::new("lua-strategies/my_strategy.lua")?;
```

**What happens:**
1. Read Lua file from disk
2. Create new Lua VM
3. Execute script (loads functions into VM)
4. Validate required functions exist:
   - `detect_opportunity`
   - `filter_commitment`
   - `manage_position`
5. Extract strategy name from filename
6. Return LuaStrategy instance

**Errors caught:**
- File not found
- Invalid Lua syntax
- Missing required functions
- Runtime errors during load

### 2. Execution

```rust
// Called by main loop
match state_machine.current_state() {
    State::Idle => {
        let opportunity = strategy.detect_opportunity(
            &market_data,
            state_machine.context(),
            &indicator_api
        )?;
        // Handle opportunity...
    }
    State::Analyzing => {
        let action = strategy.filter_commitment(
            &market_data,
            state_machine.context(),
            &indicator_api
        )?;
        // Handle action...
    }
    State::InPosition => {
        let action = strategy.manage_position(
            &market_data,
            state_machine.context(),
            &indicator_api
        )?;
        // Handle action...
    }
}
```

**What happens:**
1. Rust prepares inputs:
   - Convert `MarketData` → Lua table
   - Convert `Context` → Lua table
   - Create `IndicatorApi` with window clone
   - Convert `IndicatorApi` → Lua functions table
2. Call appropriate Lua function
3. Lua executes user logic
4. Lua returns result (nil or table)
5. Rust validates and converts result
6. Return to state machine

**Errors caught:**
- Lua runtime errors (nil access, type errors)
- Invalid return types
- Missing action field
- Unknown action types

### 3. State Updates

```rust
// Update context from strategy
if let Some(opp_table) = opportunity {
    // Extract values from Lua table
    if let Ok(signal) = opp_table.get::<_, String>("signal") {
        context.set("signal", signal);
    }
    if let Ok(confidence) = opp_table.get::<_, f64>("confidence") {
        context.set("confidence", confidence);
    }
}
```

**Context persistence:**
- Values set in `detect_opportunity` available in `filter_commitment`
- Values set in `filter_commitment` available in `manage_position`
- Context clears on position exit (optional behavior)

---

## State Machine Integration

### How Strategies Drive the State Machine

The state machine is **reactive** - it doesn't contain trading logic. Instead, it:
1. Maintains current state
2. Calls the strategy for decisions
3. Executes actions returned by strategy
4. Provides auto-exit protection

```rust
// Simplified state machine update loop
impl StateMachine {
    pub fn update(&mut self, market_data: &MarketData) {
        // Update context with latest data
        self.context.set("latest_price", market_data.close);

        // Auto-exit check (if in position)
        if let Some(position) = &self.position {
            if position.is_stop_loss_hit() || position.is_take_profit_hit() {
                self.exit_position(market_data.close);
                return;
            }
        }

        // Strategy would be called here by external code
        // (not part of StateMachine - separation of concerns)
    }

    pub fn execute(&mut self, action: Action) -> Result<()> {
        match action {
            Action::EnterLong { price, quantity } => {
                self.position = Some(Position::new_long(price, quantity));
                self.transition_to(State::InPosition, "Entered long".to_string());
            }
            Action::ExitPosition { price } => {
                if let Some(mut pos) = self.position.take() {
                    pos.close(price);
                    // P&L recorded, position cleared
                }
                self.transition_to(State::Idle, "Exited position".to_string());
            }
            // ... other actions
        }
        Ok(())
    }
}
```

### Why This Design?

**Advantages:**
1. **Testable**: State machine can be tested independently
2. **Reusable**: Same state machine for all strategies
3. **Safe**: Auto-exit provides baseline risk management
4. **Flexible**: Strategies can be swapped without changing engine
5. **Debuggable**: Clear boundary between logic and execution

**Trade-offs:**
- Extra layer of indirection (minimal overhead)
- Strategies must return Actions (can't manipulate state directly)
- Context is generic HashMap (not typed structs)

---

## Type Conversion Layer

### The Bridge Between Rust and Lua

All data crossing the Rust ↔ Lua boundary must be converted.

### Rust → Lua Conversions

#### MarketData → Lua Table

```rust
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
```

**Lua sees:**
```lua
market_data.symbol      -- "BTCUSDT"
market_data.timestamp   -- 1234567890
market_data.close       -- 50000.0
-- etc.
```

#### Context → Lua Table

```rust
pub fn context_to_lua<'lua>(lua: &'lua Lua, context: &Context) -> Result<Table<'lua>> {
    let table = lua.create_table()?;

    // Numbers (f64)
    for (key, value) in context.iter_numbers() {
        table.set(key.clone(), *value)?;
    }

    // Strings
    for (key, value) in context.iter_strings() {
        table.set(key.clone(), value.clone())?;
    }

    // Integers (i64)
    for (key, value) in context.iter_integers() {
        table.set(key.clone(), *value)?;
    }

    // Booleans
    for (key, value) in context.iter_booleans() {
        table.set(key.clone(), *value)?;
    }

    Ok(table)
}
```

**Lua sees:**
```lua
context.signal       -- "bullish" (string)
context.confidence   -- 0.85 (number)
context.entry_count  -- 5 (integer)
context.active       -- true (boolean)
```

#### IndicatorApi → Lua Functions

```rust
pub fn indicators_to_lua<'lua>(lua: &'lua Lua, api: &IndicatorApi) -> Result<Table<'lua>> {
    let table = lua.create_table()?;
    let closes = api.closes();

    // SMA function
    let sma_closes = closes.clone();
    let sma_fn = lua.create_function(move |_, period: usize| {
        if sma_closes.len() < period {
            return Ok(Value::Nil);
        }
        match simple_moving_average(&sma_closes, period).last() {
            Some(&value) => Ok(Value::Number(value)),
            None => Ok(Value::Nil),
        }
    })?;
    table.set("sma", sma_fn)?;

    // Similar for ema, rsi, etc.

    // Window queries (simple values)
    table.set("high", api.high().unwrap_or(0.0))?;
    table.set("low", api.low().unwrap_or(0.0))?;

    Ok(table)
}
```

**Lua sees:**
```lua
local sma = indicators.sma(20)  -- Calls Rust function
local high = indicators.high    -- Simple value
```

### Lua → Rust Conversions

#### Lua Table → Action

```rust
pub fn table_to_action(table: &Table) -> Result<Option<Action>> {
    let action_type: String = match table.get("action")? {
        Value::String(s) => s.to_str()?.to_string(),
        Value::Nil => return Ok(None),
        _ => return Err(TradingEngineError::StrategyError(
            "action field must be a string".to_string()
        )),
    };

    match action_type.as_str() {
        "enter_long" => {
            let price: f64 = table.get("price")?;
            let quantity: f64 = table.get("quantity")?;
            Ok(Some(Action::EnterLong { price, quantity }))
        }
        "exit" => {
            let price: f64 = table.get("price")?;
            Ok(Some(Action::ExitPosition { price }))
        }
        // ... other action types
        _ => Err(TradingEngineError::StrategyError(
            format!("Unknown action type: {}", action_type)
        )),
    }
}
```

**Validation happens here:**
- `action` field must be a string
- Required fields must exist (`price`, `quantity`)
- Field types must match (f64, not string)
- Action type must be recognized

---

## Error Handling

### Error Types

```rust
#[derive(Error, Debug)]
pub enum TradingEngineError {
    #[error("Strategy error: {0}")]
    StrategyError(String),

    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::Error),

    // ... other error types
}
```

### Error Propagation

```
Lua Runtime Error
    ↓
mlua::Error (caught by mlua)
    ↓
TradingEngineError::LuaError (converted)
    ↓
Returned to caller (Result type)
    ↓
Logged and handled gracefully
```

### Common Errors

**1. Missing Required Function**
```
Error: Strategy error: Missing required function: detect_opportunity
```
**Caught at:** Load time (before execution)

**2. Invalid Action Type**
```
Error: Strategy error: Unknown action type: enter_postion
```
**Caught at:** Conversion time (table_to_action)

**3. Lua Runtime Error**
```
Error: Lua error: attempt to compare nil with number
```
**Caught at:** Execution time (Lua VM)

**4. Type Mismatch**
```
Error: Lua error: error converting Lua nil to f64
```
**Caught at:** Conversion time (getting field from table)

---

## Performance Characteristics

### Memory Usage

**Per Strategy Instance:**
- Lua VM: ~100-200KB
- Script code: ~10-50KB
- Total: ~200-300KB per strategy

**Per State Machine:**
- State: 8 bytes (enum)
- Context: ~1-5KB (depends on data stored)
- Position: ~200 bytes
- Transition history: ~10KB (100 transitions)
- Total: ~10-15KB per state machine

**Per Symbol (Full Stack):**
- MarketDataWindow: ~50KB (50 candles)
- Indicators: negligible (calculated on-demand)
- Strategy: ~250KB
- StateMachine: ~15KB
- Total: ~315KB per symbol

**Scaling:**
- 100 symbols: ~31MB
- 1000 symbols: ~310MB

### Latency Breakdown

**Per Tick (1 market data update):**
```
Market data arrival:           0 ms (instant)
  ↓
Window insertion:              <0.001 ms (O(1) circular buffer)
  ↓
Indicator calculation:         0.1-1.0 ms (depends on period)
  ↓
Rust → Lua conversion:         0.01-0.05 ms (table creation)
  ↓
Lua function execution:        0.05-0.5 ms (user logic)
  ↓
Lua → Rust conversion:         0.01-0.05 ms (table parsing)
  ↓
Action execution:              0.001 ms (state update)
  ↓
Total: ~0.2-2.0 ms per tick
```

**Breakdown by Component:**
- Market data operations: <1%
- Indicator calculations: 50-70%
- Type conversions: 5-10%
- Lua execution: 20-40%
- State machine: <1%

**Bottlenecks (in order):**
1. Indicator calculations (especially MACD)
2. Lua script complexity
3. Type conversions
4. Everything else is negligible

### Throughput

**Single Symbol:**
- Updates per second: 500-1000 (limited by indicator calc)
- Actions per second: 1000+ (state machine can handle more)

**Multi-Symbol (Phase 5):**
- With threading: N symbols × 500-1000 updates/sec
- Limited by CPU cores and indicator overhead

---

## Future Integration (Phase 5+)

### SymbolRunner Architecture

```rust
// Future Phase 5 implementation
pub struct SymbolRunner {
    symbol: String,
    market_window: MarketDataWindow,
    state_machine: StateMachine,
    strategy: LuaStrategy,
    indicator_api: IndicatorApi,
}

impl SymbolRunner {
    pub async fn run(&mut self) {
        loop {
            // Receive market data from channel
            let market_data = self.receive_market_data().await;

            // Update window
            self.market_window.push(market_data.clone());

            // Update indicator API
            self.indicator_api = IndicatorApi::new(self.market_window.clone());

            // Call strategy based on state
            let action = match self.state_machine.current_state() {
                State::Idle => {
                    self.strategy.detect_opportunity(
                        &market_data,
                        self.state_machine.context(),
                        &self.indicator_api
                    )?
                    .map(|_| Action::StartAnalyzing { reason: "Signal".into() })
                }
                State::Analyzing => {
                    self.strategy.filter_commitment(
                        &market_data,
                        self.state_machine.context(),
                        &self.indicator_api
                    )?
                }
                State::InPosition => {
                    self.strategy.manage_position(
                        &market_data,
                        self.state_machine.context(),
                        &self.indicator_api
                    )?
                }
            };

            // Execute action
            if let Some(action) = action {
                self.state_machine.execute(action)?;
            }

            // Update state machine
            self.state_machine.update(&market_data);
        }
    }
}
```

### Multi-Symbol Orchestration

```
Main Thread
    ├── Data feed connection (WebSocket)
    ├── Broadcast market data to symbol channels
    └── Aggregate results
         ↓
    ┌────┴────┬────────┬────────┬─────────┐
    ↓         ↓        ↓        ↓         ↓
Symbol 1  Symbol 2  Symbol 3  ...   Symbol N
(Thread)  (Thread)  (Thread)       (Thread)
    │         │        │                  │
    ↓         ↓        ↓                  ↓
Runner    Runner    Runner             Runner
 (BTC)     (ETH)     (SOL)              (...)
```

Each SymbolRunner:
- Runs in own thread
- Has own StateMachine
- Has own MarketDataWindow
- Can use same or different strategy
- Completely independent

---

## Summary

The trading system architecture is designed with clear separation of concerns:

1. **Data Layer**: Collects and stores market data efficiently
2. **Analysis Layer**: Calculates technical indicators
3. **State Layer**: Manages trading states and positions
4. **Strategy Layer**: Implements trading logic (user-defined)

**Strategies fit in by:**
- Receiving market data, context, and indicators
- Returning actions to the state machine
- Never directly manipulating state
- Being completely swappable

**This design enables:**
- ✅ Testability (each layer independently tested)
- ✅ Reusability (same engine for all strategies)
- ✅ Safety (auto-exit protection built-in)
- ✅ Flexibility (swap strategies without changing engine)
- ✅ Performance (<2ms latency per tick)
- ✅ Scalability (100+ concurrent symbols in Phase 5)

The architecture is production-ready for Phase 5 deployment!
