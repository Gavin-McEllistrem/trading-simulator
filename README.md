# Trading Simulator

A high-performance, multi-threaded trading system with Rust core engine, OCaml indicators, Lua strategy scripting, and comprehensive web interface.

## Project Status

**Current Phase:** 6 (Web App Infrastructure) ✅ **COMPLETE**

**Progress:** 6.0 of 12 phases complete (50% of core system)

### Completed (Phases 0-6)

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
  - Runs indefinitely until channel closed
- **TradingEngine** multi-runner management (~1,400 LOC)
  - **Runner-based architecture**: multiple strategies per symbol
  - Efficient broadcast to all runners watching a symbol
  - Dynamic runner add/remove with control commands
- **Health monitoring** & error recovery
  - Per-runner health checks and uptime tracking
  - Engine-wide health summary
- **Runner control system**:
  - RunnerStatus enum (Running, Paused, Stopped)
  - Pause/Resume/Stop commands via channels
  - State preservation during pause
- **28 tests** passing (17 unit + 11 integration)
- **Demo**: 6 concurrent runners (2 strategies × 3 symbols)

**Phase 6: Web App Infrastructure** ✅ **COMPLETE**
- **Event System** (~258 LOC)
  - 10 event types (TickReceived, StateTransition, PositionUpdated, etc.)
  - Real-time streaming from runners → engine → multiple subscribers
  - Event aggregation with automatic subscriber cleanup
  - JSON serialization for WebSocket transmission
  - 10 tests passing
- **State Introspection API** (~195 LOC)
  - Command channel for querying runner state on-demand
  - `get_runner_snapshot()` - Query current state, position, context, stats
  - `get_price_history()` - Query recent price data from window
  - RunnerSnapshot with full JSON serialization
  - Non-blocking queries with 100ms timeout
  - 7 tests passing
- **Web Backend - HTTP Server** (~800 LOC)
  - axum-based REST API server
  - **14 endpoints**: health, engine summary, runner CRUD, control, strategies, symbols
  - Complete error handling with proper HTTP status codes
  - JSON request/response serialization
  - CORS middleware and request logging
  - **Live Binance US WebSocket integration** in background task
  - Automatic market data feed when runners are created
  - 11 tests passing
- **Web Frontend - React Application** (~1,200 LOC) ✨
  - React 18 + TypeScript + Vite
  - TanStack Query for data fetching and caching
  - Recharts for candlestick chart visualization
  - Tailwind CSS v4 for styling
  - **Dashboard page**:
    - Engine summary with metrics
    - **Enhanced runner list table** with status indicators
    - Symbol dropdown (18 symbols across crypto, stocks, forex)
    - Strategy dropdown (automatically populated from lua-strategies/)
    - Runner control buttons (pause/resume/stop)
  - **Runner detail page**:
    - Live state, position, stats
    - Price chart with real-time updates
  - Auto-refresh: Dashboard (5s for summary, 3s for snapshots), Runner details (2s)
  - Type-safe API client with error handling
- **Architecture**
  - Push model: Real-time events via pub-sub (prepared for WebSocket)
  - Pull model: On-demand snapshots via REST API + auto-polling
  - Live data: Binance US feed → Engine → Runners
  - Combined: Complete observability for web dashboards

**Total:** 120 tests passing, ~14,000 LOC

### Next Steps
- 📅 Historical backtesting (Phase 7)
- 📅 WebSocket real-time streaming (Phase 6 - Optional)

## Quick Start

### Trading Engine

```bash
# Navigate to project root
cd /path/to/trading-simulator

# Build the project
cargo build

# Run demo with simulated data (fast, no network)
cargo run --example multi_symbol_engine_demo

# Run with live Binance US data
cargo run -p trading-web-backend

# Run tests
cargo test

# Run specific package tests
cargo test -p trading-engine
cargo test -p trading-web-backend

# View documentation
cargo doc --no-deps --open
```

### Web Backend (REST API)

