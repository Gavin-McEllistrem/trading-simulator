# Trading Simulator

A high-performance, multi-threaded trading system with Rust core engine, OCaml indicators, and Lua strategy scripting.

## Project Status

**Current Phase:** 6 (Web App Infrastructure) 🚧 **IN PROGRESS**

**Progress:** 5.67 of 12 phases complete (47% of core system, Phase 6: 40% complete)

### Completed (Phases 0-5)

**Phase 1: Market Data Infrastructure** ✅
- Core data structures (MarketData, MarketDataWindow)
- Data source abstraction (MarketDataSource trait)
- **Binance WebSocket integration** with real bid/ask prices
- Simulated feed for testing
- Thread-safe storage system
- 47 tests passing

**Phase 2: Technical Indicators** ✅
- **Dual Rust/OCaml implementation** (SMA, EMA, RSI, MACD, Bollinger Bands)
- OCaml subprocess bridge (1-2ms latency)
- Full verification suite (Rust ↔ OCaml match within 0.001)
- 48 tests passing

**Phase 3: State Machine Core** ✅
- 3-state FSM (Idle, Analyzing, InPosition)
- Position tracking with P&L calculation
- Auto-exit on stop loss/take profit
- Transition history tracking
- Generic, strategy-agnostic design
- 28 tests passing

**Phase 4: Lua Strategy Integration** ✅
- **LuaStrategy system** with VM management
- **3 production-ready example strategies:**
  - EMA Crossover (10/20 periods)
  - RSI Mean Reversion (oversold <30)
  - Range Breakout (20-bar range + volume)
- Full Lua API for market data, indicators, and actions
- Table-based strategy interface
- 14 tests passing

**Phase 5: Multi-Symbol Threading Engine** ✅
- **SymbolRunner** orchestration (~570 LOC)
  - Per-symbol async task with component coordination
  - Runs indefinitely until channel closed (not tick-limited)
- **TradingEngine** multi-runner management (~1,236 LOC)
  - **Runner-based architecture**: multiple strategies per symbol
  - Efficient broadcast to all runners watching a symbol
  - Dynamic runner add/remove
- **Health monitoring** & error recovery
  - Per-runner health checks and uptime tracking
  - Engine-wide health summary
- **28 tests** passing (17 unit + 11 integration)
- **Demo**: 6 concurrent runners (2 strategies × 3 symbols)

**Phase 6: Web App Infrastructure** 🚧 (Steps 1-4 Complete)
- **Event System** (~258 LOC)
  - 10 event types (TickReceived, StateTransition, PositionUpdated, etc.)
  - Real-time streaming from runners → engine → multiple subscribers
  - Event aggregation with automatic subscriber cleanup
  - JSON serialization for WebSocket transmission
  - 10 tests passing (event types, emission, aggregation)
- **State Introspection API** (~195 LOC) ✨ **NEW!**
  - Command channel for querying runner state on-demand
  - `get_runner_snapshot()` - Query current state, position, context, stats
  - `get_price_history()` - Query recent price data from window
  - RunnerSnapshot with full JSON serialization
  - Non-blocking queries with 100ms timeout
  - 7 new tests passing (snapshot creation, queries, error handling)
- **Architecture**
  - Push model: Real-time events via pub-sub
  - Pull model: On-demand snapshots via request-response
  - Combined: Complete observability for web dashboards

**Total:** 98 tests passing, ~10,900 LOC

### Next Steps
- 📅 HTTP/WebSocket server (Phase 6 - Steps 5-8)
- 📅 Historical backtesting (Phase 7)

## Quick Start

```bash
# Navigate to engine core
cd engine-core

# Build the project
cargo build

# Run demo with simulated data (fast, no network)
cargo run

# Run demo with live Binance data (requires network)
cargo run -- --binance

# Run tests
cargo test

# Run Binance integration tests (slow, requires network)
cargo test --test binance_integration -- --ignored --nocapture

# Run Lua strategy integration tests
cargo test --test lua_strategy_integration

# Run examples
cargo run --example indicators_demo
cargo run --example state_machine_demo
cargo run --example lua_strategy_demo
cargo run --example multi_symbol_engine_demo

# View documentation
cargo doc --no-deps --open
```

### Demo Output

**Simulated Feed:**
```
INFO  Trading Engine Demo
INFO  Running SIMULATED FEED demo
INFO  Tick 1: BTCUSDT - O:49650.54 H:49779.63 L:49579.16 C:49679.90 V:1147 | Bid:49655.07 Ask:49704.73
...
INFO  10-period High: 51672.62
INFO  10-period Low: 49579.16
```

