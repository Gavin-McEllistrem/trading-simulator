# Trading System Roadmap

## Project Overview
Build a high-performance, multi-threaded trading system with:
- **Rust core engine** for state machine and concurrency
- **Lua scripting** for strategy definitions
- **OCaml** for pure functional indicators
- Multi-symbol support with per-symbol threading

---

## Phase 0: Project Setup & Foundation (Week 1)

### 0.1 Development Environment
- [ ] Set up Rust project with `cargo new trading-engine`
- [ ] Configure OCaml environment with `opam` and `dune`
- [ ] Set up Lua development environment
- [ ] Create workspace structure:
  ```
  trading-engine/
  ├── rust-core/          # Main engine
  ├── ocaml-indicators/   # Indicator library
  ├── lua-strategies/     # Strategy scripts
  ├── tests/             # Integration tests
  └── docs/              # Documentation
  ```

### 0.2 Core Dependencies
- [ ] Add Rust dependencies to `Cargo.toml`:
  - `tokio` (async runtime)
  - `mlua` (Lua embedding)
  - `serde` (serialization)
  - `chrono` (time handling)
  - `crossbeam` (concurrent data structures)
  - `tracing` (logging)
  - `tokio-tungstenite` (WebSocket for market data)
  - `async-trait` (async traits)
- [ ] Set up OCaml `dune` project
- [ ] Configure FFI bridge between Rust and OCaml

### 0.3 Version Control & CI
- [ ] Initialize git repository
- [ ] Create `.gitignore` for Rust, OCaml, Lua
- [ ] Set up GitHub Actions for CI:
  - Rust tests and clippy
  - OCaml compilation and tests
  - Integration tests

---

## Phase 1: Market Data Infrastructure (Week 2-3)

### 1.1 Core Data Structures
- [ ] Define `MarketData` struct in Rust:
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
- [ ] Implement `MarketDataWindow` with circular buffer
- [ ] Add time-series query methods (`high()`, `low()`, `avg_volume()`)
- [ ] Write comprehensive unit tests

### 1.2 Market Data Source Abstraction
- [ ] Design abstract `MarketDataSource` trait:
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
- [ ] Implement `SimulatedFeed` for testing (replays historical data)
- [ ] Implement `CSVFeed` for backtesting from files
- [ ] Add thread-safe data distribution to symbol runners

### 1.3 Binance WebSocket Integration (Primary Crypto Source)
- [ ] Add dependencies: `tokio-tungstenite`, `serde_json`, `url`
- [ ] Create Binance account (no funding needed for market data)
- [ ] Implement `BinanceSource` struct with WebSocket connection:
  ```rust
  pub struct BinanceSource {
      ws_stream: Option<WebSocketStream>,
      subscribed_symbols: Vec<String>,
      reconnect_attempts: u32,
  }
  ```
- [ ] Parse Binance kline/candlestick events:
  ```rust
  #[derive(Debug, Deserialize)]
  struct BinanceKline {
      #[serde(rename = "t")] open_time: i64,
      #[serde(rename = "o")] open: String,
      #[serde(rename = "h")] high: String,
      #[serde(rename = "l")] low: String,
      #[serde(rename = "c")] close: String,
      #[serde(rename = "v")] volume: String,
  }
  ```
- [ ] Convert Binance string prices to f64 in `MarketData` format
- [ ] Implement reconnection logic with exponential backoff
- [ ] Add connection health monitoring (ping/pong)
- [ ] Handle Binance rate limits and errors gracefully
- [ ] Test with BTC-USDT, ETH-USDT, and other major pairs
- [ ] Support multiple timeframes (1m, 5m, 15m, 1h)
- [ ] Add WebSocket URL construction: `wss://stream.binance.com:9443/ws/{symbol}@kline_{interval}`

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

### 1.6 Data Source Configuration
- [ ] Create configuration system for selecting data source:
  ```toml
  [data_source]
  type = "binance"  # or "alpaca", "coinbase", "simulated"
  
  [data_source.binance]
  # Binance-specific config
  
  [data_source.alpaca]
  api_key_env = "APCA_API_KEY_ID"
  secret_key_env = "APCA_API_SECRET_KEY"
  ```
- [ ] Implement data source factory pattern
- [ ] Add validation for required credentials per source

### 1.7 Data Storage
- [ ] Create in-memory storage with `Arc<RwLock<HashMap>>`
- [ ] Implement windowed storage (only keep last N datapoints)
- [ ] Add serialization for market data snapshots
- [ ] Test concurrent read/write performance
- [ ] Add optional persistence to disk for replay