```bash
# Run from repository root (important for lua-strategies path)
cd /path/to/trading-simulator
cargo run -p trading-web-backend

# In another terminal, test the API
curl http://localhost:3000/health
curl http://localhost:3000/api/engine/summary
curl http://localhost:3000/api/strategies
curl http://localhost:3000/api/symbols

# Run backend tests
cargo test -p trading-web-backend
```

### Web Frontend (React Dashboard)

```bash
# Navigate to frontend directory
cd web-frontend

# Install dependencies (first time only)
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

Open your browser to `http://localhost:5173` to access the dashboard.

**Features:**
- View engine status and all active runners in a comprehensive table
- Control runners: pause, resume, stop with instant feedback
- Create new runners:
  - Select from 18 curated symbols (crypto, stocks, forex)
  - Choose from available Lua strategies
  - Customize window size and parameters
- Monitor runner state, positions, and P&L in real-time
- Visualize price charts with candlestick data
- Auto-refreshing data with optimized polling intervals

### Demo Output

**Web Backend with Live Binance Feed:**
```
INFO  Trading System Web Backend v0.1.0
INFO  Server will listen on 127.0.0.1:3000
INFO  Trading engine initialized
INFO  Starting server on 127.0.0.1:3000
INFO  Starting Binance US feed for symbols: ["BTCUSDT", "ETHUSDT"]
```

**Frontend Dashboard:**
- Engine metrics: runners count, healthy runners, active symbols
- Runner table with status indicators, control buttons
- Strategy dropdown populated from lua-strategies/
- Symbol dropdown with 18 organized options

## Project Structure

```
trading-simulator/
├── Cargo.toml            # Workspace configuration
├── engine-core/          # Main Rust engine
│   ├── src/
│   │   ├── market_data/  # OHLCV data structures
│   │   ├── sources/      # Data source implementations (Binance, Simulated)
│   │   ├── storage/      # Thread-safe storage
│   │   ├── indicators/   # Technical indicators (SMA, EMA, RSI, MACD, BB)
│   │   ├── state_machine/# Trading FSM and position tracking
│   │   ├── strategy/     # Lua integration layer
│   │   ├── runner/       # Multi-symbol engine, events, introspection, control
│   │   ├── config/       # Configuration types
│   │   ├── error.rs      # Error handling
│   │   ├── lib.rs        # Library root
│   │   └── main.rs       # Binary entry point
│   ├── tests/            # Integration tests
│   ├── examples/         # Demo applications
│   └── Cargo.toml
├── web-backend/          # HTTP/WebSocket server
│   ├── src/
│   │   ├── routes/       # API endpoint handlers
│   │   │   ├── engine.rs     # Engine endpoints
│   │   │   ├── health.rs     # Health checks
│   │   │   ├── runners.rs    # Runner CRUD + control
│   │   │   ├── strategies.rs # Strategy + symbol listings
│   │   │   └── mod.rs
│   │   ├── error.rs      # API error types
│   │   ├── state.rs      # Application state
│   │   ├── lib.rs        # Server core
│   │   └── main.rs       # Entry point with Binance US feed
│   ├── tests/            # Integration tests
│   └── Cargo.toml
├── web-frontend/         # React dashboard
│   ├── src/
│   │   ├── main.tsx      # Entry point
│   │   ├── App.tsx       # Root component with routing
│   │   ├── pages/
│   │   │   ├── Dashboard.tsx    # Main dashboard with runner table
│   │   │   └── RunnerDetail.tsx # Runner detail with charts
│   │   ├── components/
│   │   │   ├── AddRunnerForm.tsx    # Runner creation with dropdowns
│   │   │   └── RunnerListTable.tsx  # Enhanced table with controls
│   │   ├── services/
│   │   │   └── api.ts    # API client (14 endpoints)
│   │   ├── hooks/
│   │   │   └── useApi.ts # React Query hooks
│   │   └── types/
│   │       └── api.ts    # TypeScript types
│   ├── package.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   └── postcss.config.js
├── ocaml-indicators/     # OCaml indicator library
│   ├── src/              # Pure functional implementations
│   ├── bin/              # CLI with JSON I/O
│   └── test/             # OCaml test suites
├── lua-strategies/       # Strategy scripts
│   ├── examples/         # EMA crossover, RSI mean reversion, Range breakout
│   └── test_strategy.lua # Test strategy
├── changes/              # Phase completion summaries
├── tests/                # End-to-end tests
└── docs/                 # Documentation
    ├── architecture/     # System design docs
    ├── guides/           # User guides
    └── decisions/        # Architecture Decision Records
```