**Live Binance Feed:**
```
INFO  Running BINANCE LIVE FEED demo
INFO  Using Binance.US endpoint (wss://stream.binance.us:9443)
INFO  Connected successfully!
INFO  Kline #1: BTCUSDT - O:85981.48 H:85981.48 L:85981.48 C:85981.48 V:0 | Bid:85705.96 Ask:85979.13
INFO  Kline #2: ETHUSDT - O:2821.88 H:2821.88 L:2821.88 C:2821.88 V:0 | Bid:2816.94 Ask:2823.60
...
```

## Project Structure

```
trading-simulator/
├── engine-core/          # Main Rust engine
│   ├── src/
│   │   ├── market_data/  # OHLCV data structures
│   │   ├── sources/      # Data source implementations (Binance, Simulated)
│   │   ├── storage/      # Thread-safe storage
│   │   ├── indicators/   # Technical indicators (SMA, EMA, RSI, MACD, BB)
│   │   ├── state_machine/# Trading FSM and position tracking
│   │   ├── strategy/     # Lua integration layer
│   │   ├── config/       # Configuration types
│   │   ├── error.rs      # Error handling
│   │   ├── lib.rs        # Library root
│   │   └── main.rs       # Binary entry point
│   ├── tests/            # Integration tests
│   ├── examples/         # Demo applications
│   └── Cargo.toml
├── ocaml-indicators/     # OCaml indicator library (Phase 2)
│   ├── src/              # Pure functional implementations
│   ├── bin/              # CLI with JSON I/O
│   └── test/             # OCaml test suites
├── lua-strategies/       # Strategy scripts (Phase 4)
│   └── examples/         # EMA crossover, RSI mean reversion, Range breakout
├── changes/              # Phase completion summaries
├── tests/                # End-to-end tests
└── docs/                 # Documentation
    ├── architecture/     # System design docs
    ├── guides/           # User guides
    ├── decisions/        # Architecture Decision Records
    └── README.md
```

## Documentation

### User Guides
- **[Getting Started Guide](docs/guides/getting-started.md)** - Setup and first run
- **[Binance Setup Guide](docs/guides/binance-setup.md)** - Live market data configuration
- **[Lua Strategy Development Guide](docs/guides/lua-strategy-guide.md)** - Creating custom trading strategies ✨ **NEW!**

### Technical Documentation
- **[Architecture Overview](docs/architecture/01-overview.md)** - System design
- **[Strategy Integration Architecture](docs/architecture/02-strategy-integration.md)** - How strategies work
- **[Event System Architecture](docs/architecture/03-event-system.md)** - Real-time event streaming ✨ **NEW!**
- **[Web App Architecture](WEB_APP_ARCHITECTURE.md)** - HTTP/WebSocket API design ✨ **NEW!**
- **[API Documentation](engine-core/target/doc/trading_engine/index.html)** - Generated from code (`cargo doc --open`)
- **[Full Roadmap](trading-system-roadmap.md)** - Complete project plan

### Architecture Decision Records (ADRs)
- [ADR 001: Rust Edition 2021](docs/decisions/001-rust-2021.md)
- [ADR 002: engine-core Naming](docs/decisions/002-engine-core-naming.md)
- [ADR 003: Circular Buffer for Windows](docs/decisions/003-circular-buffer.md)
- [ADR 004: Subprocess over FFI for OCaml](docs/decisions/004-subprocess-over-ffi.md)
- [ADR 005: Runner-Based Architecture](docs/decisions/005-runner-based-architecture.md) ✨ **NEW!**

### Phase Completion Reports
- [Phase 1: Market Data Infrastructure](changes/2025-12-17-phase1-completion.md)
- [Phase 2: Technical Indicators](changes/2025-12-18-phase2-completion.md)
- [Phase 3: State Machine Core](changes/2025-12-18-phase3-completion.md)
- [Phase 4: Lua Strategy Integration](changes/2025-12-18-phase4-completion.md)
- [Phase 5: Multi-Symbol Threading Engine](changes/2025-12-19-phase5-completion.md)
- [Phase 6: Event System (Steps 1-3)](changes/2025-12-20-phase6-event-system.md)
- [Phase 6: State Introspection (Step 4)](changes/2025-12-20-phase6-state-introspection.md) ✨ **NEW!**

## Development

### Prerequisites

