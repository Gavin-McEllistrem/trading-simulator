# System Architecture Overview

## High-Level Design

The trading engine is designed as a modular, multi-threaded system with clear separation of concerns.

```
┌─────────────────────────────────────────────────────────────┐
│                     Trading Engine                           │
│                                                               │
│  ┌──────────────┐      ┌──────────────┐     ┌─────────────┐│
│  │ Data Sources │─────>│   Storage    │────>│    State    ││
│  │  (Binance,   │      │  (Thread-    │     │   Machine   ││
│  │   Alpaca)    │      │    Safe)     │     │  (Phase 3)  ││
│  └──────────────┘      └──────────────┘     └─────────────┘│
│         │                                           │        │
│         v                                           v        │
│  ┌──────────────┐                          ┌─────────────┐ │
│  │  Indicators  │<────────────────────────>│  Strategy   │ │
│  │   (OCaml)    │                          │   (Lua)     │ │
│  │  (Phase 2)   │                          │  (Phase 4)  │ │
│  └──────────────┘                          └─────────────┘ │
│                                                      │       │
│                                                      v       │
│                                            ┌─────────────┐  │
│                                            │  Execution  │  │
│                                            │   Engine    │  │
│                                            │  (Phase 6)  │  │
│                                            └─────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Core Principles

1. **Modularity**: Each component is independently testable and replaceable
2. **Thread Safety**: All shared state uses explicit synchronization primitives
3. **Language Selection**:
   - **Rust**: Performance-critical core (state machine, concurrency)
   - **OCaml**: Functional purity for mathematical indicator calculations
   - **Lua**: User-friendly scripting for strategy definitions
4. **Async by Default**: Tokio runtime for efficient I/O operations

## Current Status: Phase 0-1 (Complete ✅)

### Implemented Components

#### Data Sources (Phase 1)
Abstraction over multiple market data providers with pluggable backends.

- **Interface**: `MarketDataSource` trait
- **Implementations**:
  - ✅ `SimulatedFeed` - Testing and development
  - 🚧 `BinanceSource` - Real-time crypto data (Day 4-5)
  - 📅 `AlpacaSource` - Stock market data
  - 📅 `CSVFeed` - Historical backtesting

#### Storage (Phase 1)
Thread-safe, in-memory storage with windowed history per symbol.

- **Key Types**: `MarketDataStorage`, `MarketDataWindow`
- **Synchronization**: `Arc<RwLock<HashMap>>` for concurrent access
- **Design**: Fixed-size circular buffer to prevent memory growth

#### Configuration (Phase 1)
TOML-based configuration system for flexible runtime setup.

- **Types**: `DataSourceConfig`, `StorageConfig`, `EngineConfig`
- **Supports**: Multiple data source types with source-specific options

## Upcoming Components

### State Machine (Phase 3)
Core trading logic with three states:
- **Watch**: Monitoring for opportunities
- **CloseWatch**: Evaluating entry conditions
- **InTrade**: Managing active positions

### Indicators (Phase 2)
Pure functional calculations implemented in OCaml:
- SMA, EMA, RSI, MACD, Bollinger Bands
- Exposed to Rust via FFI
- Guaranteed correctness through functional purity

### Strategies (Phase 4)
User-defined trading logic in Lua:
- Hot-reloadable scripts
- Safe sandboxed execution
- Access to indicators and market data

## Data Flow

**Phase 1 (Current)**:
```
Data Source → Storage → (Display/Analysis)
```

**Phase 3+ (Future)**:
```
Data Source → Storage → Indicators → State Machine + Strategy → Execution Engine → Broker
```

## Threading Model

- **Main Thread**: Event loop, coordination
- **Per-Symbol Threads** (Phase 5): Independent state machines per trading pair
- **I/O Threads**: Managed by Tokio async runtime

## Performance Targets

- State machine: 1000+ transitions/second
- Concurrent symbols: 100+ with <1% CPU per symbol
- Indicator calculations: <1ms for typical periods
- End-to-end latency: <10ms from data arrival to action
- Memory: <100MB per symbol

## Configuration

All components are configurable via TOML files.

See: [Configuration Guide](../guides/configuration.md)

## References

- [Full Roadmap](../../trading-system-roadmap.md)
- [Getting Started Guide](../guides/getting-started.md)