## Documentation

### User Guides
- **[Getting Started Guide](docs/guides/getting-started.md)** - Setup and first run
- **[Binance Setup Guide](docs/guides/binance-setup.md)** - Live market data configuration
- **[Lua Strategy Development Guide](docs/guides/lua-strategy-guide.md)** - Creating custom trading strategies

### Technical Documentation
- **[Architecture Overview](docs/architecture/01-overview.md)** - System design
- **[Strategy Integration Architecture](docs/architecture/02-strategy-integration.md)** - How strategies work
- **[Event System Architecture](docs/architecture/03-event-system.md)** - Real-time event streaming
- **[Web App Architecture](WEB_APP_ARCHITECTURE.md)** - HTTP/WebSocket API design
- **[API Documentation](engine-core/target/doc/trading_engine/index.html)** - Generated from code (`cargo doc --open`)
- **[Full Roadmap](trading-system-roadmap.md)** - Complete project plan
- **[Web App Expansion Plan](WEB_APP_EXPANSION_PLAN.md)** - Future enhancements

### Architecture Decision Records (ADRs)
- [ADR 001: Rust Edition 2021](docs/decisions/001-rust-2021.md)
- [ADR 002: engine-core Naming](docs/decisions/002-engine-core-naming.md)
- [ADR 003: Circular Buffer for Windows](docs/decisions/003-circular-buffer.md)
- [ADR 004: Subprocess over FFI for OCaml](docs/decisions/004-subprocess-over-ffi.md)
- [ADR 005: Runner-Based Architecture](docs/decisions/005-runner-based-architecture.md)

### Phase Completion Reports
- [Phase 1: Market Data Infrastructure](changes/2025-12-17-phase1-completion.md)
- [Phase 2: Technical Indicators](changes/2025-12-18-phase2-completion.md)
- [Phase 3: State Machine Core](changes/2025-12-18-phase3-completion.md)
- [Phase 4: Lua Strategy Integration](changes/2025-12-18-phase4-completion.md)
- [Phase 5: Multi-Symbol Threading Engine](changes/2025-12-19-phase5-completion.md)
- [Phase 6: Event System (Steps 1-3)](changes/2025-12-20-phase6-event-system.md)
- [Phase 6: State Introspection (Step 4)](changes/2025-12-20-phase6-state-introspection.md)
- [Phase 6: Web Backend Scaffolding (Step 5)](changes/2025-12-20-web-backend-scaffolding.md)
- [Phase 6: REST API Implementation (Steps 6-7)](changes/2025-12-20-rest-api-implementation.md)
- [Phase 6: Web Application (Steps 8-9)](changes/2025-12-20-web-application-implementation.md)
- [Phase 6: Enhanced UI (Step 10)](changes/2025-12-21-enhanced-ui-completion.md) ✨ **NEW!**

## Development

### Prerequisites