### 1.8 Historical Data Download
- [ ] Implement Binance REST API client for historical klines
- [ ] Implement Alpaca REST API for historical bars
- [ ] Create tools for downloading and caching historical data
- [ ] Store historical data in efficient format (Parquet or compressed CSV)

**Deliverable**: Market data can be ingested from multiple sources (Binance, Alpaca), stored, and queried efficiently. Full support for both crypto (24/7) and stock (market hours) data sources.

---

## Phase 2: OCaml Indicator Library (Week 3-4)

### 2.1 Core Indicator Framework
- [ ] Set up OCaml project structure:
  ```
  ocaml-indicators/
  ├── src/
  │   ├── indicators.mli   # Public interface
  │   ├── indicators.ml    # Implementation
  │   └── ffi.ml          # C FFI exports
  ├── dune               # Build config
  └── test/
  ```
- [ ] Define `price_data` type
- [ ] Implement helper functions (average, std_dev, sliding_window)

### 2.2 Basic Indicators
- [ ] Implement SMA (Simple Moving Average)
- [ ] Implement EMA (Exponential Moving Average)
- [ ] Implement RSI (Relative Strength Index)
- [ ] Write property-based tests with `QCheck`

### 2.3 Advanced Indicators
- [ ] Implement MACD (Moving Average Convergence Divergence)
- [ ] Implement Bollinger Bands
- [ ] Implement ATR (Average True Range)
- [ ] Implement VWAP (Volume Weighted Average Price)
- [ ] Implement Stochastic Oscillator

### 2.4 FFI Bridge to Rust
- [ ] Create C-compatible exports from OCaml
- [ ] Write Rust FFI bindings using `extern "C"`
- [ ] Implement `IndicatorEngine` wrapper in Rust:
  ```rust
  pub struct IndicatorEngine {
      data_window: MarketDataWindow,
  }
  
  impl IndicatorEngine {
      pub fn ema(&self, period: usize) -> Vec<f64>;
      pub fn rsi(&self, period: usize) -> Vec<f64>;
      pub fn macd(&self, fast: usize, slow: usize, signal: usize) 
          -> (Vec<f64>, Vec<f64>, Vec<f64>);
  }
  ```
- [ ] Test data marshalling performance
- [ ] Add benchmarks comparing OCaml vs native Rust implementations

**Deliverable**: OCaml indicator library callable from Rust with verified correctness

---

## Phase 3: State Machine Core (Week 4-5)

### 3.1 State Machine Foundation
- [ ] Define `State` enum (Watch, CloseWatch, InTrade)
- [ ] Define `Context` struct with flexible data storage
- [ ] Define `Action` enum (EnterTrade, ExitTrade, UpdateStop, etc.)
- [ ] Implement basic state transitions with logging

### 3.2 Strategy Trait Definition
- [ ] Define `Strategy` trait:
  ```rust
  pub trait Strategy: Send + Sync {
      fn detect_opportunity(
          &self,
          market_data: &MarketData,
          context: &Context,
          indicators: &IndicatorEngine,
      ) -> Option<HashMap<String, Value>>;
      
      fn filter_commitment(
          &self,
          market_data: &MarketData,
          context: &Context,
          indicators: &IndicatorEngine,
      ) -> Option<Action>;
      
      fn manage_position(
          &self,
          market_data: &MarketData,
          context: &Context,
          indicators: &IndicatorEngine,
      ) -> Option<Action>;
  }
  ```
- [ ] Implement basic `MockStrategy` for testing

### 3.3 State Machine Implementation
- [ ] Implement `StateMachine` struct
- [ ] Implement `update()` method with state transition logic
- [ ] Add dynamic update frequency based on state
- [ ] Implement timeout mechanisms for CloseWatch state
- [ ] Add state transition history tracking
- [ ] Write extensive state transition tests

### 3.4 Context Management
- [ ] Implement flexible context storage using `HashMap<String, Value>`
- [ ] Add helper methods for common context operations
- [ ] Implement context serialization for debugging
- [ ] Test context preservation across state transitions

**Deliverable**: Working state machine that can be driven by mock strategies

---

## Phase 4: Lua Strategy Integration (Week 5-6)

### 4.1 Lua Embedding
- [ ] Set up `mlua` runtime management
- [ ] Implement Lua VM pool for multi-strategy execution
- [ ] Define Lua API for market data access:
  ```lua
  -- market_data table available in Lua
  market_data.price
  market_data.volume
  market_data.timestamp
  market_data.time_since_open()
  ```

