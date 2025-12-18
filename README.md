# Trading Simulator

A high-performance, multi-threaded trading system with Rust core engine, OCaml indicators, and Lua strategy scripting.

## Project Status

**Current Phase:** 2 (Technical Indicators) ✅ **COMPLETE**

### Completed (Phase 0-2)
- ✅ Project structure and build system
- ✅ Core data structures (MarketData, MarketDataWindow)
- ✅ Data source abstraction (MarketDataSource trait)
- ✅ Simulated feed for testing
- ✅ Thread-safe storage system
- ✅ Configuration framework
- ✅ Error handling
- ✅ **Binance WebSocket integration** 🎉
  - Real-time kline (OHLCV) data
  - Live bid/ask prices via bookTicker
  - Support for Binance.com and Binance.US
  - Multiple symbols simultaneously
  - Automatic ping/pong keepalive
- ✅ **Technical Indicators (Dual Rust/OCaml)** 🎉
  - SMA, EMA (Moving Averages)
  - RSI (Relative Strength Index)
  - MACD (Moving Average Convergence Divergence)
  - Bollinger Bands
  - OCaml reference implementation via subprocess
  - Full verification suite (41 tests passing)
- ✅ Comprehensive documentation

### Next Steps
- 📅 State machine (Phase 3)
- 📅 Lua strategies (Phase 4)

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
│   │   ├── sources/      # Data source implementations
│   │   ├── storage/      # Thread-safe storage
│   │   ├── config/       # Configuration types
│   │   ├── error.rs      # Error handling
│   │   ├── lib.rs        # Library root
│   │   └── main.rs       # Binary entry point
│   ├── tests/            # Integration tests
│   └── Cargo.toml
├── ocaml-indicators/     # Indicator library (Phase 2)
├── lua-strategies/       # Strategy scripts (Phase 4)
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

### Technical Documentation
- **[Architecture Overview](docs/architecture/01-overview.md)** - System design
- **[API Documentation](engine-core/target/doc/trading_engine/index.html)** - Generated from code (`cargo doc --open`)
- **[Full Roadmap](trading-system-roadmap.md)** - Complete project plan

### Architecture Decision Records (ADRs)
- [ADR 001: Rust Edition 2021](docs/decisions/001-rust-2021.md)
- [ADR 002: engine-core Naming](docs/decisions/002-engine-core-naming.md)
- [ADR 003: Circular Buffer for Windows](docs/decisions/003-circular-buffer.md)

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

## Features

### Phase 1 - Market Data Infrastructure (Complete ✅)

**Data Sources**
- ✅ Abstract `MarketDataSource` trait for pluggable data feeds
- ✅ Simulated feed with random walk price generation
- ✅ **Binance WebSocket integration**
  - Real-time kline/candlestick data (1s to 1M intervals)
  - Live bid/ask prices from bookTicker stream
  - Combined streams for multiple symbols
  - Binance.com (international) and Binance.US support
  - Automatic reconnection with ping/pong keepalive
  - Regional endpoint selection

**Core Data Structures**
- ✅ `MarketData`: OHLCV candlesticks with bid/ask prices
- ✅ `MarketDataWindow`: Circular buffer with time-series queries
  - High/low/average volume calculations
  - Closes extraction for indicator calculations
  - O(1) insertion, bounded memory
- ✅ `MarketDataStorage`: Thread-safe multi-symbol storage
  - Concurrent reads with RwLock
  - Automatic window creation per symbol

**Developer Experience**
- ✅ Comprehensive error handling with `thiserror`
- ✅ Structured logging with `tracing`
- ✅ Full API documentation with examples
- ✅ 47+ tests (unit, integration, doc tests)
- ✅ Demo application with simulated and live modes

### Upcoming Features

- **Phase 2** (Week 3-4): OCaml indicator library (SMA, EMA, RSI, MACD, Bollinger Bands)
- **Phase 3** (Week 4-5): State machine core (idle, analyzing, trading states)
- **Phase 4** (Week 5-6): Lua strategy integration (custom trading logic)
- **Phase 5** (Week 6-7): Multi-symbol threading (parallel symbol processing)

See [trading-system-roadmap.md](trading-system-roadmap.md) for complete plan.

## Performance Targets

- State machine: 1000+ transitions/second
- Concurrent symbols: 100+ with <1% CPU per symbol
- Indicator calculations: <1ms for typical periods
- End-to-end latency: <10ms from data → action
- Memory: <100MB per symbol

## License

[Your license here]

## Contributing

See [docs/README.md](docs/README.md) for documentation standards and contributing guidelines.