- Rust 1.70+ ([install from rustup.rs](https://rustup.rs))
- Git

### Build & Test

```bash
cd engine-core

# Format code
cargo fmt

# Lint
cargo clippy

# Run tests
cargo test

# Build release
cargo build --release
```

### Documentation

```bash
# Generate API documentation
cargo doc --no-deps --open

# Test documentation examples
cargo test --doc
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    TradingEngine                                │
│  - Multi-runner management                                      │
│  - Broadcast data to runners                                    │
│  - Health monitoring                                            │
└──────────────────────┬──────────────────────────────────────────┘
                       │ Spawns & manages
                       ↓
         ┌─────────────────────────┐ ┌─────────────────────────┐
         │  SymbolRunner (BTC-EMA) │ │  SymbolRunner (BTC-RSI) │  ... (N runners)
         │  - Async task per runner│ │  - Independent state    │
         │  - Channel-based comms  │ │  - Own strategy         │
         └────────┬────────────────┘ └────────┬────────────────┘
                  │                           │
                  ↓                           ↓
         ┌────────────────────────────────────────────────┐
         │      Lua Strategy Scripts                     │  User-defined trading logic
         │  (EMA crossover, RSI, Range breakout)         │
         └─────────────┬──────────────────────────────────┘
                       │ Returns Actions
                       ↓
         ┌─────────────────────────────────────────────┐
         │      LuaStrategy (Rust wrapper)             │  VM management, type conversion
         └─────────────┬───────────────────────────────┘
                       │ Executes Actions
                       ↓
         ┌─────────────────────────────────────────────┐
         │      StateMachine (3-state FSM)             │  Idle → Analyzing → InPosition
         │  - Position tracking & P&L                  │
         │  - Auto-exit on stop/profit                 │
         │  - Transition history                       │
         └─────────────┬───────────────────────────────┘
                       │ Queries
                       ↓
         ┌─────────────────────────────────────────────┐
         │    Technical Indicators (Rust/OCaml)        │  SMA, EMA, RSI, MACD, BB
         │  - Rust: Performance                        │
         │  - OCaml: Correctness verification          │
         └─────────────┬───────────────────────────────┘
                       │ Calculates from
                       ↓
         ┌─────────────────────────────────────────────┐
         │    MarketDataWindow (circular buffer)       │  Time-series OHLCV data
         │  - O(1) insertion                           │
         │  - Bounded memory                           │
         └─────────────┬───────────────────────────────┘
                       │ Fed by
                       ↓
         ┌─────────────────────────────────────────────┐
         │    Data Sources (WebSocket/Simulated)       │  Real-time or test data
         │  - Binance WebSocket                        │
         │  - Simulated random walk                    │
         └─────────────────────────────────────────────┘
```

## Features Summary

| Phase | Feature | Status | Tests |
|-------|---------|--------|-------|
| 1 | Market Data Infrastructure | ✅ | 47 |
| 2 | Technical Indicators (Rust/OCaml) | ✅ | 48 |
| 3 | State Machine & Position Tracking | ✅ | 28 |
| 4 | Lua Strategy Integration | ✅ | 14 |
| 5 | Multi-Symbol Threading Engine | ✅ | 28 |
| 6 | Execution & Risk Management | 📅 | - |
| 7 | Historical Backtesting | 📅 | - |

**Total: 145 tests passing, 5 of 12 phases complete (42%)**

See [trading-system-roadmap.md](trading-system-roadmap.md) for complete plan.

## Creating a Lua Strategy

Strategies are simple Lua scripts with 3 required functions:

```lua
-- lua-strategies/my_strategy.lua

function detect_opportunity(market_data, context, indicators)
    -- Called in Idle state to find trading opportunities
    local ema_10 = indicators.ema(10)
    local ema_20 = indicators.ema(20)

    if ema_10 and ema_20 and ema_10 > ema_20 then
        return { signal = "bullish", confidence = 0.8 }
    end
    return nil
end

function filter_commitment(market_data, context, indicators)
    -- Called in Analyzing state to decide on entry
    if context.signal == "bullish" then
        return {
            action = "enter_long",
            price = market_data.close,
            quantity = 0.1
        }
    end
    return { action = "cancel_analysis", reason = "No signal" }
end

function manage_position(market_data, context, indicators)
    -- Called in InPosition state to manage the trade
    local ema_10 = indicators.ema(10)
    local ema_20 = indicators.ema(20)

    if ema_10 and ema_20 and ema_10 < ema_20 then
        return {
            action = "exit",
            price = market_data.close,
            reason = "Bearish crossover"
        }
    end
    return nil
end
```

See [lua-strategies/examples/](lua-strategies/examples/) for complete examples.

## Performance Targets

- State machine: 1000+ transitions/second ✅ (achieved)
- Concurrent symbols: 100+ with <1% CPU per symbol
- Indicator calculations: <1ms for typical periods ✅ (achieved)
- Lua strategy overhead: <1ms per tick ✅ (achieved)
- End-to-end latency: <10ms from data → action
- Memory: <100MB per symbol

## License

[Your license here]

## Contributing

See [docs/README.md](docs/README.md) for documentation standards and contributing guidelines.