### 4.2 Strategy Loading System
- [ ] Implement `LuaStrategy` struct:
  ```rust
  pub struct LuaStrategy {
      vm: Lua,
      strategy_name: String,
  }
  ```
- [ ] Load Lua scripts from filesystem
- [ ] Validate required functions exist (detect_opportunity, etc.)
- [ ] Add hot-reloading capability for development
- [ ] Implement error handling and sandboxing

### 4.3 Lua-Rust Type Conversion
- [ ] Implement `MarketData` → Lua table conversion
- [ ] Implement `Context` → Lua table conversion
- [ ] Implement `IndicatorEngine` wrapper for Lua:
  ```lua
  indicators.ema(20)
  indicators.rsi(14)
  indicators.macd(12, 26, 9)
  ```
- [ ] Implement Lua return values → Rust types
- [ ] Add type validation and error messages

### 4.4 Example Strategy Library
- [ ] Create template strategy script
- [ ] Implement range breakout strategy in Lua
- [ ] Implement EMA crossover strategy
- [ ] Implement RSI mean reversion strategy
- [ ] Document strategy API thoroughly

**Deliverable**: Lua strategies can be loaded and executed within state machine

---

## Phase 5: Multi-Symbol Threading Engine (Week 6-7)

### 5.1 Symbol Runner Architecture
- [ ] Implement `SymbolRunner` struct:
  ```rust
  struct SymbolRunner {
      symbol: String,
      state_machine: StateMachine,
      data_feed: Arc<RwLock<HashMap<String, MarketDataWindow>>>,
  }
  ```
- [ ] Implement main event loop with dynamic sleeping
- [ ] Add graceful shutdown handling
- [ ] Implement panic recovery per symbol

### 5.2 Trading Engine Core
- [ ] Implement `TradingEngine` with async Tokio runtime
- [ ] Add symbol registration: `add_symbol(symbol, strategy_path)`
- [ ] Implement symbol removal and cleanup
- [ ] Add engine-level configuration (max symbols, thread pool size)
- [ ] Implement health monitoring for symbol runners

### 5.3 Concurrency & Synchronization
- [ ] Implement thread-safe market data distribution
- [ ] Add action queue for trade execution
- [ ] Implement cross-symbol coordination (if needed)
- [ ] Add deadlock detection
- [ ] Benchmark with 10, 50, 100+ concurrent symbols

### 5.4 Resource Management
- [ ] Implement memory limits per symbol
- [ ] Add CPU usage monitoring
- [ ] Implement automatic throttling under load
- [ ] Add metrics collection (state changes/sec, indicator calls, etc.)

**Deliverable**: Multiple symbols running concurrently with isolated strategies

---

## Phase 6: Execution & Risk Management (Week 7-8)

### 6.1 Order Types & Execution
- [ ] Define `Order` types (Market, Limit, Stop, StopLimit)
- [ ] Define `Position` struct tracking entry, stops, targets
- [ ] Implement `ExecutionEngine` trait:
  ```rust
  pub trait ExecutionEngine {
      async fn submit_order(&mut self, order: Order) -> Result<OrderId>;
      async fn cancel_order(&mut self, order_id: OrderId) -> Result<()>;
      async fn get_position(&self, symbol: &str) -> Option<Position>;
  }
  ```
- [ ] Implement `SimulatedExecution` for backtesting
- [ ] Add slippage and commission modeling

### 6.2 Position Management
- [ ] Implement position tracking per symbol
- [ ] Add P&L calculation (realized and unrealized)
- [ ] Implement stop-loss management (initial, trailing, breakeven)
- [ ] Add take-profit handling
- [ ] Implement partial position exits

### 6.3 Risk Management Layer
- [ ] Implement per-trade risk calculation
- [ ] Add position sizing based on stop distance
- [ ] Implement account-level risk limits (max drawdown, daily loss limit)
- [ ] Add correlation-based exposure management
- [ ] Implement maximum position count limits
- [ ] Add risk reporting and alerting

### 6.4 Portfolio Management
- [ ] Track portfolio-level P&L
- [ ] Implement equity curve calculation
- [ ] Add portfolio statistics (Sharpe ratio, max drawdown, win rate)
- [ ] Create portfolio rebalancing hooks

**Deliverable**: Complete order execution with comprehensive risk management

---

## Phase 7: Backtesting Framework (Week 8-9)

