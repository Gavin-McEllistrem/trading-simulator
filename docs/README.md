# Trading Engine Documentation

Welcome to the trading engine documentation!

## Getting Started

- [Getting Started Guide](guides/getting-started.md) - Setup and first run
- [Architecture Overview](architecture/01-overview.md) - System design

## For Users

### Guides
- [Getting Started](guides/getting-started.md)
- [Binance Setup](guides/binance-setup.md) ✅
- [Technical Indicators](guides/indicators.md) (Coming Soon)
- Writing Strategies (Coming in Phase 4)
- Configuration Reference (Coming Soon)

## For Developers

### Architecture
- [System Overview](architecture/01-overview.md)
- Indicator Architecture (Coming Soon)
- Threading Model (Coming in Phase 5)
- State Machine Design (Coming in Phase 3)

### Decision Records
- [ADR 001: Rust Edition 2021](decisions/001-rust-2021.md)
- [ADR 002: engine-core Naming](decisions/002-engine-core-naming.md)
- [ADR 003: Circular Buffer for Windows](decisions/003-circular-buffer.md)
- [ADR 004: Subprocess over FFI for OCaml](decisions/004-subprocess-over-ffi.md) (Coming Soon)

### API Documentation

Generate and view the API documentation:

```bash
cd engine-core
cargo doc --no-deps --open
```

## Project Status

**Current Phase:** 3 (State Machine Core) ✅ **COMPLETE**

See [Full Roadmap](../trading-system-roadmap.md) for complete project plan.

### Completed
- ✅ Project setup and structure
- ✅ Core data structures (MarketData, MarketDataWindow)
- ✅ Data source abstraction (MarketDataSource trait)
- ✅ Simulated feed for testing
- ✅ Thread-safe storage
- ✅ Configuration system
- ✅ Error handling framework
- ✅ **Binance WebSocket integration**
  - Real-time kline (OHLCV) streams
  - Live bid/ask from bookTicker
  - Binance.com and Binance.US endpoints
  - Multiple symbols support
  - Automatic keepalive
- ✅ **Technical Indicators (Dual Rust/OCaml)**
  - SMA, EMA (Moving Averages)
  - RSI (Relative Strength Index)
  - MACD (Moving Average Convergence Divergence)
  - Bollinger Bands
  - OCaml reference implementation via subprocess
  - 48 tests passing (40 Rust + 8 OCaml)
- ✅ **State Machine Core**
  - 3 states: Idle, Analyzing, InPosition
  - Position tracking with P&L calculation
  - Auto-exit on stop loss / take profit
  - Transition history
  - 28 tests passing

### Upcoming
- 📅 Lua strategies (Phase 4)
- 📅 Multi-symbol engine (Phase 5)

## Documentation Standards

### Code Documentation

All public APIs should have:
- Module-level docs (`//!`)
- Item-level docs (`///`)
- Examples in doc comments
- Tests for examples

Example:
```rust
/// Calculates the mid-price between bid and ask.
///
/// # Examples
///
/// ```
/// # use trading_engine::MarketData;
/// let data = MarketData { /* ... */ };
/// assert_eq!(data.mid_price(), 101.0);
/// ```
pub fn mid_price(&self) -> f64 {
    (self.bid + self.ask) / 2.0
}
```

### Architecture Decision Records (ADRs)

Document significant decisions using the ADR format:
- Status (Proposed/Accepted/Deprecated/Superseded)
- Date
- Context
- Decision
- Rationale
- Consequences
- Alternatives Considered

See [existing ADRs](decisions/) for examples.

## Contributing

When adding new features:

1. **Write code** with Rustdoc comments
2. **Add tests** for new functionality
3. **Update guides** if user-facing
4. **Create ADR** for significant design decisions
5. **Update this index** with new documents

## Questions?

- Check the [Getting Started Guide](guides/getting-started.md)
- Review [Architecture docs](architecture/01-overview.md)
- Read the [Full Roadmap](../trading-system-roadmap.md)
