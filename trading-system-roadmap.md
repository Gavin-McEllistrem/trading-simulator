# Trading System Roadmap

## Project Overview
Build a high-performance, multi-threaded trading system with:
- **Rust core engine** for state machine and concurrency
- **Lua scripting** for strategy definitions
- **OCaml** for pure functional indicators
- Multi-symbol support with per-symbol threading

## Current Status (Updated: 2025-12-20, End of Day 6)

**Phase 0: Project Setup** ✅ **COMPLETE** (Day 1)
- ✅ Rust project initialized with full dependency stack
- ✅ Project structure created (engine-core, docs, tests)
- ✅ Version control with .gitignore
- ✅ Comprehensive build system configured

**Phase 1: Market Data Infrastructure** ✅ **COMPLETE** (Day 1-3)
- ✅ Core data structures (MarketData, MarketDataWindow)
- ✅ Data source abstraction (MarketDataSource trait)
- ✅ Binance WebSocket integration with real bid/ask prices
- ✅ Thread-safe storage (MarketDataStorage)
- ✅ Simulated feed for testing
- ✅ 47 tests passing (18 unit + 13 integration + 15 doc + 3 Binance)

**Phase 2: Technical Indicators** ✅ **COMPLETE** (Day 4)
- ✅ OCaml Indicator Library (682 LOC)
  - Pure functional implementations: SMA, EMA, RSI, MACD, Bollinger Bands
  - CLI with JSON I/O for subprocess communication
  - 8 comprehensive test suites (all passing)
- ✅ Rust Native Indicators (602 LOC)
  - Mirror implementations of all OCaml indicators
  - 25 unit tests, all passing
  - Zero-copy iterators for efficiency
- ✅ Subprocess Bridge (224 LOC)
  - JSON-based IPC (stdin/stdout)
  - Process pooling ready
  - 1-2ms latency with 1000x performance headroom
  - 6 verification tests (Rust ↔ OCaml match within 0.001 epsilon)
- ✅ Demo application showing dual implementation comparison
- ✅ **Architecture Decision:** Subprocess over FFI
  - Simpler, safer, scalable
  - Sufficient performance for trading use case
  - Independent updates without recompilation

**Phase 3: State Machine Core** ✅ **COMPLETE** (Day 4)
- ✅ 3-state FSM (Idle, Analyzing, InPosition)
- ✅ Position tracking with P&L calculation (250 LOC)
- ✅ Type-safe Context for strategy state (180 LOC)
- ✅ Action system (8 action types)
- ✅ Auto-exit on stop loss/take profit
- ✅ Transition history tracking (bounded to 100)
- ✅ 28 tests passing (3 state + 8 context + 3 action + 9 position + 8 integration)
- ✅ Generic, strategy-agnostic design
- ✅ Demo application with EMA crossover example
- ✅ Total: ~1,000 LOC across 5 modules

**Phase 4: Lua Strategy Integration** ✅ **COMPLETE** (Day 4)
- ✅ LuaStrategy system with VM management (256 LOC)
- ✅ Lua API for type conversions (340 LOC)
- ✅ IndicatorApi exposing SMA, EMA, RSI to Lua
- ✅ Three-function strategy interface (detect_opportunity, filter_commitment, manage_position)
- ✅ Table-based returns with action conversion
- ✅ 3 example strategies: EMA Crossover, RSI Mean Reversion, Range Breakout
- ✅ 14 tests passing (6 unit + 8 integration)
- ✅ Context iterator methods for Lua conversion
- ✅ Demo application showing Lua strategy driving state machine
- ✅ Total: ~900 LOC Rust + 474 LOC Lua

**Phase 5: Multi-Symbol Threading Engine** ✅ **COMPLETE** (Day 5)
- ✅ SymbolRunner orchestration (393 LOC)
  - Per-symbol async task with component coordination
  - Market data window, state machine, strategy, indicators integration
  - Channel-based communication (mpsc::unbounded)
  - Runs indefinitely until channel closed (not limited by tick count)
- ✅ TradingEngine multi-runner management (1,087 LOC)
  - **Runner-based architecture** (not symbol-based)
  - Multiple strategies per symbol simultaneously
  - Efficient broadcast to all runners watching a symbol
  - Dynamic runner addition/removal
- ✅ RunnerConfig & RunnerStats (113 LOC)
  - Production/development/quiet presets
  - Performance metrics: ticks processed, actions executed, error rate
  - Tick duration tracking (avg/min/max)
- ✅ Health Monitoring & Error Recovery
  - Per-runner health checks
  - Unhealthy runner detection
  - Runner uptime tracking
  - Engine-wide health summary
- ✅ 28 tests passing (17 unit + 11 integration)
- ✅ Demo application with 6 concurrent runners (2 strategies × 3 symbols)
- ✅ Total: ~1,600 LOC across 3 modules

**Phase 6: Web App Infrastructure** ✅ **COMPLETE** (Day 6-7)
- ✅ **Event System Foundation** (Steps 1-3 Complete)
  - Event type system with 10 event variants (258 LOC)
  - Runner event emission at all critical points
  - Engine event aggregation with broadcast to multiple subscribers
  - <1ms latency, <0.1% CPU overhead
  - 10 tests passing (event types, emission, aggregation)
  - JSON serialization for WebSocket transmission
- ✅ **State Introspection API** (Step 4 Complete)
  - Command channel for on-demand state queries (193 LOC)
  - `get_runner_snapshot()` - Query state, position, context, stats
  - `get_price_history()` - Query recent price data
  - RunnerSnapshot with full JSON serialization
  - Non-blocking queries with 100ms timeout
  - 7 tests passing (snapshot creation, queries, error handling)
- ✅ **Web Backend - HTTP Server** (Steps 5-7 Complete)
  - axum-based REST API server (950+ LOC)
  - **14 endpoints total**:
    - Engine: health, summary
    - Runners: list, create, delete, snapshot, history
    - Control: pause, resume, stop
    - Reference: strategies, symbols
  - Complete error handling with HTTP status codes
  - CORS middleware and request logging
  - **Live Binance US WebSocket integration** in background task
  - Automatic market data feed when runners are created
  - 11 tests passing (error types, endpoints, integration)
- ✅ **Web Frontend - React Dashboard** (Steps 8-9 Complete)
  - React 18 + TypeScript + Vite (1,200+ LOC)
  - TanStack Query for data fetching/caching
  - Recharts for candlestick chart visualization
  - Tailwind CSS v4 for styling
  - **Dashboard page**: Engine summary, runner list, add runner form
  - **Runner detail page**: Live state, position, stats, price charts
  - Auto-refresh: Dashboard (5s), Runner details (2s)
  - Type-safe API client with error handling
- ✅ **Enhanced Frontend Features** (Step 10 Complete) ✨ **NEW!**
  - **Runner Control System**:
    - RunnerStatus enum (Running, Paused, Stopped)
    - Pause/Resume/Stop commands via channels
    - Enhanced RunnerListTable with control buttons
    - Status color indicators (green/yellow/red)
    - Real-time state updates with auto-refresh
  - **Strategy Dropdown**:
    - Dynamic strategy listing from lua-strategies/
    - Multi-path resolution for different working dirs
    - Strategy metadata (name, path, category)
  - **Symbol Dropdown**:
    - 18 curated symbols across 4 categories
    - Crypto Major/Alt, Stocks Tech, Forex Major
    - Grouped dropdown with optgroups
  - Backend: +600 LOC (runner control, reference data)
  - Frontend: +400 LOC (enhanced components, dropdowns)
  - Tests: 6 additional snapshot tests
- 📅 WebSocket real-time streaming (Optional - Phase 7)

**Documentation** ✅ **COMPLETE** (Day 1-6)
- ✅ Comprehensive Rustdoc comments (30+ tested examples in docstrings)
- ✅ Architecture overview (docs/architecture/01-overview.md)
- ✅ Strategy integration architecture (docs/architecture/02-strategy-integration.md)
- ✅ **Event system architecture** (docs/architecture/03-event-system.md) ✨ **NEW!**
- ✅ **Web app architecture** (WEB_APP_ARCHITECTURE.md) ✨ **NEW!**
- ✅ Getting started guide (docs/guides/getting-started.md)
- ✅ **Binance setup guide** (docs/guides/binance-setup.md) - comprehensive usage documentation
- ✅ **Lua strategy guide** (docs/guides/lua-strategy-guide.md) - complete tutorial
- ✅ 5 Architecture Decision Records:
  - ADR-001: Rust 2021 Edition
  - ADR-002: engine-core naming
  - ADR-003: Circular buffer design
  - ADR-004: Subprocess over FFI for OCaml indicators
  - ADR-005: Runner-based architecture over symbol-based
- ✅ All public APIs documented with examples
- ✅ Phase completion summaries (6 detailed reports in changes/)

