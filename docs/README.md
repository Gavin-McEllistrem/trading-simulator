# Trading Engine Documentation

Welcome to the trading engine documentation!

## Getting Started

- [Getting Started Guide](guides/getting-started.md) - Setup and first run
- [Architecture Overview](architecture/01-overview.md) - System design

## For Users

### Guides
- [Getting Started](guides/getting-started.md) ✅
- [Binance Setup](guides/binance-setup.md) ✅
- [Lua Strategy Development Guide](guides/lua-strategy-guide.md) ✅ **NEW!**
- [Technical Indicators](guides/indicators.md) (Coming Soon)
- Configuration Reference (Coming Soon)

## For Developers

### Architecture
- [System Overview](architecture/01-overview.md) ✅
- [Strategy Integration](architecture/02-strategy-integration.md) ✅ **NEW!**
- Threading Model (Coming in Phase 5)
- Performance Analysis (Coming Soon)

### Decision Records
- [ADR 001: Rust Edition 2021](decisions/001-rust-2021.md)
- [ADR 002: engine-core Naming](decisions/002-engine-core-naming.md)
- [ADR 003: Circular Buffer for Windows](decisions/003-circular-buffer.md)
- [ADR 004: Subprocess over FFI for OCaml](decisions/004-subprocess-over-ffi.md)

### API Documentation

Generate and view the API documentation:

```bash
cd engine-core
cargo doc --no-deps --open
```

## Project Status

**Current Phase:** 4 (Lua Strategy Integration) ✅ **COMPLETE**

**Progress:** 4 of 12 phases complete (33%)

See [Full Roadmap](../trading-system-roadmap.md) for complete project plan.

### Completed
- ✅ **Phase 1: Market Data Infrastructure**
  - Core data structures (MarketData, MarketDataWindow)
  - Binance WebSocket integration
  - Simulated feed for testing
  - Thread-safe storage
  - 47 tests passing

- ✅ **Phase 2: Technical Indicators (Dual Rust/OCaml)**
  - SMA, EMA, RSI, MACD, Bollinger Bands
  - OCaml subprocess bridge (1-2ms latency)
  - Full verification suite
  - 48 tests passing

- ✅ **Phase 3: State Machine Core**
  - 3-state FSM (Idle, Analyzing, InPosition)
  - Position tracking with P&L calculation
  - Auto-exit on stop loss / take profit
  - Transition history
  - 28 tests passing

- ✅ **Phase 4: Lua Strategy Integration** 🎉
  - LuaStrategy system with VM management
  - Full Lua API (market data, indicators, actions)
  - 3 production-ready example strategies
  - 14 tests passing
  - **[Strategy Development Guide](guides/lua-strategy-guide.md)** available!

**Total: 117 tests passing, ~8,600 LOC**

### Upcoming
- 📅 Multi-symbol threading engine (Phase 5)
- 📅 Execution & risk management (Phase 6)

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
