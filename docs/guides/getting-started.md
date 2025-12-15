# Getting Started with Trading Engine

## Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Git

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd trading-simulator

# Navigate to the engine core
cd engine-core

# Build the project
cargo build --release
```

## Quick Start

### 1. Run with Simulated Data

The easiest way to get started is with the simulated data feed:

```bash
cargo run
```

You should see output like:

```
INFO Trading Engine starting...
INFO Tick 1: BTCUSDT - close: 50646.50, volume: 1051
INFO Tick 2: BTCUSDT - close: 51246.79, volume: 1405
...
INFO Storage contains 10 data points
INFO 10-period High: 51281.47, Low: 47102.37
INFO Trading Engine stopped
```

### 2. Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific module tests
cargo test market_data::
```

### 3. View Documentation

```bash
# Generate and open HTML documentation
cargo doc --no-deps --open
```

## Next Steps

### Phase 1: Market Data (Current)

- [x] Simulated feed working
- [ ] Binance WebSocket integration (Day 4-5)
- [ ] Alpaca integration (optional)
- [ ] Historical data loading

### Phase 2: Indicators (Week 3-4)

- OCaml indicator library
- FFI integration with Rust
- Basic indicators (SMA, EMA, RSI)

### Phase 3: State Machine (Week 4-5)

- Core state machine implementation
- Strategy trait definition

## Project Structure

```
trading-simulator/
├── engine-core/          # Main Rust engine
│   ├── src/
│   │   ├── market_data/  # OHLCV data structures
│   │   ├── sources/      # Data source implementations
│   │   ├── storage/      # Thread-safe storage
│   │   ├── config/       # Configuration types
│   │   └── error.rs      # Error handling
│   ├── tests/            # Integration tests
│   └── Cargo.toml
├── ocaml-indicators/     # Indicator library (Phase 2)
├── lua-strategies/       # Strategy scripts (Phase 4)
├── tests/                # End-to-end tests
└── docs/                 # Documentation
```

## Development Workflow

### 1. Make Changes

Edit code in `engine-core/src/`

### 2. Format Code

```bash
cargo fmt
```

### 3. Lint Code

```bash
cargo clippy
```

### 4. Run Tests

```bash
cargo test
```

### 5. Build

```bash
cargo build
```

## Common Tasks

### Adding a New Module

1. Create module file: `src/my_module/mod.rs`
2. Declare in `lib.rs`: `pub mod my_module;`
3. Add tests in module or `tests/`
4. Document with `//!` module docs and `///` item docs

### Adding a New Dependency

1. Add to `Cargo.toml`:
   ```toml
   [dependencies]
   new-crate = "1.0"
   ```
2. Use in code: `use new_crate::Thing;`
3. Run `cargo build` to download

## Troubleshooting

### Build Errors

```bash
# Clean build artifacts
cargo clean

# Rebuild
cargo build
```

### Test Failures

```bash
# Run with detailed output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Getting Help

- [Architecture Documentation](../architecture/01-overview.md)
- [API Documentation](https://docs.rs/trading-engine)
- [Full Roadmap](../../trading-system-roadmap.md)

## Next: Binance Integration

Once you're comfortable with the basics, proceed to:
- [Binance Setup Guide](binance-setup.md) (Coming in Phase 1)