### 7.1 Historical Data Management
- [ ] Design data storage format (Parquet or custom binary)
- [ ] Implement historical data loader
- [ ] Add data validation (missing bars, outliers)
- [ ] Support multiple data sources (CSV, database, API)
- [ ] Implement tick-level vs bar-level replay

### 7.2 Backtest Engine
- [ ] Implement `Backtester` struct:
  ```rust
  pub struct Backtester {
      engine: TradingEngine,
      historical_feed: HistoricalDataFeed,
      start_date: DateTime<Utc>,
      end_date: DateTime<Utc>,
  }
  ```
- [ ] Add time-travel mechanics (fast-forward through history)
- [ ] Implement realistic order filling simulation
- [ ] Add configurable slippage models
- [ ] Handle corporate actions (splits, dividends)

### 7.3 Performance Metrics
- [ ] Calculate standard metrics:
  - Total return, annualized return
  - Sharpe ratio, Sortino ratio
  - Maximum drawdown, recovery time
  - Win rate, profit factor
  - Average win/loss size
- [ ] Implement equity curve plotting
- [ ] Add trade-by-trade analysis
- [ ] Generate HTML report with charts

### 7.4 Strategy Optimization
- [ ] Implement parameter grid search
- [ ] Add walk-forward analysis
- [ ] Implement Monte Carlo simulation
- [ ] Add overfitting detection (train/test split)
- [ ] Create optimization report comparing parameter sets

**Deliverable**: Robust backtesting with comprehensive performance analysis

---

## Phase 8: Configuration & Deployment (Week 9-10)

### 8.1 Configuration System
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

### 8.2 Logging & Monitoring
- [ ] Implement structured logging with `tracing`
- [ ] Add different log levels per component
- [ ] Implement log rotation
- [ ] Add performance tracing (state transitions, indicator calls)
- [ ] Create monitoring dashboard data export

### 8.3 CLI Interface
- [ ] Implement command-line interface:
  ```bash
  trading-engine run --config config.toml
  trading-engine backtest --strategy range.lua --symbol BTCUSDT --start 2024-01-01
  trading-engine validate --strategy range.lua
  trading-engine indicators --list
  trading-engine download-data --source binance --symbol BTCUSDT --days 90
  ```
- [ ] Add interactive REPL for testing strategies
- [ ] Implement status monitoring commands

### 8.4 Deployment Preparation
- [ ] Create Docker container
- [ ] Write deployment documentation
- [ ] Add systemd service file
- [ ] Implement graceful shutdown on SIGTERM
- [ ] Add crash recovery and restart logic

**Deliverable**: Production-ready deployment configuration

---

## Phase 9: Live Trading Integration (Week 10-11)

### 9.1 Broker API Integration
- [ ] Extend existing Binance/Alpaca sources with trading capabilities
- [ ] Implement order placement for Binance:
  ```rust
  impl BinanceSource {
      async fn place_order(&mut self, order: Order) -> Result<OrderId>;
      async fn cancel_order(&mut self, order_id: OrderId) -> Result<()>;
  }
  ```
- [ ] Implement order placement for Alpaca
- [ ] Handle authentication and session management
- [ ] Implement reconnection logic
- [ ] Add rate limiting to respect exchange limits

### 9.2 Real-Time Data Feed (Already Implemented)
- [ ] Verify WebSocket stability under load
- [ ] Add data quality monitoring
- [ ] Handle market hours for Alpaca (stocks)
- [ ] Add 24/7 monitoring for Binance (crypto)

### 9.3 Live Execution Safety
- [ ] Implement order preview before submission
- [ ] Add maximum order size validation
- [ ] Implement kill switch (emergency stop all)
- [ ] Add duplicate order prevention
- [ ] Implement order reconciliation
- [ ] Add position limits enforcement

### 9.4 Live Monitoring
- [ ] Implement real-time position dashboard
- [ ] Add alert system (email, SMS, webhook)
- [ ] Create performance tracking dashboard
- [ ] Implement strategy health monitoring
- [ ] Add anomaly detection

**Deliverable**: Live trading capability with comprehensive safety measures

---

## Phase 10: Testing & Optimization (Week 11-12)

### 10.1 Unit Testing
- [ ] Achieve >80% code coverage for Rust core
- [ ] Test all indicator calculations vs known values
- [ ] Test state machine transitions exhaustively
- [ ] Test Lua strategy loading and error handling
- [ ] Test concurrent access patterns