**Testing Infrastructure** ✅ **EXCELLENT COVERAGE** (Day 2-6)
- ✅ **Phase 1 Tests:** 47 tests passing
  - 18 unit tests in src/market_data/tests.rs
  - 7 integration tests in tests/market_data_integration.rs
  - 6 integration tests in tests/data_pipeline_integration.rs
  - 3 Binance integration tests (network required, marked #[ignore])
  - 15 doc tests (examples in documentation)
- ✅ **Phase 2 Tests:** 48 tests passing
  - 25 Rust indicator unit tests
  - 6 Rust/OCaml verification tests
  - 2 OCaml bridge integration tests
  - 8 OCaml test suites
  - 1 demo application test
- ✅ **Phase 3 Tests:** 28 tests passing
  - 3 state tests (state helpers, transitions)
  - 8 context tests (type-safe storage)
  - 3 action tests (action classification)
  - 9 position tests (P&L, stop/profit, lifecycle)
  - 8 state machine integration tests
- ✅ **Phase 4 Tests:** 14 tests passing
  - 6 Lua API unit tests (type conversions)
  - 8 integration tests (strategy loading, execution, all 3 examples)
- ✅ **Phase 5 Tests:** 28 tests passing
  - 17 engine unit tests (creation, add/remove, health monitoring)
  - 11 integration tests (single runner, multi-symbol, concurrent processing)
- ✅ **Phase 6 Tests:** 38 tests passing ✨ **NEW!**
  - 7 event type tests (serialization, helpers, classification)
  - 1 runner emission test
  - 2 engine aggregation tests (single + multiple subscribers)
  - 4 snapshot unit tests (creation, serialization, context)
  - 3 introspection integration tests (snapshot query, history query, error handling)
  - 11 web backend tests (error types, endpoints, integration)
  - 10 frontend manual tests (dashboard, runner detail, charts, forms)
- ✅ **Total: 203 tests, all passing** ✅
- ✅ Test organization: Unit, Integration, Verification, Doc, Live
- ✅ Comprehensive coverage across all phases

**Architecture Decisions** (Day 2-5)
- ✅ **Phase 1 Decisions:**
  - MarketDataWindow = pure data storage and query layer
  - Test organization: separate unit tests from source to prevent code bloat
  - Binance dual-stream approach: kline + bookTicker for accurate bid/ask
  - Thread-safe storage with Arc<RwLock<HashMap>>
- ✅ **Phase 2 Decisions:**
  - Subprocess over FFI for OCaml integration (ADR-004)
  - Dual implementation: Rust for production, OCaml for correctness verification
  - JSON-based IPC for language-agnostic communication
  - No process pooling yet (sufficient performance without it)
- ✅ **Phase 3 Decisions:**
  - Generic, strategy-agnostic state machine
  - Type-safe context with multiple HashMaps (not serde_json::Value)
  - Auto-exit protection for risk management
  - Bounded transition history (last 100 transitions)
  - Separation: StateMachine manages state, Strategies generate Actions
- ✅ **Phase 4 Decisions:**
  - Lua 5.4 with Send + Serialize features for multi-threading
  - Three-function strategy interface (detect_opportunity, filter_commitment, manage_position)
  - Table-based returns (not multiple values)
  - Explicit context updates (no automatic state pollution)
  - Script validation at load time
- ✅ **Phase 5 Decisions:**
  - Runner-based architecture (not symbol-based) for strategy flexibility
  - Multiple strategies per symbol via unique runner IDs
  - Channel-based message passing (mpsc::unbounded) for data distribution
  - Tokio async tasks (one per runner) for true parallelism
  - Runner lifetime: indefinite until channel closed (not tick-limited)
  - Health monitoring at both runner and engine levels
- ✅ Clean separation of concerns: data structures (Rust) → indicators (OCaml) → state machine (Rust) → strategies (Lua) → runners (Rust) ✅

**Code Quality** (Day 3)
- ✅ Zero compiler warnings
- ✅ All clippy lints passing
- ✅ Comprehensive error handling with thiserror (split WebSocketError into String and TungsteniteError variants)
- ✅ Full async support with tokio
- ✅ Thread-safe with parking_lot RwLock
- ✅ Demo application with dual mode (simulated + live Binance)

**Project Metrics (Day 7 Summary):**
- **Total Code:** ~13,326 LOC
  - Rust production: ~8,976 LOC (engine core + runner system + web backend)
  - React frontend: ~1,250 LOC (TypeScript + components)
  - OCaml: 1,606 LOC (indicators)
  - Lua strategies: 474 LOC (3 examples)
  - Tests/Examples: ~1,100 LOC
- **Total Tests:** 209 passing ✅
  - Phase 1 (Market Data): 47 tests
  - Phase 2 (Indicators): 48 tests (40 Rust + 8 OCaml)
  - Phase 3 (State Machine): 28 tests
  - Phase 4 (Lua Strategies): 14 tests
  - Phase 5 (Multi-Symbol Engine): 28 tests (17 unit + 11 integration)
  - Phase 6 (Web App): 44 tests (10 event + 7 introspection + 11 backend + 6 snapshot + 10 frontend)
- **Documentation:** 17+ guides, 5 ADRs, 9 phase summaries, comprehensive API docs
- **Completion:** 6.0 of 12 phases complete (50% of core system, Phase 6: 100% complete)

**Next Milestone:** Historical Backtesting (Phase 7)
- **Goal:** Add historical data management and backtesting framework
- **Current:** Complete web app with runner control, strategy/symbol dropdowns, real-time monitoring
- **Next:** Historical data fetching, backtest runner, performance metrics, replay system
- **Previous Milestone Complete:** Phase 6 - Full web infrastructure with control system ✅

---

## Phase 0: Project Setup & Foundation (Week 1) ✅ COMPLETE

### 0.1 Development Environment ✅
- [x] Set up Rust project with `cargo new trading-engine` (named `engine-core`)
- [ ] Configure OCaml environment with `opam` and `dune` (Deferred to Phase 2)
- [ ] Set up Lua development environment (Deferred to Phase 4)
- [x] Create workspace structure:
  ```
  trading-simulator/
  ├── engine-core/        # Main engine (renamed from rust-core)
  ├── ocaml-indicators/   # Indicator library
  ├── lua-strategies/     # Strategy scripts
  ├── tests/              # Integration tests
  └── docs/               # Documentation
  ```

### 0.2 Core Dependencies ✅
- [x] Add Rust dependencies to `Cargo.toml`:
  - `tokio` (async runtime)
  - ~~`mlua` (Lua embedding)~~ (Deferred to Phase 4)
  - `serde` (serialization)
  - `chrono` (time handling)
  - `crossbeam` (concurrent data structures)
  - `tracing` (logging)
  - `tokio-tungstenite` (WebSocket for market data)
  - `async-trait` (async traits)
  - `parking_lot` (fast locks)
  - `thiserror` (error handling)
  - `anyhow` (error propagation)
  - `rand` (SimulatedFeed)
- [ ] Set up OCaml `dune` project (Deferred to Phase 2)
- [ ] Configure FFI bridge between Rust and OCaml (Deferred to Phase 2)

### 0.3 Version Control & CI
- [x] Initialize git repository (already initialized as submodule)
- [x] Create `.gitignore` for Rust, OCaml, Lua
- [ ] Set up GitHub Actions for CI (Deferred - not needed for Day 1)

---

## Phase 1: Market Data Infrastructure (Week 2-3) - IN PROGRESS

### 1.1 Core Data Structures ✅ COMPLETE (Day 1-2)
- [x] Define `MarketData` struct in Rust:
  ```rust
  pub struct MarketData {
      symbol: String,
      timestamp: i64,
      open: f64,
      high: f64,
      low: f64,
      close: f64,
      volume: u64,
      bid: f64,
      ask: f64,
  }
  ```
- [x] Implement `MarketDataWindow` with circular buffer (`VecDeque`)
- [x] Add time-series query methods (`high()`, `low()`, `avg_volume()`, `range()`)
- [x] Add data access methods (`closes()`, `oldest()`, `latest()`, `iter()`, `get()`)
- [x] Add utility methods (`clear()`, `len()`, `is_empty()`)
- [x] Write comprehensive unit tests (18 tests passing)
- [x] Add full Rustdoc documentation with examples (15 doc tests)
- [x] Implement `validate()` method for data consistency checks
- [x] Add `mid_price()` helper method
- [x] **Architecture decision:** No indicators in MarketDataWindow (deferred to OCaml Phase 2)

### 1.1.1 Testing Infrastructure ✅ COMPLETE (Day 2)
- [x] Reorganize tests - separate from source code
- [x] Create `src/market_data/tests.rs` with 18 unit tests
- [x] Create `tests/market_data_integration.rs` with 7 integration tests
- [x] Create `tests/data_pipeline_integration.rs` with 6 integration tests
- [x] All 46 tests passing (18 unit + 13 integration + 15 doc)

### 1.2 Market Data Source Abstraction ✅ COMPLETE (Day 1)
- [x] Design abstract `MarketDataSource` trait:
  ```rust
  #[async_trait]
  pub trait MarketDataSource: Send + Sync {
      async fn connect(&mut self) -> Result<()>;
      async fn subscribe(&mut self, symbols: Vec<String>) -> Result<()>;
      async fn next_tick(&mut self) -> Result<MarketData>;
      async fn disconnect(&mut self) -> Result<()>;
      fn source_name(&self) -> &str;
  }
  ```
- [x] Implement `SimulatedFeed` for testing (random walk generation)
- [ ] Implement `CSVFeed` for backtesting from files (Deferred to Phase 8)
- [ ] Add thread-safe data distribution to symbol runners (Deferred to Phase 5)

### 1.3 Binance WebSocket Integration (Primary Crypto Source) ✅ COMPLETE (Day 3)
- [x] Add dependencies: `tokio-tungstenite`, `serde_json`, `url`
- [x] Create Binance account (no funding needed for market data)
- [x] Implement `BinanceFeed` struct with WebSocket connection:
  ```rust
  pub struct BinanceFeed {
      symbols: Vec<String>,
      interval: String,
      region: BinanceRegion,  // International or US
      ws_stream: Option<WebSocketStream>,
      last_ping: Option<Instant>,
      book_tickers: HashMap<String, BookTicker>,  // Cache bid/ask
  }
  ```
- [x] Parse Binance kline/candlestick events and bookTicker stream:
  ```rust
  #[derive(Debug, Deserialize)]
  struct BinanceKline {
      #[serde(rename = "e")] event_type: String,
      #[serde(rename = "E")] event_time: i64,
      #[serde(rename = "k")] kline: KlineData,
  }

  #[derive(Debug, Deserialize)]
  struct BookTicker {
      #[serde(rename = "u")] update_id: i64,
      #[serde(rename = "s")] symbol: String,
      #[serde(rename = "b")] best_bid: String,
      #[serde(rename = "a")] best_ask: String,
  }
  ```
- [x] Convert Binance string prices to f64 in `MarketData` format
- [x] Implement connection health monitoring with ping/pong (20s interval, 60s timeout)
- [x] Handle Binance rate limits and errors gracefully (proper error types)
- [x] Test with BTC-USDT, ETH-USDT (live data verified at ~$85,978 BTC, ~$2,821 ETH)
- [x] Support multiple timeframes (1s, 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M)
- [x] Add WebSocket URL construction for combined streams:
  - `wss://stream.binance.com:9443/stream?streams={symbol}@kline_{interval}/{symbol}@bookTicker`
  - `wss://stream.binance.us:9443/stream?streams=...` (US endpoint)
- [x] Regional endpoint support (BinanceRegion enum)
- [x] Dual-stream approach: kline for OHLCV + bookTicker for real bid/ask prices
- [x] Only emit completed klines (filter partial candles with `is_closed` flag)
- [x] Integration tests (3 tests in tests/binance_integration.rs)
- [x] Comprehensive documentation (docs/guides/binance-setup.md)

### 1.4 Alpaca Integration (Stock Market Testing)
- [ ] Add dependency: `apca` crate
- [ ] Set up Alpaca paper trading account (free at alpaca.markets)
- [ ] Implement `AlpacaSource` for real-time bar streaming:
  ```rust
  pub struct AlpacaSource {
      client: AlpacaClient,
      stream: Option<BarStream>,
      api_key: String,
      secret_key: String,
  }
  ```
- [ ] Configure API keys via environment variables:
  ```bash
  APCA_API_KEY_ID=your_key
  APCA_API_SECRET_KEY=your_secret
  APCA_API_BASE_URL=https://paper-api.alpaca.markets
  ```
- [ ] Convert Alpaca bar data to internal `MarketData` format
- [ ] Test with major stock symbols (AAPL, MSFT, SPY, QQQ)
- [ ] Handle market hours (pre-market 4am-9:30am, regular 9:30am-4pm, after-hours 4pm-8pm EST)
- [ ] Implement market status checking (is market open?)
- [ ] Add proper error handling for market closed periods

### 1.5 Coinbase Integration (Alternative Crypto Source)
- [ ] Research Coinbase Advanced Trade API
- [ ] Implement `CoinbaseSource` as alternative to Binance
- [ ] Test with BTC-USD, ETH-USD pairs
- [ ] Compare data quality with Binance

### 1.6 Data Source Configuration ✅ COMPLETE
- [x] Create configuration system for selecting data source:
  ```toml
  [data_source]
  type = "binance"  # or "alpaca", "coinbase", "simulated"

  [data_source.binance]
  # Binance-specific config

  [data_source.alpaca]
  api_key_env = "APCA_API_KEY_ID"
  secret_key_env = "APCA_API_SECRET_KEY"
  ```
- [x] Implement configuration types (`DataSourceConfig`, `EngineConfig`)
- [ ] Implement data source factory pattern (Deferred to Day 7)
- [ ] Add validation for required credentials per source (Deferred to Day 7)

### 1.7 Data Storage ✅ COMPLETE
- [x] Create in-memory storage with `Arc<RwLock<HashMap>>` (using `parking_lot`)
- [x] Implement windowed storage (only keep last N datapoints)
- [x] Add `MarketDataStorage` with thread-safe access
- [x] Implement `Clone` for storage (via Arc)
- [ ] Add serialization for market data snapshots (Deferred to Phase 7)
- [ ] Test concurrent read/write performance (Deferred to Day 6)
- [ ] Add optional persistence to disk for replay (Deferred to Phase 7)

### 1.8 Historical Data Download
- [ ] Implement Binance REST API client for historical klines
- [ ] Implement Alpaca REST API for historical bars
- [ ] Create tools for downloading and caching historical data
- [ ] Store historical data in efficient format (Parquet or compressed CSV)

**Deliverable**: Market data can be ingested from multiple sources (Binance, Alpaca), stored, and queried efficiently. Full support for both crypto (24/7) and stock (market hours) data sources.

---

## Phase 2: Technical Indicators Library (Week 3-4) ✅ **COMPLETE** (Day 4)

### 2.1 OCaml Indicator Framework ✅ **COMPLETE**
- [x] Set up OCaml project structure:
  ```
  ocaml-indicators/
  ├── src/
  │   ├── indicators.mli   # Public interface (185 LOC)
  │   ├── indicators.ml    # Implementation (187 LOC)
  ├── bin/
  │   └── main.ml         # CLI with JSON I/O (199 LOC)
  ├── test/
  │   └── test_indicators.ml  # Test suite (111 LOC)
  └── dune-project
  ```
- [x] Define `price_data` type (float array)
- [x] Implement helper functions (average, std_dev, sliding_window)
- [x] 8 test suites, all passing ✅

### 2.2 Indicators Implemented ✅ **COMPLETE**
- [x] Implement SMA (Simple Moving Average)
- [x] Implement EMA (Exponential Moving Average)
- [x] Implement RSI (Relative Strength Index)
- [x] Implement MACD (Moving Average Convergence Divergence)
- [x] Implement Bollinger Bands
- [ ] Implement ATR (Average True Range) - Deferred to Phase 7
- [ ] Implement VWAP (Volume Weighted Average Price) - Deferred to Phase 7
- [ ] Implement Stochastic Oscillator - Deferred to Phase 7

### 2.3 Rust Native Implementation ✅ **COMPLETE**
- [x] Implement all indicators in native Rust (378 LOC)
- [x] Mirror OCaml implementations exactly
- [x] 25 unit tests, all passing ✅
- [x] Comprehensive documentation with examples

### 2.4 Subprocess Bridge (Instead of FFI) ✅ **COMPLETE**
- [x] Create OCaml CLI binary with JSON I/O (199 LOC)
- [x] Implement Rust subprocess wrapper (224 LOC)
- [x] JSON protocol for IPC (stdin/stdout)
- [x] Error handling with proper propagation
- [x] 6 verification tests comparing Rust ↔ OCaml ✅
- [x] Demo application showing both implementations

**Architecture Decision**: Used **subprocess approach** instead of FFI:
- Simpler implementation (no C bindings)
- ~1-2ms latency (acceptable for trading use case)
- 1000x performance headroom vs actual needs
- Jane Street uses this pattern for similar workloads
- Can batch 1000s of prices per call if needed

**Deliverable**: ✅ Dual Rust/OCaml indicator library with full verification (40 Rust tests + 8 OCaml tests passing)

---

## Phase 3: State Machine Core (Week 4-5) ✅ **COMPLETE** (Day 4)

### 3.1 State Machine Foundation ✅
- [x] Define `State` enum (Idle, Analyzing, InPosition) - 3-state FSM
- [x] Define `Context` struct with type-safe flexible data storage (180 LOC)
  - `HashMap<String, String>`, `HashMap<String, f64>`, `HashMap<String, i64>`, `HashMap<String, bool>`
  - Methods: `set()`, `get()`, `remove()`, `clear()`, `latest_price()`, `latest_timestamp()`
- [x] Define `Action` enum (EnterLong, EnterShort, ExitPosition, UpdateStopLoss, UpdateTakeProfit, StartAnalyzing, CancelAnalysis, NoAction)
- [x] Implement `Side` enum (Long, Short) for position tracking
- [x] Implement basic state transitions with logging and history tracking

### 3.2 Position Management ✅
- [x] Implement `Position` struct (250 LOC):
  - Entry tracking: price, quantity, side, timestamp
  - Exit tracking: price, timestamp
  - Current price updates
  - Stop loss and take profit management
  - Unrealized P&L calculation (long and short)
  - Auto-detection of stop loss/take profit hits
- [x] Position lifecycle: new → update → close
- [x] 9 comprehensive position tests (P&L, stop/profit, lifecycle)

### 3.3 State Machine Implementation ✅
- [x] Implement `StateMachine` struct (320 LOC)
- [x] Implement `update()` method with state transition logic
- [x] Add state transition history tracking (bounded to 100)
- [x] Implement auto-exit on stop loss or take profit hit
- [x] Add position management (enter/exit with P&L tracking)
- [x] Implement `execute()` for strategy actions
- [x] Write extensive state transition tests (8 integration tests)
- [x] Public API: `new()`, `current_state()`, `position()`, `context()`, `update()`, `execute()`, `transition_to()`, `reset()`

### 3.4 Context Management ✅
- [x] Implement type-safe context storage using multiple HashMaps
- [x] Add helper methods: `set_number()`, `get_number()`, `set_string()`, `get_string()`, `set_boolean()`, `get_boolean()`, `set_integer()`, `get_integer()`
- [x] Implement context serialization for debugging (serde)
- [x] Test context preservation across state transitions (8 tests)
- [x] Convenience methods: `latest_price()`, `latest_timestamp()`

### 3.5 Demo Application ✅
- [x] Create `state_machine_demo.rs` example (150 LOC)
- [x] Demonstrate EMA crossover strategy (10/20 periods)
- [x] Show full lifecycle: Idle → Analyzing → InPosition → Idle
- [x] Display transition history and P&L tracking
- [x] Test with simulated market data

### 3.6 Testing ✅
**Total: 28 tests passing**
- State tests (3): state helpers, transitions
- Context tests (8): type-safe storage, all data types
- Action tests (3): action classification, side enum
- Position tests (9): P&L long/short, stop/profit, lifecycle
- StateMachine integration (8): full workflows, auto-exit

**Key Metrics:**
- Memory per StateMachine: ~10-15KB
- State transitions: microseconds
- Position updates: nanoseconds
- Can handle 1000s of concurrent symbols

**Architecture Notes:**
- **Generic and reusable**: Not tied to any specific strategy
- **Strategy-agnostic**: Strategies generate Actions, StateMachine executes them
- **Auto-exit protection**: Checks stop/profit on every update
- **Transition history**: Bounded buffer (last 100 transitions)
- **Type-safe context**: Compile-time type checking, no JSON parsing
- **Clean separation**: State management separate from trading logic

**Future Integration (Phase 5):**
```
SymbolRunner (per-symbol)
  ├── StateMachine (✅ Phase 3 - state management)
  ├── Strategy (Phase 4 - Lua logic)
  ├── IndicatorEngine (✅ Phase 2 - technical analysis)
  └── MarketDataWindow (✅ Phase 1 - price history)
```

**Deliverable**: ✅ Generic, production-ready state machine that can be driven by any strategy (Lua in Phase 4)

---

## Phase 4: Lua Strategy Integration (Week 5-6) ✅ **COMPLETE** (Day 4)

### 4.1 Lua Embedding ✅
- [x] Set up `mlua` runtime management (Lua 5.4 with Send + Serialize)
- [x] Implement LuaStrategy with VM per strategy instance
- [x] Define Lua API for market data access:
  ```lua
  market_data.symbol, .timestamp, .open, .high, .low
  market_data.close, .volume, .bid, .ask, .mid_price
  ```

### 4.2 Strategy Loading System ✅
- [x] Implement `LuaStrategy` struct (256 LOC):
  ```rust
  pub struct LuaStrategy {
      lua: Lua,
      script_path: PathBuf,
      strategy_name: String,
  }
  ```
- [x] Load Lua scripts from filesystem with path resolution
- [x] Validate required functions exist (detect_opportunity, filter_commitment, manage_position)
- [x] Implement comprehensive error handling (StrategyError, LuaError)
- [x] Script validation at load time (not runtime)

### 4.3 Lua-Rust Type Conversion ✅
- [x] Implement `MarketData` → Lua table conversion (lua_api.rs)
- [x] Implement `Context` → Lua table conversion (all types: f64, String, i64, bool)
- [x] Implement `IndicatorApi` wrapper for Lua (340 LOC):
  ```lua
  indicators.sma(period)
  indicators.ema(period)
  indicators.rsi(period)
  indicators.high, .low, .range, .avg_volume
  ```
- [x] Implement Lua table → Action conversion (8 action types)
- [x] Add type validation with descriptive error messages
- [x] Context iterator methods for Lua conversion

### 4.4 Example Strategy Library ✅
- [x] Create **EMA Crossover** strategy (140 LOC)
  - Fast/Slow EMA (10/20), volume confirmation
  - 2% stop loss, 5% take profit
- [x] Create **RSI Mean Reversion** strategy (138 LOC)
  - Oversold (<30) entry, neutral (>=50) exit
  - 3% stop loss, 4% take profit
- [x] Create **Range Breakout** strategy (150 LOC)
  - 20-bar range tracking, volume spike confirmation
  - Range low stop, 2x range target
- [x] Document strategy API with comprehensive comments
- [x] Create test strategy for integration tests (46 LOC)

### 4.5 Integration & Testing ✅
- [x] Integration test suite (152 LOC, 8 tests)
  - Strategy loading and validation
  - detect_opportunity execution
  - filter_commitment with context
  - manage_position functionality
  - All 3 example strategies load successfully
- [x] Demo application (150 LOC)
  - Shows Lua strategy driving state machine
  - EMA crossover strategy with simulated data
  - Position tracking and P&L display
  - Transition history

### 4.6 Testing ✅
**Total: 14 tests passing**
- Lua API tests (6): MarketData conversion, Context conversion, Action conversion
- Integration tests (8): Strategy loading, execution, all examples validate
- All tests passing ✅

**Key Metrics:**
- Lua VM overhead: <1ms per tick
- Function call latency: ~0.01-0.1ms
- Memory per strategy: ~100-200KB
- Can support 100+ concurrent strategies

**Architecture:**
```
Lua Strategy Script (user code)
    ↓ returns Actions
LuaStrategy (Rust wrapper)
    ↓ executes
StateMachine (state management)
    ↓ manages
Position (P&L tracking)
```

**Strategy Interface:**
```lua
function detect_opportunity(market_data, context, indicators)
    -- Scan for opportunities (Idle → Analyzing)
    return { signal = "bullish" } or nil
end

function filter_commitment(market_data, context, indicators)
    -- Decide on entry (Analyzing → InPosition)
    return { action = "enter_long", price, quantity } or nil
end

function manage_position(market_data, context, indicators)
    -- Manage trade (InPosition updates)
    return { action = "exit", price } or nil
end
```

**Deliverable**: ✅ Complete Lua strategy system with 3 production-ready example strategies, full testing, and demo application

---

## Phase 5: Multi-Symbol Threading Engine (Week 6-7) ✅ COMPLETE

### 5.1 SymbolRunner Architecture ✅ COMPLETE
- [x] Implemented `SymbolRunner` struct (393 LOC):
  ```rust
  pub struct SymbolRunner {
      symbol: String,
      window: MarketDataWindow,
      state_machine: StateMachine,
      strategy: LuaStrategy,
      data_receiver: mpsc::UnboundedReceiver<MarketData>,
      config: RunnerConfig,
      stats: RunnerStats,
  }
  ```
- [x] Implemented async `run()` method with infinite loop
- [x] Channel-based communication (receives data via mpsc)
- [x] Graceful shutdown when channel closes
- [x] Per-tick processing: window update → strategy execution → state update
- [x] Component orchestration: window + state machine + strategy + indicators
- [x] Statistics tracking per runner (ticks, actions, errors, timing)

### 5.2 TradingEngine - Multi-Runner Management ✅ COMPLETE
- [x] Implemented `TradingEngine` struct (1,087 LOC):
  ```rust
  pub struct TradingEngine {
      runners: HashMap<String, RunnerHandle>,  // runner_id → handle
      subscriptions: HashMap<String, Vec<String>>,  // symbol → runner_ids
      default_config: RunnerConfig,
      default_window_size: usize,
  }
  ```
- [x] **Runner-based architecture** (not symbol-based)
  - Multiple runners can watch the same symbol
  - Each runner has unique ID (e.g., "btc_ema", "btc_rsi")
  - Enables A/B testing and strategy comparison
- [x] Dynamic runner management:
  - `add_runner()` - Add runner with default config
  - `add_runner_with_config()` - Add with custom config
  - `remove_runner()` - Graceful shutdown of specific runner
- [x] Efficient data distribution:
  - Broadcast market data to ALL runners watching a symbol
  - O(n) distribution where n = runners for that symbol
- [x] Tokio task spawning (one task per runner)
- [x] Graceful shutdown with `shutdown()` and `shutdown_with_results()`

### 5.3 RunnerConfig & RunnerStats ✅ COMPLETE
- [x] `RunnerConfig` (config.rs - 63 LOC):
  - `stop_on_error` - Error handling policy
  - `log_actions` - Action logging toggle
  - `log_positions` - Position logging toggle
  - `collect_metrics` - Performance metrics toggle
  - Presets: `production()`, `development()`, `quiet()`
- [x] `RunnerStats` (stats.rs - 98 LOC):
  - Counters: ticks_processed, actions_executed, errors
  - Timing: avg/min/max tick duration, total duration
  - Rates: error_rate(), action_rate()
  - Methods: record_tick(), record_action(), record_error()

### 5.4 Health Monitoring & Error Recovery ✅ COMPLETE
- [x] Per-runner health checks:
  - `runner_is_healthy()` - Check if task is still running
  - `runner_uptime()` - Get duration since runner started
  - `runner_symbol()` - Get symbol for a runner
- [x] Engine-wide monitoring:
  - `health_check()` - Map of all runner health statuses
  - `unhealthy_runners()` - List of failed runners
  - `summary()` - Engine statistics summary
- [x] Query methods:
  - `runner_ids()` - All runner IDs
  - `active_symbols()` - Symbols with runners
  - `runners_for_symbol()` - Get all runners watching a symbol
  - `runner_count_for_symbol()` - Count runners per symbol
  - `has_runner()` - Check if runner exists

### 5.5 Testing & Validation ✅ COMPLETE
- [x] 17 unit tests (engine module):
  - Engine creation and configuration
  - Adding/removing runners
  - Duplicate runner ID handling
  - Multiple runners per symbol
  - Data feeding and broadcasting
  - Health monitoring
  - Shutdown procedures
- [x] 11 integration tests:
  - Single runner with strategy
  - Multiple runners same symbol
  - Multi-symbol engine (3+ symbols)
  - Simulated feed integration
  - Concurrent multi-symbol processing (6 runners, 100 ticks)
  - Error handling (unknown symbol, duplicate IDs)
  - Dynamic runner removal during operation

### 5.6 Demo Application ✅ COMPLETE
- [x] `multi_symbol_engine_demo.rs` (150 LOC):
  - 3 symbols: BTCUSDT, ETHUSDT, SOLUSDT
  - 2 strategies per symbol (6 total runners)
  - 100 ticks of simulated trading
  - Simulated feeds with random price walks
  - Health checks every 25 ticks
  - Real-time uptime display
  - Graceful shutdown with results summary

**Deliverable**: ✅ Production-ready multi-symbol trading engine with runner-based architecture, health monitoring, comprehensive testing, and demo showing 6 concurrent runners across 3 symbols
- [ ] Historical mode monitoring:
  - Progress tracking (% complete)
  - Playback speed
  - Estimated completion time
  - Fast-forward capability
- [ ] Unified logging that includes mode context

### 5.7 Configuration System for Modes
- [ ] Implement TOML configuration for mixed mode runners:
  ```toml
  [[runners]]
  symbol = "BTCUSDT"
  strategy = "strategies/range.lua"
  
  [runners.data_source]
  type = "live"
  source = "binance"
  
  [runners.execution]
  type = "live"
  
  [[runners]]
  symbol = "ETHUSDT"
  strategy = "strategies/range.lua"
  
  [runners.data_source]
  type = "historical"
  path = "data/ETHUSDT-2024.parquet"
  simulate_realtime = true
  speed = 10.0
  
  [runners.execution]
  type = "simulated"
  slippage = 0.001
  ```
- [ ] Support running live and historical runners simultaneously
- [ ] Validate mode compatibility (can't use live execution with historical data)

### 5.8 Concurrency & Synchronization
- [ ] Implement thread-safe market data distribution
- [ ] Add action queue for trade execution
- [ ] Implement cross-symbol coordination (if needed)
- [ ] Add deadlock detection
- [ ] Benchmark with 10, 50, 100+ concurrent symbols (mixed modes)
- [ ] Test performance difference between live and historical modes

### 5.9 Resource Management
- [ ] Implement memory limits per symbol
- [ ] Add CPU usage monitoring
- [ ] Implement automatic throttling under load
- [ ] Add metrics collection (state changes/sec, indicator calls, etc.)
- [ ] Track resources separately for live vs historical runners
- [ ] Implement priority system (live runners get priority over historical)

**Deliverable**: Unified symbol runner architecture where live and historical modes are mechanically identical, differing only in data source blocking behavior and execution simulation. Multiple symbols can run concurrently in either mode or mixed modes.

---

## Phase 6: Web App Infrastructure (Week 7-8) 🚧 IN PROGRESS

### 6.1 Event System Foundation ✅ COMPLETE (Steps 1-3)
- [x] Define `RunnerEvent` enum with 10 event types (258 LOC)
  - Lifecycle events: RunnerStarted, RunnerStopped
  - Trading activity: TickReceived, StateTransition, ActionExecuted
  - Position events: PositionOpened, PositionUpdated, PositionClosed
  - Diagnostics: Error, StatsUpdate
- [x] Add event emission to SymbolRunner (modified mod.rs to 570 LOC)
  - Optional event channel (zero overhead if not subscribed)
  - Events emitted at all critical points (7 emission points)
  - Unique runner_id for event attribution
- [x] Implement event aggregation in TradingEngine (modified engine.rs to 1,236 LOC)
  - Global event broadcast system
  - Multiple subscriber support with auto-cleanup
  - Async event forwarding task
- [x] Add JSON serialization for WebSocket transmission
- [x] Write 10 comprehensive tests (event types, emission, aggregation)
- [x] Create architecture documentation (docs/architecture/03-event-system.md)

### 6.2 State Introspection API (Step 4) ✅ COMPLETE
- [x] Created command & snapshot types (src/runner/snapshot.rs - 193 LOC)
  - Implemented `RunnerCommand` enum (GetSnapshot, GetPriceHistory)
  - Implemented `RunnerSnapshot` struct with full state capture
  - Implemented `ContextSnapshot` for strategy context
  - Added oneshot channel pattern for request-response
- [x] Added command channel to SymbolRunner
  - New field: `command_rx: Option<mpsc::UnboundedReceiver<RunnerCommand>>`
  - Builder method: `with_command_channel(rx)`
  - Modified `run()` loop to use `tokio::select!` for concurrent processing
- [x] Implemented command processing
  - `handle_command()` - Process introspection requests
  - `create_snapshot()` - Build snapshot from current state
  - `create_context_snapshot()` - Extract context data
  - `get_price_history()` - Extract window data
- [x] Added TradingEngine introspection API
  - `get_runner_snapshot(runner_id)` - Query current state (100ms timeout)
  - `get_price_history(runner_id, count)` - Query price window
  - Both return `Option<T>` for graceful failure
- [x] Added Serde serialization to RunnerStats and Context
- [x] Write 7 comprehensive tests (snapshot unit tests + integration tests)
- [x] Create completion documentation (changes/2025-12-20-phase6-state-introspection.md)

**Key Features:**
- Non-blocking queries via oneshot channels
- Concurrent command processing with market data
- Full JSON serialization for REST API
- <1ms typical latency, 100ms timeout
- Zero overhead if not used

### 6.3 HTTP Server Setup (Step 5) 📅 NEXT
- [ ] Add axum dependencies
- [ ] Create basic server with health endpoint
- [ ] Implement CORS middleware
- [ ] Add request logging

### 6.4 REST API Endpoints (Steps 6-7)
- [ ] Implement engine status endpoints:
  ```
  GET /api/engine/health
  GET /api/engine/summary
  GET /api/runners
  GET /api/runners/:id
  GET /api/runners/:id/snapshot
  GET /api/runners/:id/history
  ```
- [ ] Add runner control endpoints:
  ```
  POST /api/runners (add runner)
  DELETE /api/runners/:id (remove runner)
  ```

### 6.5 WebSocket Event Streaming (Step 8)
- [ ] Implement WebSocket handler
- [ ] Subscribe clients to event stream
- [ ] Add client-side event filtering (by runner_id)
- [ ] Implement throttling for high-frequency events

### 6.6 Integration Tests (Step 9)
- [ ] Test full HTTP/WebSocket stack
- [ ] Test concurrent WebSocket clients
- [ ] Test event filtering

### 6.7 Documentation & Demo (Step 10)
- [ ] Create web API documentation
- [ ] Build simple HTML/JavaScript dashboard demo
- [ ] Add Phase 6 completion summary

**Deliverable**: Complete web API with real-time event streaming for dashboard integration

---

## Phase 7: Event Sourcing & Replay System (Week 8-9)

### 7.1 Event Sourcing Foundation
- [ ] Design `StateMachineEvent` structure:
  ```rust
  pub struct StateMachineEvent {
      pub sequence: u64,
      pub timestamp: i64,
      pub market_data: MarketData,
      pub state_before: StateSnapshot,
      pub transition: Option<StateTransition>,
      pub action: Option<Action>,
      pub state_after: StateSnapshot,
  }
  ```
- [ ] Define `StateSnapshot` to capture context and indicators
- [ ] Implement `EventStream` for complete session recording
- [ ] Add event sequence validation

### 7.2 State Machine Event Capture
- [ ] Modify `StateMachine` to support event capture mode
- [ ] Implement `with_event_capture(session_id)` constructor
- [ ] Capture state before and after each `update()` call
- [ ] Detect and record state transitions with reasons
- [ ] Store action decisions with full context
- [ ] Add `get_event_stream()` method to retrieve recorded events

### 7.3 Event Storage
- [ ] Implement `EventLog` for sequential event storage:
  ```rust
  pub struct EventLog {
      path: PathBuf,
      writer: Option<BufWriter<File>>,
  }
  ```
- [ ] Use MessagePack + LZ4 compression for events
- [ ] Implement length-prefixed event serialization
- [ ] Add `append_event()` for real-time recording
- [ ] Implement `read_events()` for loading complete streams
- [ ] Create file structure: `data/sessions/{session_id}/events.msgpack.lz4`

### 7.4 Storage Optimization Modes
- [ ] Implement `EventStorageMode` enum (Full vs Delta)
- [ ] **Full mode**: Store complete state snapshots at each event
- [ ] **Delta mode**: Store only changed values + periodic snapshots
- [ ] Add configuration for snapshot interval in delta mode
- [ ] Implement optional indicator caching (vs recompute on replay)
- [ ] Benchmark storage sizes for different modes

### 7.5 Replay State Machine
- [ ] Implement `ReplayStateMachine` (read-only, no strategy execution):
  ```rust
  pub struct ReplayStateMachine {
      current_state: State,
      context: HashMap<String, Value>,
      sequence: u64,
  }
  ```
- [ ] Implement `apply_event()` to reconstruct state from events
- [ ] Add sequence number validation
- [ ] Implement `seek()` to jump to specific event
- [ ] Add `at_time()` for timestamp-based seeking
- [ ] Implement binary search for efficient time navigation

### 7.6 Replay Features
- [ ] Variable playback speed (1x, 5x, 10x, 100x)
- [ ] Pause/resume functionality
- [ ] Step-by-step event navigation (forward/backward)
- [ ] Time travel to specific timestamps
- [ ] Filter events by type (state changes only, actions only, etc.)
- [ ] Export replay state at any point for analysis

### 7.7 Alternative Analysis Tools
- [ ] Replay with different strategy to compare "what if" scenarios
- [ ] Replay with modified parameters
- [ ] A/B testing framework using saved event streams
- [ ] Generate comparison reports between original and alternative runs

### 7.8 Web API for Replay
- [ ] Implement REST endpoints:
  ```
  GET  /api/sessions              - List all sessions
  GET  /api/sessions/:id          - Get session metadata
  GET  /api/sessions/:id/events   - Get event stream
  GET  /api/sessions/:id/replay   - Server-Sent Events for real-time replay
  ```
- [ ] Add query parameters for speed, start/end sequence
- [ ] Implement Server-Sent Events (SSE) for streaming replay
- [ ] Add WebSocket support for interactive control (pause, seek, speed)
- [ ] Implement event filtering in API

### 7.9 Debugging Tools
- [ ] CLI tool to inspect events at specific sequence
- [ ] Pretty-print state snapshots
- [ ] Diff tool to compare state between events
- [ ] Find events matching specific criteria (e.g., "all trades entered")
- [ ] Generate timeline visualization of state transitions
- [ ] Export events to CSV/JSON for external analysis

**Deliverable**: Complete event sourcing system with replay capabilities, enabling time-travel debugging and alternative strategy analysis

---

## Phase 8: Backtesting Framework (Week 9-10)

### 8.1 Historical Data Management
- [ ] Design data storage format (Parquet or custom binary)
- [ ] Implement historical data loader for Parquet
- [ ] Implement historical data loader for CSV
- [ ] Add data validation (missing bars, outliers)
- [ ] Support multiple data sources (CSV, database, API)
- [ ] Implement tick-level vs bar-level replay
- [ ] Create data preprocessing tools (resampling, cleaning)

### 8.2 Backtest Engine
- [ ] Implement `Backtester` struct using unified runner architecture:
  ```rust
  pub struct Backtester {
      symbol: String,
      strategy_path: String,
      historical_source: HistoricalDataSource,
      start_date: DateTime<Utc>,
      end_date: DateTime<Utc>,
  }
  ```
- [ ] Use `SymbolRunner` in Historical mode internally
- [ ] Add progress tracking and status updates
- [ ] Implement configurable slippage models in `SimulatedExecution`
- [ ] Handle corporate actions (splits, dividends)
- [ ] Enable event capture during backtesting
- [ ] Generate event streams for each backtest run
- [ ] Support variable playback speed (fast-forward through quiet periods)

### 8.3 Simulated Execution Models
- [ ] Implement fill models:
  - Immediate fill (optimistic)
  - OHLC-based fill (check if price reached)
  - Slippage model (percentage or fixed)
  - Volume-based partial fills
- [ ] Add realistic commission structures
- [ ] Implement market impact modeling
- [ ] Add bid-ask spread simulation
- [ ] Handle rejected orders (insufficient funds, invalid prices)

### 8.4 Performance Metrics
- [ ] Calculate standard metrics:
  - Total return, annualized return
  - Sharpe ratio, Sortino ratio
  - Maximum drawdown, recovery time
  - Win rate, profit factor
  - Average win/loss size
- [ ] Implement equity curve plotting
- [ ] Add trade-by-trade analysis
- [ ] Generate HTML report with charts
- [ ] Use event streams for detailed trade analysis
- [ ] Compare metrics across different parameter sets

### 8.5 Strategy Optimization
- [ ] Implement parameter grid search
- [ ] Add walk-forward analysis
- [ ] Implement Monte Carlo simulation
- [ ] Add overfitting detection (train/test split)
- [ ] Create optimization report comparing parameter sets
- [ ] Save event streams for each parameter combination
- [ ] Enable replay comparison of different parameters
- [ ] Parallelize backtests across parameter combinations
- [ ] Use historical mode runners in parallel for optimization

**Deliverable**: Robust backtesting framework that uses unified runner architecture in historical mode, with comprehensive performance analysis and event capture for replay

---

## Phase 9: Configuration & Deployment (Week 10-11)

### 9.1 Configuration System
- [ ] Design TOML/YAML config format:
  ```toml
  [engine]
  max_symbols = 50
  log_level = "info"
  
  [data_source]
  type = "binance"  # or "alpaca"
  
  [risk]
  max_account_risk = 0.02
  max_position_size = 0.1
  
  [event_sourcing]
  enabled = true
  storage_mode = "full"  # or "delta"
  snapshot_interval = 100  # for delta mode
  include_indicators = false  # save space by recomputing
  
  [[strategies]]
  name = "range_breakout"
  script = "strategies/range_breakout.lua"
  symbols = ["BTCUSDT", "ETHUSDT"]
  
  [[strategies]]
  name = "ema_cross"
  script = "strategies/ema_cross.lua"
  symbols = ["AAPL", "MSFT"]
  ```
- [ ] Implement config validation
- [ ] Add config hot-reloading
- [ ] Support environment variable overrides

### 9.2 Logging & Monitoring
- [ ] Implement structured logging with `tracing`
- [ ] Add different log levels per component
- [ ] Implement log rotation
- [ ] Add performance tracing (state transitions, indicator calls)
- [ ] Create monitoring dashboard data export

### 9.3 CLI Interface
- [ ] Implement command-line interface:
  ```bash
  # Live trading
  trading-engine run --config config.toml
  trading-engine run-live --symbol BTCUSDT --strategy range.lua
  
  # Historical/Backtesting
  trading-engine backtest --strategy range.lua --symbol BTCUSDT --start 2024-01-01 --end 2024-03-01
  trading-engine backtest --config backtest.toml --speed 100
  
  # Mixed mode (live + historical simultaneously)
  trading-engine run --config mixed-mode.toml
  
  # Utilities
  trading-engine validate --strategy range.lua
  trading-engine indicators --list
  trading-engine download-data --source binance --symbol BTCUSDT --days 90
  
  # Replay
  trading-engine replay --session abc123 --speed 10
  trading-engine inspect-event --session abc123 --sequence 150
  trading-engine compare --session1 abc123 --session2 def456
  ```
- [ ] Add interactive REPL for testing strategies
- [ ] Implement status monitoring commands
- [ ] Add replay controls (play, pause, seek, speed)
- [ ] Show mode (Live/Historical) in status output
- [ ] Add progress bars for historical mode

### 9.4 Deployment Preparation
- [ ] Create Docker container
- [ ] Write deployment documentation
- [ ] Add systemd service file
- [ ] Implement graceful shutdown on SIGTERM
- [ ] Add crash recovery and restart logic

**Deliverable**: Production-ready deployment configuration

---

## Phase 10: Live Trading Integration (Week 11-12)

### 10.1 Broker API Integration
- [ ] Extend existing Binance/Alpaca sources with trading capabilities
- [ ] Implement `LiveExecution` engine for Binance:
  ```rust
  pub struct BinanceLiveExecution {
      client: BinanceClient,
      api_key: String,
      api_secret: String,
  }
  
  impl ExecutionEngine for BinanceLiveExecution {
      async fn submit_order(&mut self, order: Order) -> Result<OrderId>;
      async fn cancel_order(&mut self, order_id: OrderId) -> Result<()>;
  }
  ```
- [ ] Implement `LiveExecution` engine for Alpaca
- [ ] Handle authentication and session management
- [ ] Implement reconnection logic
- [ ] Add rate limiting to respect exchange limits
- [ ] Use unified runner architecture with Live mode

### 10.2 Real-Time Data Feed (Already Implemented)
- [ ] Verify WebSocket stability under load
- [ ] Add data quality monitoring
- [ ] Handle market hours for Alpaca (stocks)
- [ ] Add 24/7 monitoring for Binance (crypto)
- [ ] Enable event capture during live trading
- [ ] Store live trading sessions as event streams
- [ ] Ensure `next_tick()` blocking behavior is efficient

### 10.3 Live Execution Safety
- [ ] Implement order preview before submission
- [ ] Add maximum order size validation
- [ ] Implement kill switch (emergency stop all)
- [ ] Add duplicate order prevention
- [ ] Implement order reconciliation
- [ ] Add position limits enforcement
- [ ] Log all execution decisions to event stream

### 10.4 Live Monitoring
- [ ] Implement real-time position dashboard
- [ ] Add alert system (email, SMS, webhook)
- [ ] Create performance tracking dashboard
- [ ] Implement strategy health monitoring
- [ ] Add anomaly detection

**Deliverable**: Live trading capability with comprehensive safety measures

---

## Phase 11: Testing & Optimization (Week 12-13)

### 11.1 Unit Testing
- [ ] Achieve >80% code coverage for Rust core
- [ ] Test all indicator calculations vs known values
- [ ] Test state machine transitions exhaustively
- [ ] Test Lua strategy loading and error handling
- [ ] Test concurrent access patterns

### 11.2 Integration Testing
- [ ] Test end-to-end strategy execution
- [ ] Test multi-symbol coordination
- [ ] Test Binance/Alpaca API integration
- [ ] Test backtesting accuracy
- [ ] Test configuration loading
- [ ] Test event capture and replay accuracy
- [ ] Verify replay produces identical results to original run

### 11.3 Performance Optimization
- [ ] Profile indicator calculations
- [ ] Optimize hot paths with `perf` and `flamegraph`
- [ ] Reduce memory allocations
- [ ] Optimize Lua-Rust data marshalling
- [ ] Benchmark state machine throughput
- [ ] Test with extreme market data rates
- [ ] Optimize event serialization overhead

### 11.4 Stress Testing
- [ ] Test with 100+ concurrent symbols
- [ ] Simulate market data spikes
- [ ] Test memory usage under load
- [ ] Verify graceful degradation
- [ ] Test recovery from crashes

**Deliverable**: Production-quality system with verified performance

---

## Phase 12: Documentation & Polish (Week 13-14)

### 12.1 User Documentation
- [ ] Write comprehensive README
- [ ] Create "Getting Started" tutorial for Binance
- [ ] Create "Getting Started" tutorial for Alpaca
- [ ] Document strategy development guide
- [ ] Write indicator reference documentation
- [ ] Create configuration guide
- [ ] Add troubleshooting guide
- [ ] Document replay system and debugging workflows

### 12.2 Developer Documentation
- [ ] Document architecture decisions
- [ ] Create API reference for Rust components
- [ ] Document Lua strategy API
- [ ] Add contributing guidelines
- [ ] Create development setup guide
- [ ] Document event sourcing architecture
- [ ] Add replay system internals documentation

### 12.3 Example Strategies
- [ ] Create 5+ fully documented example strategies
- [ ] Add backtested performance reports for each
- [ ] Create strategy template with best practices
- [ ] Document common pitfalls
- [ ] Include replay sessions for each example

### 12.4 Final Polish
- [ ] Improve error messages
- [ ] Add input validation everywhere
- [ ] Implement helpful CLI hints
- [ ] Create demo video/screencast
- [ ] Polish UI/UX elements

**Deliverable**: Polished, well-documented system ready for users

---

## Phase 13: Advanced Features (Week 14+)

### 13.1 Advanced Strategy Features
- [ ] Multi-timeframe analysis support
- [ ] Portfolio optimization algorithms
- [ ] Machine learning integration hooks
- [ ] Sentiment analysis integration
- [ ] News-based trading signals

### 13.2 Visualization & Analytics
- [ ] Web-based dashboard (React/Svelte)
- [ ] Real-time chart visualization
- [ ] Interactive backtesting results
- [ ] Strategy comparison tools
- [ ] Advanced portfolio analytics
- [ ] **Replay Viewer UI**:
  - Visual timeline with state transitions
  - Interactive chart with trade markers
  - Scrubbing/seeking controls
  - Speed controls (1x, 5x, 10x, 100x)
  - State inspector (context, indicators at any point)
  - Side-by-side comparison of multiple sessions
  - **AI reasoning visualization** (when integrated)

### 13.3 AI Agent Integration (Advanced)

**Prerequisites**: Stable architecture, event sourcing working, sufficient historical data collected

#### 13.3.1 Contextual Memory Accumulation (Foundation)
- [ ] Extend `Context` to support agent state:
  ```rust
  pub struct CloseWatchContext {
      pub setup_reason: String,
      pub observations: Vec<Observation>,
      pub confidence_history: Vec<f64>,
      pub current_hypothesis: EntryHypothesis,
      pub alternative_scenarios: Vec<Scenario>,
      pub entry_confidence: f64,
  }
  
  pub struct Observation {
      pub timestamp: i64,
      pub observation_type: ObservationType,
      pub data: Value,
      pub confidence_impact: f64,
  }
  
  pub struct EntryHypothesis {
      pub prediction: String,
      pub supporting_evidence: Vec<String>,
      pub contradicting_evidence: Vec<String>,
      pub confidence: f64,
  }
  ```
- [ ] Implement observation tracking in CloseWatch state
- [ ] Add confidence scoring system
- [ ] Create Lua API for agent reasoning:
  ```lua
  function filter_commitment(market_data, context, indicators)
      -- Initialize hypothesis on first CloseWatch entry
      if not context.hypothesis then
          context.hypothesis = {
              prediction = "Breakout will succeed",
              supporting_evidence = {},
              contradicting_evidence = {},
              confidence = 0.5
          }
      end
      
      -- Accumulate observations
      -- Update confidence based on evidence
      -- Return entry when confidence threshold met
  end
  ```
- [ ] Implement reasoning chain serialization in events
- [ ] Test with simple rule-based agent

#### 13.3.2 Multi-Criteria Decision Matrix
- [ ] Implement `DecisionMatrix` struct:
  ```rust
  pub struct DecisionMatrix {
      pub criteria: Vec<Criterion>,
      pub weights: HashMap<String, f64>,
      pub threshold: f64,
  }
  
  pub struct Criterion {
      pub name: String,
      pub satisfied: bool,
      pub confidence: f64,
      pub observations: Vec<String>,
  }
  ```
- [ ] Add weighted evaluation system
- [ ] Create configurable criteria and weights
- [ ] Implement criterion satisfaction tracking
- [ ] Add matrix evaluation to Lua API
- [ ] Visualize decision matrix in replay viewer

#### 13.3.3 Bayesian Belief Update System
- [ ] Implement `BayesianAgent`:
  ```rust
  pub struct BayesianAgent {
      pub prior_probability: f64,
      pub likelihoods: HashMap<String, Likelihood>,
      pub posterior_probability: f64,
      pub evidence: Vec<Evidence>,
  }
  ```
- [ ] Add Bayesian update logic
- [ ] Define likelihood ratios for common observations
- [ ] Calibrate likelihoods from historical data
- [ ] Implement evidence accumulation
- [ ] Expose Bayesian reasoning to Lua
- [ ] Add belief update visualization

#### 13.3.4 Neural Network Predictor Integration
- [ ] Add ML dependencies (ONNX runtime or `tract`):
  ```toml
  [dependencies]
  tract-onnx = "0.21"
  ndarray = "0.15"
  ```
- [ ] Implement `NeuralPredictor`:
  ```rust
  pub struct NeuralPredictor {
      model: Model,
      input_window: usize,
      feature_buffer: VecDeque<Vec<f32>>,
  }
  
  pub struct Prediction {
      pub entry_quality: f64,
      pub expected_return: f64,
      pub confidence: f64,
      pub attention_weights: Vec<f64>,
      pub reasoning: Vec<String>,
  }
  ```
- [ ] Implement feature engineering pipeline:
  - Price momentum features
  - Volume features
  - Indicator features
  - Time-based features
  - Market regime features
- [ ] Load pre-trained ONNX models
- [ ] Implement online inference
- [ ] Add attention mechanism interpretation
- [ ] Generate human-readable explanations

#### 13.3.5 Model Training Pipeline
- [ ] Create training data extraction from event streams
- [ ] Label historical entries (success/failure)
- [ ] Implement feature extraction for training:
  ```python
  # Python training pipeline
  import torch
  import pandas as pd
  
  def extract_features_from_events(event_stream):
      # Extract market state at CloseWatch entry
      # Calculate subsequent trade outcome
      # Return features + labels
      pass
  ```
- [ ] Train entry quality prediction model
- [ ] Train expected return prediction model
- [ ] Implement attention-based explainable architecture
- [ ] Export models to ONNX format
- [ ] Add model versioning and A/B testing

#### 13.3.6 Hybrid Agent (Rules + ML)
- [ ] Implement `HybridAgent`:
  ```rust
  pub struct HybridAgent {
      rule_engine: RuleEngine,
      ml_scorer: NeuralPredictor,
      thoughts: Vec<Thought>,
  }
  
  pub struct Thought {
      pub timestamp: i64,
      pub rule_evaluation: RuleResult,
      pub ml_score: f64,
      pub combined_confidence: f64,
      pub explanation: String,
  }
  ```
- [ ] Combine rule-based and ML predictions
- [ ] Weight combination (configurable rule vs ML balance)
- [ ] Implement reasoning chain tracking
- [ ] Add explanation aggregation
- [ ] Test hybrid agent vs pure rules vs pure ML

#### 13.3.7 Agent State Integration
- [ ] Integrate agent state into `StateMachine`:
  ```rust
  pub struct StateMachine {
      // ... existing fields
      ai_agent: Option<Box<dyn Agent>>,
      prediction_history: Vec<Prediction>,
  }
  ```
- [ ] Add agent update in CloseWatch state
- [ ] Store agent predictions in context
- [ ] Serialize agent reasoning in event stream
- [ ] Enable agent state recovery from events

#### 13.3.8 Lua API for Agent Interaction
- [ ] Expose agent predictions to Lua:
  ```lua
  function filter_commitment(market_data, context, indicators)
      -- Access agent's current thoughts
      if ai_agent then
          context.ai_thought = ai_agent.current_hypothesis
          context.ai_confidence = ai_agent.confidence
          
          -- Combine with traditional logic
          if rules_satisfied() and ai_agent.confidence > 0.8 then
              return enter_trade(ai_agent.reasoning)
          end
      end
  end
  ```
- [ ] Add agent control functions
- [ ] Implement agent parameter tuning
- [ ] Add real-time agent state inspection

#### 13.3.9 Agent Performance Analysis
- [ ] Track agent prediction accuracy
- [ ] Measure calibration (are 80% confidence predictions right 80% of time?)
- [ ] Compare agent decisions vs actual outcomes
- [ ] Generate agent performance reports
- [ ] Implement continuous learning pipeline
- [ ] Add agent A/B testing framework

#### 13.3.10 Visualization & Debugging
- [ ] Replay viewer shows agent reasoning:
  - Hypothesis evolution over time
  - Confidence trajectory
  - Evidence accumulation
  - Decision matrix state
  - ML attention heatmaps
- [ ] Add agent thought timeline
- [ ] Visualize reasoning chains
- [ ] Compare agent reasoning between successful/failed trades
- [ ] Create "agent transparency report"

#### 13.3.11 Configuration & Deployment
- [ ] Add agent configuration:
  ```toml
  [agent]
  enabled = true
  type = "hybrid"  # or "rules", "bayesian", "neural", "hybrid"
  
  [agent.neural]
  model_path = "models/entry_predictor_v1.onnx"
  confidence_threshold = 0.8
  
  [agent.hybrid]
  rule_weight = 0.6
  ml_weight = 0.4
  explanation_required = true
  ```
- [ ] Implement agent versioning
- [ ] Add model hot-swapping
- [ ] Create agent rollback mechanism
- [ ] Monitor agent performance in production

**Deliverable**: Optional AI agent system that operates in CloseWatch state to accumulate observations, reason about entry quality, and provide explainable predictions. Fully integrated with event sourcing for analysis and continuous improvement.

### 13.4 Distributed Computing
- [ ] Support for distributed backtesting
- [ ] Cloud deployment (AWS/GCP)
- [ ] Horizontal scaling for live trading
- [ ] Message queue integration (Kafka/RabbitMQ)

### 13.4 Distributed Computing
- [ ] Support for distributed backtesting
- [ ] Cloud deployment (AWS/GCP)
- [ ] Horizontal scaling for live trading
- [ ] Message queue integration (Kafka/RabbitMQ)
- [ ] Distributed agent training pipeline

### 13.5 Community & Ecosystem
- [ ] Create plugin system for custom indicators
- [ ] Build strategy marketplace
- [ ] Add community strategy sharing
- [ ] Create Discord/forum for users
- [ ] Share replay sessions for educational purposes
- [ ] Agent model sharing and leaderboards

---

## Unified Runner Architecture Benefits

The **Live/Historical unified runner architecture** provides several critical advantages:

### 1. **Mechanical Equivalence**
- Same `SymbolRunner` code executes in both modes
- Same state machine logic
- Same strategy execution
- Only difference: `next_tick()` blocks (live) vs returns immediately (historical)

### 2. **Testing Confidence**
- Test strategies in historical mode with 100% confidence they'll work the same in live
- No "backtesting doesn't match live" discrepancies
- Identical code paths eliminate surprises

### 3. **Development Workflow**
- Develop strategies using historical data (fast iteration)
- Switch to paper trading (live mode, simulated execution)
- Deploy to production (live mode, real execution)
- All using the same runner infrastructure

### 4. **Mixed Mode Operation**
- Run live trading on some symbols while backtesting others
- Use same monitoring infrastructure
- Share indicator calculations and state machine logic

### 5. **Debugging**
- Event sourcing works identically in both modes
- Can replay live sessions in historical mode
- Use historical mode to reproduce live issues

### 6. **Performance Optimization**
- Historical mode can run at 10x, 100x, or unlimited speed
- Live mode respects real-time constraints
- Same optimization benefits both modes

---

## Success Metrics

### Technical Metrics
- [ ] State machine handles 1000+ state transitions/second
- [ ] Support 100+ concurrent symbols with <1% CPU per symbol
- [ ] Indicator calculations <1ms for typical periods
- [ ] End-to-end latency <10ms from data → action
- [ ] Memory usage <100MB per symbol
- [ ] Binance WebSocket maintains <100ms latency
- [ ] Alpaca streaming maintains stable connection during market hours

### Quality Metrics
- [ ] >80% unit test coverage
- [ ] Zero critical bugs in production
- [ ] <5 minute recovery time from crashes
- [ ] Backtesting results within 5% of live trading

### User Metrics
- [ ] Strategy development time <1 hour
- [ ] Documentation completeness score >90%
- [ ] User can deploy in <30 minutes
- [ ] Can connect to Binance in <5 minutes
- [ ] Can connect to Alpaca paper trading in <10 minutes

### AI Agent Metrics (Phase 13.3)
- [ ] Agent prediction accuracy >65% (better than random)
- [ ] Agent calibration error <10% (confidence matches reality)
- [ ] Explainability: 100% of decisions have reasoning chains
- [ ] Latency: Agent inference <50ms per update
- [ ] Agent improves win rate by >5% vs pure rules

---

## Risk Mitigation

### Technical Risks
- **Risk**: OCaml FFI performance bottleneck
  - **Mitigation**: Benchmark early, have Rust fallback implementations
  
- **Risk**: Lua strategy sandboxing insufficient
  - **Mitigation**: Use mlua sandboxing features, implement resource limits

- **Risk**: Threading bugs causing data corruption
  - **Mitigation**: Extensive testing, use Rust's type system, formal verification tools

- **Risk**: Binance/Alpaca API changes breaking integration
  - **Mitigation**: Version lock dependencies, monitor API changelogs, implement abstraction layer

- **Risk**: WebSocket disconnections during critical trading moments
  - **Mitigation**: Robust reconnection logic, duplicate connection monitoring, failover strategies

### Project Risks
- **Risk**: Scope creep
  - **Mitigation**: Stick to roadmap, defer advanced features to Phase 13

- **Risk**: Underestimated complexity
  - **Mitigation**: Build Phase 1 (data sources) and Phase 2 (indicators) in parallel to validate approach

- **Risk**: Exchange rate limits
  - **Mitigation**: Implement proper rate limiting, use WebSocket instead of polling, add backoff strategies

- **Risk**: Historical mode doesn't accurately represent live behavior
  - **Mitigation**: Unified runner architecture ensures mechanical equivalence. Differences only in execution simulation (slippage, fills). Validate with paper trading before live deployment.

- **Risk**: Performance degradation with event capture enabled
  - **Mitigation**: Benchmark early, make event capture optional, optimize serialization, use delta mode for storage efficiency

- **Risk**: AI agent makes poor predictions leading to losses (Phase 13.3)
  - **Mitigation**: Start with hybrid approach (rules + ML). Require minimum confidence thresholds. Extensive backtesting before live deployment. Implement kill switches for agent underperformance. Always maintain human override capability.

- **Risk**: AI agent becomes a "black box" without interpretability
  - **Mitigation**: Use attention mechanisms for explainability. Require reasoning chains for all decisions. Implement agent transparency reports. Compare agent decisions with rule-based baseline. Store all agent state in event streams for post-analysis.

- **Risk**: Model drift as market conditions change
  - **Mitigation**: Continuous monitoring of agent prediction accuracy. Regular retraining on recent data. A/B test new models before deployment. Implement online learning where appropriate. Track calibration metrics.

---

## Market Data Source Details

### Binance Setup (Crypto - Free)
1. **No Account Required** for market data (only for trading)
2. **WebSocket Endpoints**:
   - Production: `wss://stream.binance.com:9443/ws/`
   - Testnet: `wss://testnet.binance.vision/ws/`
3. **Popular Pairs**: BTCUSDT, ETHUSDT, BNBUSDT, SOLUSDT, ADAUSDT
4. **Intervals**: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d
5. **Documentation**: https://binance-docs.github.io/apidocs/spot/en/

### Alpaca Setup (Stocks - Free Paper Trading)
1. **Create Account**: https://alpaca.markets (paper trading free)
2. **Get API Keys**: Dashboard → Your API Keys
3. **Endpoints**:
   - Paper: `https://paper-api.alpaca.markets`
   - Data: `https://data.alpaca.markets`
4. **Market Data**: Real-time bars, trades, quotes
5. **Popular Symbols**: AAPL, MSFT, GOOGL, AMZN, TSLA, SPY, QQQ
6. **Documentation**: https://docs.alpaca.markets/

### Data Comparison
| Feature | Binance | Alpaca |
|---------|---------|--------|
| Asset Class | Crypto | US Stocks |
| Cost | Free | Free (paper) |
| Market Hours | 24/7 | 9:30am-4pm EST |
| Latency | <50ms | <100ms |
| Data Quality | Excellent | Excellent |
| Best For | Crypto strategies, 24/7 testing | Stock strategies, US market |

---

## Tooling & Infrastructure

### Development Tools
- IDE: VSCode with rust-analyzer, OCaml LSP
- Debugging: `gdb`, `lldb`, OCaml debugger
- Profiling: `perf`, `valgrind`, `flamegraph`
- Testing: `cargo test`, `dune test`, integration test framework

### Build & CI
- Rust: `cargo` with `cargo-watch` for development
- OCaml: `dune` build system
- CI: GitHub Actions with matrix builds
- Release: `cargo-release` for versioning

### Deployment
- Containerization: Docker multi-stage builds
- Orchestration: Docker Compose for development, Kubernetes for production
- Monitoring: Prometheus + Grafana
- Logging: ELK stack or Loki

---

## Timeline Summary

| Phase | Duration | Key Deliverable |
|-------|----------|-----------------|
| 0 | 1 week | Project setup |
| 1 | 2 weeks | Market data infrastructure + Binance/Alpaca integration |
| 2 | 2 weeks | OCaml indicators with FFI |
| 3 | 2 weeks | State machine core |
| 4 | 2 weeks | Lua strategy integration |
| 5 | 2 weeks | Multi-symbol threading |
| 6 | 2 weeks | Execution & risk management |
| 7 | 2 weeks | Event sourcing & replay system |
| 8 | 2 weeks | Backtesting framework |
| 9 | 2 weeks | Configuration & deployment |
| 10 | 2 weeks | Live trading integration |
| 11 | 2 weeks | Testing & optimization |
| 12 | 2 weeks | Documentation |
| **Total** | **~3.5 months** | **Production system** |

---

## Next Steps

1. **Week 1**: Set up development environment, create project structure
2. **Week 2**: Begin Phase 1 with Binance WebSocket integration - this will validate the architecture quickly with real data
3. **Week 2-3**: Add Alpaca integration in parallel, begin Phase 2 (Indicators)
4. **Week 3**: Validate OCaml FFI approach with performance benchmarks using real market data
5. **Week 4**: Complete state machine core and test with live Binance data streams
6. **Week 5**: Begin Lua integration and test strategies on crypto markets (24/7 availability helps development)
7. **Week 6**: Review progress, adjust timeline if needed
8. **Week 8**: Implement event sourcing - critical for debugging and analysis

**Critical Path**: Phase 1 (Market Data) is now the foundation - getting Binance and Alpaca working early enables all subsequent testing and development with real market data rather than simulated data. Phase 7 (Event Sourcing) becomes crucial for debugging and performance analysis. Phase 13.3 (AI Agent) should only be attempted after the core system is stable and producing reliable event streams for training data.

**Development Philosophy**: Build a solid, reliable foundation first. The AI agent integration in Phase 13.3 is powerful but optional - the system should work excellently with rule-based strategies before adding ML complexity. Event sourcing throughout the project enables later AI integration by providing high-quality training data.

**Start Date**: [Your start date]
**Target MVP Date**: [+8 weeks]
**Target Production**: [+14 weeks]
**Target AI Integration**: [+16+ weeks] (after core system proven stable)
