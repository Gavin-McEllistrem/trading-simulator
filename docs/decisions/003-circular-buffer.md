# ADR 003: Circular Buffer for Market Data Window

## Status
Accepted

## Date
2025-12-15

## Context

The trading engine needs to store recent market data efficiently for real-time indicator calculations. Strategies typically only need the last N bars (e.g., 100-1000 data points).

Requirements:
- Store fixed number of recent data points
- Fast insertion (O(1) for new data)
- Efficient queries (high, low, average)
- Bounded memory usage

## Decision

Use a **circular buffer** (`VecDeque<MarketData>`) for `MarketDataWindow` with a fixed size.

## Rationale

1. **Fixed Memory**: O(window_size), prevents unbounded growth in long-running processes
2. **O(1) Push**: New data added in constant time
3. **Standard Library**: Well-tested, maintained by Rust team
4. **Cache-Friendly**: Contiguous memory allocation
5. **Simple**: Easy to reason about, debug, and maintain

## Implementation

```rust
pub struct MarketDataWindow {
    data: VecDeque<MarketData>,
    max_size: usize,
}

impl MarketDataWindow {
    pub fn push(&mut self, market_data: MarketData) {
        if self.data.len() >= self.max_size {
            self.data.pop_front(); // Remove oldest
        }
        self.data.push_back(market_data); // Add newest
    }
}
```

## Consequences

### Positive
- Fixed memory: Won't leak in 24/7 operation
- Predictable performance
- Simple implementation
- Well-understood data structure
- Good cache locality for recent data access

### Negative
- **Fixed history**: Can't query beyond window size
  - *Mitigation*: Store historical data separately (Phase 7)
- **No persistence**: Data lost on restart
  - *Mitigation*: Event sourcing system (Phase 7)
- **O(n) queries**: high(), low(), avg() are linear
  - *Acceptable*: n is small (typically 20-200 for lookback)

### Configuration

Window size should be configurable per symbol:
- **Minute bars**: 1000-5000 (16-83 hours)
- **Tick data**: 100-500 (seconds of data)
- **Daily bars**: 365-1000 (1-3 years)

## Alternatives Considered

### 1. Vec with Unbounded Growth
**Rejected**: Memory leak risk in long-running processes (24/7 trading)

### 2. LinkedList
**Rejected**: Poor cache locality, slower traversal for indicator calculations

### 3. Custom Ring Buffer
**Rejected**: Over-engineering. `VecDeque` is sufficient and well-optimized

### 4. Database-backed Storage
**Rejected for window**: Too slow for real-time indicator calculations. Database is for historical data (separate concern)

## Performance Characteristics

- `push()`: O(1) amortized
- `high(n)`: O(n) where n is lookback period
- `low(n)`: O(n)
- `avg_volume(n)`: O(n)
- Memory: O(max_size)
- Typical n: 20-200 (SMA, EMA periods)

## Testing

Unit tests verify:
- Circular behavior (oldest removed when full)
- Correct high/low calculations
- Volume averaging
- Empty window edge cases

See: `src/market_data/window.rs` tests

## Related

- Phase 1.1: Core Data Structures
- Phase 1.7: Data Storage
- Phase 7: Event Sourcing (for historical persistence)

## Review Date

After Phase 2 (indicators) - verify performance is adequate for OCaml FFI