- Rust 1.70+ ([install from rustup.rs](https://rustup.rs))
- Node.js 18+ and npm (for frontend)
- Git

### Build & Test

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Run all tests
cargo test

# Run specific package tests
cargo test -p trading-engine
cargo test -p trading-web-backend

# Build release
cargo build --release
```

### Frontend Development

```bash
cd web-frontend

# Install dependencies
npm install

# Start dev server with hot reload
npm run dev

# Type checking
npm run type-check

# Build for production
npm run build
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
│                    Web Frontend (React)                          │
│  - Dashboard with runner table                                  │
│  - Runner creation with dropdowns                               │
│  - Runner control (pause/resume/stop)                           │
│  - Real-time charts and position tracking                       │
└──────────────────────┬──────────────────────────────────────────┘
                       │ HTTP REST API
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│                    Web Backend (Axum)                            │
│  - 14 REST endpoints                                            │
│  - Runner CRUD + control                                        │
│  - Strategy/symbol listings                                     │
│  - Binance US feed integration                                  │
└──────────────────────┬──────────────────────────────────────────┘
                       │ Rust API calls
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│                    TradingEngine                                │
│  - Multi-runner management                                      │
│  - Broadcast data to runners                                    │
│  - Health monitoring                                            │
│  - Runner control (pause/resume/stop)                           │
└──────────────────────┬──────────────────────────────────────────┘
                       │ Spawns & manages
                       ↓
         ┌─────────────────────────┐ ┌─────────────────────────┐
         │  SymbolRunner (BTC-EMA) │ │  SymbolRunner (BTC-RSI) │
         │  - Async task per runner│ │  - Independent state    │
         │  - Channel-based comms  │ │  - Own strategy         │
         │  - Status: Running/     │ │  - Pause/resume support │
         │    Paused/Stopped       │ │                         │
         └────────┬────────────────┘ └────────┬────────────────┘
                  │                           │
                  ↓                           ↓
         ┌────────────────────────────────────────────────┐
         │      Lua Strategy Scripts                     │
         │  (EMA crossover, RSI, Range breakout)         │
         └─────────────┬──────────────────────────────────┘
                       │ Returns Actions
                       ↓
         ┌─────────────────────────────────────────────┐
         │      LuaStrategy (Rust wrapper)             │
         └─────────────┬───────────────────────────────┘
                       │ Executes Actions
                       ↓
         ┌─────────────────────────────────────────────┐
         │      StateMachine (3-state FSM)             │
         │  - Position tracking & P&L                  │
         │  - Auto-exit on stop/profit                 │
         └─────────────┬───────────────────────────────┘
                       │ Queries
                       ↓
         ┌─────────────────────────────────────────────┐
         │    Technical Indicators (Rust/OCaml)        │
         └─────────────┬───────────────────────────────┘
                       │ Calculates from
                       ↓
         ┌─────────────────────────────────────────────┐
         │    MarketDataWindow (circular buffer)       │
         └─────────────┬───────────────────────────────┘
                       │ Fed by
                       ↓
         ┌─────────────────────────────────────────────┐
         │    Data Sources (WebSocket/Simulated)       │
         │  - Binance WebSocket (Binance US)           │
         │  - Simulated random walk                    │
         └─────────────────────────────────────────────┘
```

## API Endpoints

### Engine Endpoints
- `GET /health` - Basic health check
- `GET /api/engine/health` - Engine health with runner counts
- `GET /api/engine/summary` - Complete engine summary

### Runner Endpoints
- `GET /api/runners/:id/snapshot` - Get runner snapshot
- `GET /api/runners/:id/history` - Get price history
- `POST /api/runners` - Create new runner
- `DELETE /api/runners/:id` - Remove runner
- `POST /api/runners/:id/pause` - Pause runner
- `POST /api/runners/:id/resume` - Resume paused runner
- `POST /api/runners/:id/stop` - Stop runner

### Reference Data Endpoints
- `GET /api/strategies` - List available strategies
- `GET /api/symbols` - List available symbols (18 curated)

## Features Summary

| Phase | Feature | Status | Tests |
|-------|---------|--------|-------|
| 1 | Market Data Infrastructure | ✅ | 47 |
| 2 | Technical Indicators (Rust/OCaml) | ✅ | 48 |
| 3 | State Machine & Position Tracking | ✅ | 28 |
| 4 | Lua Strategy Integration | ✅ | 14 |
| 5 | Multi-Symbol Threading Engine | ✅ | 28 |
| 6 | Web Application (Full Stack) | ✅ | 38 |
| 7 | Historical Backtesting | 📅 | - |
| 8 | Execution & Risk Management | 📅 | - |

**Total: 203 tests passing, 6.0 of 12 phases complete (50%)**

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
- Web API response time: <50ms for snapshots
- Frontend render: <16ms (60 FPS) for charts

## License

[Your license here]

## Contributing

See [docs/README.md](docs/README.md) for documentation standards and contributing guidelines.
