# Trading Engine Documentation

Welcome to the trading engine documentation!

## Getting Started

- [Getting Started Guide](guides/getting-started.md) - Setup and first run
- [Architecture Overview](architecture/01-overview.md) - System design

## For Users

### Guides
- [Getting Started](guides/getting-started.md)
- Binance Setup (Coming in Phase 1)
- Writing Strategies (Coming in Phase 4)
- Configuration Reference (Coming in Phase 1)

## For Developers

### Architecture
- [System Overview](architecture/01-overview.md)
- Data Flow (Coming in Phase 2)
- Threading Model (Coming in Phase 5)
- State Machine Design (Coming in Phase 3)

### Decision Records
- [ADR 001: Rust Edition 2021](decisions/001-rust-2021.md)
- [ADR 002: engine-core Naming](decisions/002-engine-core-naming.md)
- [ADR 003: Circular Buffer for Windows](decisions/003-circular-buffer.md)

### API Documentation

Generate and view the API documentation:

```bash
cd engine-core
cargo doc --no-deps --open
```

## Project Status

**Current Phase:** 0-1 (Foundation + Market Data Infrastructure) ✅

See [Full Roadmap](../trading-system-roadmap.md) for complete project plan.

### Completed
- ✅ Project setup and structure
- ✅ Core data structures (MarketData, MarketDataWindow)
- ✅ Data source abstraction (MarketDataSource trait)
- ✅ Simulated feed for testing
- ✅ Thread-safe storage
- ✅ Configuration system
- ✅ Error handling framework

### In Progress
- 🚧 Binance WebSocket integration (Day 4-5)

### Upcoming
- 📅 Alpaca integration (Phase 1)
- 📅 OCaml indicator library (Phase 2)
- 📅 State machine (Phase 3)
- 📅 Lua strategies (Phase 4)

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