### 10.2 Integration Testing
- [ ] Test end-to-end strategy execution
- [ ] Test multi-symbol coordination
- [ ] Test Binance/Alpaca API integration
- [ ] Test backtesting accuracy
- [ ] Test configuration loading

### 10.3 Performance Optimization
- [ ] Profile indicator calculations
- [ ] Optimize hot paths with `perf` and `flamegraph`
- [ ] Reduce memory allocations
- [ ] Optimize Lua-Rust data marshalling
- [ ] Benchmark state machine throughput
- [ ] Test with extreme market data rates

### 10.4 Stress Testing
- [ ] Test with 100+ concurrent symbols
- [ ] Simulate market data spikes
- [ ] Test memory usage under load
- [ ] Verify graceful degradation
- [ ] Test recovery from crashes

**Deliverable**: Production-quality system with verified performance

---

## Phase 11: Documentation & Polish (Week 12-13)

### 11.1 User Documentation
- [ ] Write comprehensive README
- [ ] Create "Getting Started" tutorial for Binance
- [ ] Create "Getting Started" tutorial for Alpaca
- [ ] Document strategy development guide
- [ ] Write indicator reference documentation
- [ ] Create configuration guide
- [ ] Add troubleshooting guide

### 11.2 Developer Documentation
- [ ] Document architecture decisions
- [ ] Create API reference for Rust components
- [ ] Document Lua strategy API
- [ ] Add contributing guidelines
- [ ] Create development setup guide

### 11.3 Example Strategies
- [ ] Create 5+ fully documented example strategies
- [ ] Add backtested performance reports for each
- [ ] Create strategy template with best practices
- [ ] Document common pitfalls

### 11.4 Final Polish
- [ ] Improve error messages
- [ ] Add input validation everywhere
- [ ] Implement helpful CLI hints
- [ ] Create demo video/screencast
- [ ] Polish UI/UX elements

**Deliverable**: Polished, well-documented system ready for users

---

## Phase 12: Advanced Features (Week 13+)

### 12.1 Advanced Strategy Features
- [ ] Multi-timeframe analysis support
- [ ] Portfolio optimization algorithms
- [ ] Machine learning integration hooks
- [ ] Sentiment analysis integration
- [ ] News-based trading signals

### 12.2 Visualization & Analytics
- [ ] Web-based dashboard (React/Svelte)
- [ ] Real-time chart visualization
- [ ] Interactive backtesting results
- [ ] Strategy comparison tools
- [ ] Advanced portfolio analytics

### 12.3 Distributed Computing
- [ ] Support for distributed backtesting
- [ ] Cloud deployment (AWS/GCP)
- [ ] Horizontal scaling for live trading
- [ ] Message queue integration (Kafka/RabbitMQ)

### 12.4 Community & Ecosystem
- [ ] Create plugin system for custom indicators
- [ ] Build strategy marketplace
- [ ] Add community strategy sharing
- [ ] Create Discord/forum for users

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
  - **Mitigation**: Stick to roadmap, defer advanced features to Phase 12

- **Risk**: Underestimated complexity
  - **Mitigation**: Build Phase 1 (data sources) and Phase 2 (indicators) in parallel to validate approach

- **Risk**: Exchange rate limits
  - **Mitigation**: Implement proper rate limiting, use WebSocket instead of polling, add backoff strategies

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
| 7 | 2 weeks | Backtesting framework |
| 8 | 2 weeks | Configuration & deployment |
| 9 | 2 weeks | Live trading integration |
| 10 | 2 weeks | Testing & optimization |
| 11 | 2 weeks | Documentation |
| **Total** | **~3 months** | **Production system** |

---

## Next Steps

1. **Week 1**: Set up development environment, create project structure
2. **Week 2**: Begin Phase 1 with Binance WebSocket integration - this will validate the architecture quickly with real data
3. **Week 2-3**: Add Alpaca integration in parallel, begin Phase 2 (Indicators)
4. **Week 3**: Validate OCaml FFI approach with performance benchmarks using real market data
5. **Week 4**: Complete state machine core and test with live Binance data streams
6. **Week 5**: Begin Lua integration and test strategies on crypto markets (24/7 availability helps development)
7. **Week 6**: Review progress, adjust timeline if needed

**Critical Path**: Phase 1 (Market Data) is now the foundation - getting Binance and Alpaca working early enables all subsequent testing and development with real market data rather than simulated data.

**Start Date**: [Your start date]
**Target MVP Date**: [+8 weeks]
**Target Production**: [+13 weeks]
