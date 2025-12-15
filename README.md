# Trading Simulator

A high-performance, multi-threaded trading system with Rust core engine, OCaml indicators, and Lua strategy scripting.

## Project Status

**Current Phase:** 0-1 (Foundation + Market Data) ✅ **COMPLETE**

### Completed
- ✅ Project structure and build system
- ✅ Core data structures (MarketData, MarketDataWindow)
- ✅ Data source abstraction (MarketDataSource trait)
- ✅ Simulated feed for testing
- ✅ Thread-safe storage system
- ✅ Configuration framework
- ✅ Error handling
- ✅ Comprehensive documentation

### Next Steps
- 🚧 Binance WebSocket integration (Day 4-5)
- 📅 OCaml indicator library (Phase 2)
- 📅 State machine (Phase 3)
- 📅 Lua strategies (Phase 4)

## Quick Start

```bash
# Navigate to engine core
cd engine-core

# Build the project
cargo build

# Run with simulated data
cargo run

# Run tests
cargo test

# View documentation
cargo doc --no-deps --open
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

- **[Getting Started Guide](docs/guides/getting-started.md)** - Setup and first run
- **[Architecture Overview](docs/architecture/01-overview.md)** - System design
- **[API Documentation](engine-core/target/doc/trading_engine/index.html)** - Generated from code
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

### Phase 0-1 (Complete ✅)

- **Market Data Infrastructure**
  - Abstract data source trait
  - Simulated feed for testing
  - Circular buffer for efficient storage
  - Thread-safe concurrent access

- **Core Data Structures**
  - MarketData: OHLCV candlesticks
  - MarketDataWindow: Time-series queries
  - MarketDataStorage: Multi-symbol storage

- **Developer Experience**
  - Comprehensive error handling
  - Structured logging with tracing
  - Full API documentation
  - Tested code examples

### Upcoming Features

- **Phase 1** (Week 2-3): Binance + Alpaca integration
- **Phase 2** (Week 3-4): OCaml indicator library
- **Phase 3** (Week 4-5): State machine core
- **Phase 4** (Week 5-6): Lua strategy integration
- **Phase 5** (Week 6-7): Multi-symbol threading

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
