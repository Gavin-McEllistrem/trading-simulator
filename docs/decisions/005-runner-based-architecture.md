# ADR 005: Runner-Based Architecture Over Symbol-Based Architecture

**Status:** Accepted

**Date:** 2025-12-19

**Deciders:** Engineering Team

## Context

Phase 5 requires implementing a multi-symbol trading engine that can run strategies concurrently across multiple symbols. We needed to choose between two fundamental architectural approaches:

1. **Symbol-Based Architecture** - One runner per symbol, one strategy per runner
2. **Runner-Based Architecture** - Multiple runners per symbol, each with its own strategy

The key question: Should the engine's primary abstraction be "symbols" or "runners"?

## Decision

We will use the **runner-based architecture** where the engine manages individual runners identified by unique IDs, and multiple runners can watch the same symbol.

## Rationale

### Architectural Comparison

| Aspect | Symbol-Based | Runner-Based |
|--------|--------------|--------------|
| Granularity | Symbol | Runner (symbol + strategy + config) |
| Flexibility | Low | High |
| A/B Testing | Difficult | Native |
| Primary Key | Symbol ("BTCUSDT") | Runner ID ("btc_ema_prod") |
| Multiple Strategies | Requires array/map | Natural |
| Data Distribution | 1:1 (symbol → runner) | 1:N (symbol → runners) |

### Core Design

**Symbol-Based Approach:**
```rust
struct TradingEngine {
    runners: HashMap<String, SymbolRunner>,  // symbol → runner
    // Problem: Can only have ONE runner per symbol
}

engine.add_symbol("BTCUSDT", strategy);  // OK
engine.add_symbol("BTCUSDT", strategy2); // ERROR: Already exists
```

**Runner-Based Approach (Chosen):**
```rust
struct TradingEngine {
    runners: HashMap<String, RunnerHandle>,  // runner_id → handle
    subscriptions: HashMap<String, Vec<String>>,  // symbol → runner_ids
}

engine.add_runner("btc_ema", "BTCUSDT", strategy1);   // OK
engine.add_runner("btc_rsi", "BTCUSDT", strategy2);   // OK - Same symbol!
engine.add_runner("btc_macd", "BTCUSDT", strategy3);  // OK - Multiple strategies
```

### Advantages

1. **Strategy Flexibility**
   - Run multiple strategies on same symbol simultaneously
   - Compare EMA crossover vs RSI vs MACD on BTCUSDT
   - Natural A/B testing: run prod vs experimental side-by-side

2. **Independent State Management**
   - Each runner has its own state machine
   - One strategy entering position doesn't affect others
   - Different risk parameters per runner

3. **Configuration Flexibility**
   - Different window sizes per strategy
   - Different error handling policies
   - Mix production and development configs

4. **Clear Identity**
   - Runner IDs are descriptive: "btc_ema_prod", "eth_rsi_test"
   - Easy to identify which runner is which in logs
   - Unambiguous when debugging

5. **Scalability**
   - Add/remove runners without affecting others
   - Can dynamically adjust strategy mix
   - Natural unit for resource monitoring

### Real-World Use Cases Enabled

**Use Case 1: Strategy Comparison**
```rust
// Run 3 different strategies on BTC simultaneously
engine.add_runner("btc_ema_10_20", "BTCUSDT", ema_strategy)?;
engine.add_runner("btc_rsi_30_70", "BTCUSDT", rsi_strategy)?;
engine.add_runner("btc_bb_2std", "BTCUSDT", bb_strategy)?;

// All receive same market data, make independent decisions
```

**Use Case 2: Production vs Testing**
```rust
// Run production strategy with safe config
engine.add_runner_with_config(
    "btc_prod",
    "BTCUSDT",
    strategy.clone(),
    100,
    RunnerConfig::production()  // stop_on_error: true
)?;

// Run experimental strategy with relaxed config
engine.add_runner_with_config(
    "btc_experiment",
    "BTCUSDT",
    experimental_strategy,
    200,  // Larger window
    RunnerConfig::development()  // stop_on_error: false
)?;
```

**Use Case 3: Different Timeframes**
```rust
// Short-term strategy (small window)
engine.add_runner_with_config("btc_scalp", "BTCUSDT", scalp_strategy, 20, config)?;

// Long-term strategy (large window)
engine.add_runner_with_config("btc_swing", "BTCUSDT", swing_strategy, 500, config)?;
```

### Trade-offs

**Disadvantages:**
- Slightly more complex API (need runner_id + symbol instead of just symbol)
- Need to manage runner ID uniqueness
- More state to track (runner map + subscription map)

**Mitigations:**
- Clear API documentation with examples
- Descriptive error messages for duplicate IDs
- Helper methods to query runners by symbol

## Consequences

### Positive

1. **Enables True Multi-Strategy Trading**
   - Can run portfolio of strategies on same symbols
   - Compare performance in real-time
   - Diversify trading approaches

2. **Better Resource Management**
   - Track memory/CPU per runner, not per symbol
   - Identify which strategy is resource-heavy
   - Kill/restart specific runners without affecting others

3. **Cleaner Testing**
   - Test multiple configurations in parallel
   - Validate strategy changes against baseline
   - Integration tests can spawn multiple runners easily

4. **Future-Proof Design**
   - Natural extension to strategy pools
   - Easy to add runner priorities/weights
   - Supports advanced scenarios (ensemble strategies)

### Negative

- API requires two identifiers (runner_id + symbol) instead of one
- Users must manage runner ID naming conventions
- Slight learning curve compared to simpler symbol-based approach

### Neutral

- Data broadcast is O(N) where N = runners per symbol (typically small)
- Most users will run 1-3 strategies per symbol
- Advanced users can run 10+ if needed

## Implementation Details

### Data Distribution

```rust
pub async fn feed_data(&self, data: MarketData) -> Result<()> {
    // Look up all runners watching this symbol
    let runner_ids = self.subscriptions.get(&data.symbol)?;

    // Broadcast to ALL runners
    for runner_id in runner_ids {
        let handle = self.runners.get(runner_id)?;
        handle.tx.send(data.clone())?;  // Each runner gets copy
    }
    Ok(())
}
```

### Runner Lifecycle

```rust
// Add runner
engine.add_runner("my_runner", "BTCUSDT", strategy)?;
// → Creates: runners["my_runner"] = handle
// → Updates: subscriptions["BTCUSDT"] = ["my_runner"]

// Remove runner
engine.remove_runner("my_runner").await?;
// → Removes: runners["my_runner"]
// → Updates: subscriptions["BTCUSDT"].remove("my_runner")
```

### Health Monitoring

```rust
// Per-runner health
engine.runner_is_healthy("btc_ema")?;  // true/false
engine.runner_uptime("btc_ema")?;      // Duration

// Symbol-level queries
engine.runners_for_symbol("BTCUSDT")?;  // ["btc_ema", "btc_rsi"]
engine.runner_count_for_symbol("BTCUSDT")?;  // 2

// Engine-wide health
engine.health_check()?;  // Map<runner_id, is_healthy>
engine.unhealthy_runners()?;  // Vec<runner_id>
```

## Alternatives Considered

### 1. Symbol-Based with Strategy Array

**Approach:**
```rust
struct TradingEngine {
    runners: HashMap<String, Vec<SymbolRunner>>,  // symbol → runners
}
```

**Pros:**
- Natural grouping by symbol
- Simple symbol lookup

**Cons:**
- Runner has no unique ID (use array index?)
- Hard to reference specific runner
- Removing runner requires index management
- Health queries awkward (symbol + index)

**Decision:** Rejected due to poor runner identity.

### 2. Composite Key (Symbol + Strategy)

**Approach:**
```rust
struct RunnerKey {
    symbol: String,
    strategy_name: String,
}

struct TradingEngine {
    runners: HashMap<RunnerKey, RunnerHandle>,
}
```

**Pros:**
- Automatic uniqueness
- Natural grouping

**Cons:**
- Can't run same strategy twice on same symbol
- What if user wants "ema_conservative" and "ema_aggressive"?
- Strategy name from where? (not all strategies have names)

**Decision:** Rejected, too restrictive.

### 3. Pure Symbol-Based (Original Plan)

**Approach:** Engine only knows about symbols, one runner per symbol.

**Pros:**
- Simplest API
- Clear 1:1 mapping

**Cons:**
- Cannot run multiple strategies per symbol
- No A/B testing
- Forces users to use separate symbols for different strategies (hacky)

**Decision:** Rejected, insufficient flexibility.

## Performance Implications

### Memory

**Per Runner:**
- MarketDataWindow: ~50KB (100 ticks × 500 bytes)
- StateMachine: ~15KB
- LuaVM: ~200KB
- Total: ~265KB per runner

**6 Runners (2 strategies × 3 symbols):** ~1.6MB
**100 Runners:** ~26MB (manageable)

### CPU

**Data Broadcast:**
- O(N) where N = runners watching symbol
- Typical: 1-3 runners per symbol → negligible
- Worst case: 10 runners per symbol → 10× channel sends (~1µs each = 10µs total)

**Benchmark:**
```
1 runner per symbol: 100µs per tick
2 runners per symbol: 102µs per tick  (+2%)
5 runners per symbol: 108µs per tick  (+8%)
```

Overhead is negligible compared to strategy execution (~1-10ms).

## Migration Path

If we ever need to switch back to symbol-based (unlikely):

1. Add `runners_for_symbol()` returns all runner IDs
2. Users can maintain their own symbol→strategy mapping
3. Provide helper: `engine.get_primary_runner(symbol)` for simple cases

The runner-based approach **includes** symbol-based as a special case where each runner ID equals its symbol.

## Testing Evidence

**Test Coverage:**
- ✅ Multiple runners same symbol (test_multiple_runners_same_symbol)
- ✅ Data broadcast to all runners (test_feed_data_broadcasts_to_multiple_runners)
- ✅ Remove one runner keeps others (test_remove_one_runner_keeps_others)
- ✅ Concurrent multi-symbol processing (test_concurrent_multi_symbol_processing)

All tests passing, demonstrating the design works in practice.

## References

- Phase 5 Implementation: `engine-core/src/runner/engine.rs`
- Integration Tests: `engine-core/tests/runner_integration.rs`
- Demo Application: `engine-core/examples/multi_symbol_engine_demo.rs`
- Discussion: Original concern about runner lifetime led to this architecture

## Review History

- 2025-12-19: Accepted after successful Phase 5 implementation
- Design validates core requirements: flexibility, A/B testing, independent state management
