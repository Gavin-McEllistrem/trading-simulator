# Web App Infrastructure - Phase 6

**Status:** 🚧 In Development (40% Complete - Steps 1-4 Done)

**Goal:** Build HTTP/WebSocket API to enable a web-based trading dashboard with live charts, runner management, and strategy creation.

**Completed:**
- ✅ Event streaming system (Steps 1-3)
- ✅ State introspection API (Step 4)

**Next:**
- 📅 HTTP/WebSocket server (Steps 5-8)

---

## Overview

This phase adds a complete API layer on top of the existing TradingEngine to support:
- Real-time chart updates (WebSocket streaming)
- Runner state visualization (FSM states, positions, P&L)
- Strategy management (create, validate, deploy)
- Runner lifecycle management (create, monitor, remove)

---

## Architecture Changes

### Current Architecture (Phase 5)

```
Market Data Feed
    ↓
TradingEngine (manages runners)
    ↓
SymbolRunner (async tasks) → LuaStrategy → StateMachine
```

**Problem:** No way to observe what's happening inside runners. They're "black boxes" running in background tasks.

---

### New Architecture (Phase 6)

```
┌─────────────────────────────────────────────────────────┐
│                    Web Frontend                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ Live Chart   │  │ Runner List  │  │ Strategy     │ │
│  │ (WebSocket)  │  │ (REST API)   │  │ Editor       │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└────────────┬────────────────┬────────────────┬─────────┘
             │ WebSocket      │ HTTP           │ HTTP
             │                │                │
┌────────────▼────────────────▼────────────────▼─────────┐
│              Axum HTTP/WebSocket Server                 │
│  ┌──────────────────────────────────────────────────┐  │
│  │  REST API:                                       │  │
│  │    GET  /api/runners           (list all)       │  │
│  │    POST /api/runners           (create)         │  │
│  │    GET  /api/runners/:id       (details)        │  │
│  │    GET  /api/runners/:id/snapshot               │  │
│  │    GET  /api/runners/:id/history                │  │
│  │    DELETE /api/runners/:id     (remove)         │  │
│  │                                                  │  │
│  │  WebSocket:                                      │  │
│  │    WS /ws (real-time event stream)              │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────┬────────────────────────────────┘
                      │
           ┌──────────▼──────────┐
           │  TradingEngine      │
           │  + Event Stream     │ ◄─── NEW: Event aggregation
           │  + Snapshot API     │ ◄─── NEW: State introspection
           └──────────┬──────────┘
                      │
        ┌─────────────▼─────────────┐
        │  SymbolRunner (tasks)     │
        │  + Event Emission         │ ◄─── NEW: Emit events on tick/action/state change
        │  + Command Channel        │ ◄─── NEW: Accept snapshot requests
        └───────────────────────────┘
```

---

## Key Components

### 1. Event Streaming System

**Purpose:** Broadcast real-time updates from runners to web clients

**New File:** `engine-core/src/events.rs`

```rust
pub enum RunnerEvent {
    TickReceived { runner_id, symbol, data },
    StateTransition { runner_id, from, to, reason },
    ActionExecuted { runner_id, action },
    PositionOpened { runner_id, position },
    PositionUpdated { runner_id, unrealized_pnl },
    PositionClosed { runner_id, realized_pnl },
    Error { runner_id, error },
    RunnerStarted { runner_id, symbol },
    RunnerStopped { runner_id, reason },
}
```

**Changes to SymbolRunner:**
- Add `event_tx: Option<mpsc::UnboundedSender<RunnerEvent>>`
- Emit events in `process_tick()`:
  - `TickReceived` when data arrives
  - `StateTransition` when FSM state changes
  - `ActionExecuted` when strategy returns an action
  - `PositionUpdated` every tick when in position

**Changes to TradingEngine:**
- Add `event_tx/event_rx` for global event stream
- Pass event sender to each runner on creation
- Provide `subscribe_events()` for clients to receive events

---

### 2. State Introspection API

**Purpose:** Query runner internals for dashboard display

**New Types:**

```rust
pub struct RunnerSnapshot {
    pub runner_id: String,
    pub symbol: String,
    pub state: State,                    // Idle/Analyzing/InPosition
    pub position: Option<Position>,      // Current position details
    pub context: Context,                // Strategy context data
    pub stats: RunnerStats,              // Performance metrics
    pub uptime_ms: i64,
    pub window_size: usize,
    pub window_len: usize,               // How many ticks stored
}

pub enum RunnerCommand {
    GetSnapshot { reply_tx: oneshot::Sender<RunnerSnapshot> },
    GetPriceHistory { count: usize, reply_tx: oneshot::Sender<Vec<MarketData>> },
}
```

**Changes to SymbolRunner:**
- Add `command_rx: mpsc::UnboundedReceiver<RunnerCommand>`
- Modify `run()` loop to use `tokio::select!`:
  - Handle market data from `data_receiver`
  - Handle commands from `command_rx`
- Implement `handle_command()` to respond with snapshots

**Changes to TradingEngine:**
- Add `command_tx` to each `RunnerHandle`
- Implement `get_runner_snapshot(runner_id) -> RunnerSnapshot`
- Implement `get_price_history(runner_id, count) -> Vec<MarketData>`
- Implement `get_all_snapshots() -> Vec<RunnerSnapshot>`

---

### 3. HTTP/WebSocket Server

**Purpose:** Expose REST API and WebSocket for web frontend

**New Dependencies:**
```toml
axum = "0.7"
tower-http = { version = "0.5", features = ["cors"] }
tokio-tungstenite = "0.21"
```

**New Module:** `engine-core/src/api/`

```
src/api/
├── mod.rs           # Router setup
├── runners.rs       # Runner CRUD endpoints
├── strategies.rs    # Strategy management
├── websocket.rs     # WebSocket event streaming
└── models.rs        # API request/response types
```

**REST Endpoints:**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/runners` | List all runners with summary info |
| POST | `/api/runners` | Create new runner |
| GET | `/api/runners/:id` | Get runner details |
| GET | `/api/runners/:id/snapshot` | Get full state snapshot |
| GET | `/api/runners/:id/history?count=100` | Get price history |
| DELETE | `/api/runners/:id` | Remove runner |
| GET | `/api/symbols` | List active symbols |
| GET | `/api/symbols/:symbol/runners` | Runners for symbol |
| GET | `/api/strategies` | List available strategies |
| POST | `/api/strategies` | Upload new strategy |
| POST | `/api/strategies/:name/validate` | Validate Lua code |
| GET | `/api/health` | Engine health check |

**WebSocket:**
- `WS /ws` - Subscribe to global event stream
- Receives JSON-serialized `RunnerEvent` messages
- Client can filter by runner_id/symbol

---

### 4. Enhanced LuaStrategy API

**Purpose:** Support strategy creation from string (not just file path)

**New Methods:**

```rust
impl LuaStrategy {
    // Existing
    pub fn new(path: &str) -> Result<Self>

    // NEW: Create from code string
    pub fn from_code(lua_code: &str) -> Result<Self>

    // NEW: List available strategy files
    pub fn list_available_strategies() -> Result<Vec<String>>

    // NEW: Validate without creating
    pub fn validate_code(lua_code: &str) -> Result<()>
}
```

---

## Implementation Steps

### Step 1: Event System Foundation (Est: 3-4 hours)

1. ✅ Create `engine-core/src/events.rs`
2. ✅ Define `RunnerEvent` enum with all event types
3. ✅ Define `ErrorSeverity` enum
4. ✅ Add to `lib.rs`: `pub mod events;`
5. ✅ Add `Serialize`/`Deserialize` derives

**Deliverable:** Event types compile and are JSON-serializable

---

### Step 2: Runner Event Emission (Est: 4-5 hours)

1. ✅ Add `runner_id: String` field to `SymbolRunner`
2. ✅ Add `event_tx: Option<mpsc::UnboundedSender<RunnerEvent>>` field
3. ✅ Add `with_event_channel()` builder method
4. ✅ Implement `emit_event()` helper
5. ✅ Emit events in `process_tick()`:
   - `TickReceived` at start
   - `StateTransition` when state changes
   - `ActionExecuted` when action taken
   - `PositionOpened/Updated/Closed` based on position state
6. ✅ Update `TradingEngine::add_runner()` to pass event channel

**Deliverable:** Runners emit events (verify with test that captures events)

---

### Step 3: Engine Event Aggregation (Est: 2-3 hours)

1. ✅ Add `event_tx/event_rx` to `TradingEngine`
2. ✅ Pass `event_tx.clone()` to each runner
3. ✅ Implement `subscribe_events() -> mpsc::UnboundedReceiver<RunnerEvent>`
4. ✅ Emit `RunnerStarted/Stopped` events in `add_runner()/remove_runner()`

**Deliverable:** `TradingEngine::subscribe_events()` returns stream of all events

---

### Step 4: Command Channel & Introspection (Est: 4-5 hours)

1. ✅ Create `RunnerSnapshot` struct in `runner/mod.rs`
2. ✅ Create `RunnerCommand` enum
3. ✅ Add `command_rx: mpsc::UnboundedReceiver<RunnerCommand>` to `SymbolRunner`
4. ✅ Modify `run()` to use `tokio::select!` on both channels
5. ✅ Implement `handle_command()` method
6. ✅ Add `command_tx` to `RunnerHandle`
7. ✅ Implement `TradingEngine::get_runner_snapshot()`
8. ✅ Implement `TradingEngine::get_price_history()`
9. ✅ Implement `TradingEngine::get_all_snapshots()`

**Deliverable:** Can query runner state via `get_runner_snapshot()`

---

### Step 5: LuaStrategy Enhancements (Est: 1-2 hours)

1. ✅ Add `LuaStrategy::from_code(lua_code: &str)`
2. ✅ Add `LuaStrategy::list_available_strategies()`
3. ✅ Add `LuaStrategy::validate_code(lua_code: &str)`
4. ✅ Write tests for new methods

**Deliverable:** Strategies can be created from code strings

---

### Step 6: HTTP Server Setup (Est: 3-4 hours)

1. ✅ Add dependencies to `Cargo.toml`
2. ✅ Create `src/api/mod.rs` with router
3. ✅ Create `src/api/models.rs` for request/response types
4. ✅ Create basic server in new binary `engine-core/src/bin/server.rs`
5. ✅ Add CORS middleware
6. ✅ Test with `curl` or Postman

**Deliverable:** Server starts and responds to `/api/health`

---

### Step 7: REST Endpoints (Est: 6-8 hours)

1. ✅ Implement `src/api/runners.rs`:
   - `GET /api/runners`
   - `POST /api/runners`
   - `GET /api/runners/:id`
   - `GET /api/runners/:id/snapshot`
   - `GET /api/runners/:id/history`
   - `DELETE /api/runners/:id`
2. ✅ Implement `src/api/strategies.rs`:
   - `GET /api/strategies`
   - `POST /api/strategies`
   - `POST /api/strategies/:name/validate`
3. ✅ Wire up in main router

**Deliverable:** All endpoints respond correctly

---

### Step 8: WebSocket Streaming (Est: 3-4 hours)

1. ✅ Implement `src/api/websocket.rs`
2. ✅ Add `WS /ws` route
3. ✅ Subscribe to engine events
4. ✅ Forward events to WebSocket as JSON
5. ✅ Handle client disconnection
6. ✅ Test with `websocat` or browser

**Deliverable:** WebSocket clients receive real-time events

---

### Step 9: Integration Testing (Est: 4-5 hours)

1. ✅ Write integration tests in `tests/api_integration.rs`
2. ✅ Test runner CRUD operations
3. ✅ Test snapshot retrieval
4. ✅ Test WebSocket event streaming
5. ✅ Test concurrent clients
6. ✅ Test error handling

**Deliverable:** Comprehensive test coverage

---

### Step 10: Documentation & Examples (Est: 2-3 hours)

1. ✅ Create `docs/guides/web-app-api.md`
2. ✅ Create example web app in `examples/web-dashboard/`
   - Simple HTML + vanilla JS
   - Live chart using Chart.js
   - WebSocket connection
   - Runner management UI
3. ✅ Update main README

**Deliverable:** Working example web dashboard

---

## Testing Strategy

### Unit Tests
- Event serialization/deserialization
- Snapshot creation
- Command handling in runner

### Integration Tests
- Full event flow (runner → engine → subscriber)
- Snapshot retrieval under load
- WebSocket connection handling
- REST API CRUD operations

### Manual Testing
- Open browser console, connect to WebSocket
- Watch events stream in real-time
- Test with 10+ concurrent runners
- Measure latency (tick → WebSocket delivery)

---

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Event emission overhead | <100µs per event | Should not slow down tick processing |
| Snapshot retrieval | <10ms | oneshot channel round-trip |
| WebSocket latency | <50ms | Tick received → Browser receives event |
| Concurrent clients | 100+ | WebSocket connections |
| Events per second | 10,000+ | 100 runners × 100 ticks/sec |

---

## Security Considerations

### Phase 6 (MVP)
- ❌ No authentication (local dev only)
- ❌ No authorization (all clients can control all runners)
- ✅ CORS enabled (restrict to localhost:3000)
- ✅ Input validation (Lua code validation)

### Future (Production)
- Add JWT authentication
- Add user/runner ownership
- Rate limiting on API endpoints
- WebSocket connection limits
- Strategy code sandboxing

---

## API Usage Examples

### Create a Runner

```bash
curl -X POST http://localhost:8080/api/runners \
  -H "Content-Type: application/json" \
  -d '{
    "runner_id": "btc_ema_prod",
    "symbol": "BTCUSDT",
    "strategy_name": "ema_crossover",
    "window_size": 100,
    "config": {
      "stop_on_error": true,
      "log_actions": true
    }
  }'
```

### Get Runner Snapshot

```bash
curl http://localhost:8080/api/runners/btc_ema_prod/snapshot
```

Response:
```json
{
  "runner_id": "btc_ema_prod",
  "symbol": "BTCUSDT",
  "state": "InPosition",
  "position": {
    "entry_price": 50000.0,
    "quantity": 0.1,
    "side": "Long",
    "unrealized_pnl": 150.0
  },
  "stats": {
    "ticks_processed": 1234,
    "actions_executed": 5,
    "error_rate": 0.0
  }
}
```

### WebSocket Connection (JavaScript)

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onmessage = (event) => {
  const runnerEvent = JSON.parse(event.data);

  if (runnerEvent.TickReceived) {
    updateChart(runnerEvent.TickReceived.data);
  }

  if (runnerEvent.StateTransition) {
    updateFSMVisualization(runnerEvent.StateTransition);
  }

  if (runnerEvent.PositionUpdated) {
    updatePnLDisplay(runnerEvent.PositionUpdated.unrealized_pnl);
  }
};
```

---

## Migration Notes

### Breaking Changes
- `SymbolRunner::new()` signature unchanged, but new optional builder methods
- `TradingEngine` constructor unchanged
- No breaking changes to existing API

### Backward Compatibility
- All existing examples/tests continue to work
- Event emission is opt-in (if no event channel provided, no events emitted)
- Command channel is transparent (if no commands sent, no overhead)

---

## Next Phase (Phase 7)

After web app infrastructure is complete:
- Historical backtesting engine
- Backtest result visualization
- Strategy performance comparison
- Parameter optimization

---

## Questions / Decisions

1. **Event filtering:** Should WebSocket clients filter events client-side or server-side?
   - **Decision:** Client-side for MVP (simpler), server-side filter later

2. **Event history:** Should engine store recent events for late-joining clients?
   - **Decision:** No for MVP, WebSocket only sends future events

3. **Snapshot caching:** Should snapshots be cached to reduce oneshot overhead?
   - **Decision:** No for MVP, optimize later if needed

4. **Multiple WebSocket endpoints:** One per runner or one global?
   - **Decision:** One global for MVP, can add per-runner later

---

## Success Criteria

Phase 6 is complete when:
- ✅ Web browser can connect via WebSocket and see live events
- ✅ Chart updates in real-time as ticks arrive
- ✅ Can create/remove runners via REST API
- ✅ Can view runner state (FSM state, position, P&L) in dashboard
- ✅ Can upload and validate Lua strategies
- ✅ All tests pass
- ✅ Example web dashboard works

---

**Estimated Total Time:** 35-45 hours (1-2 weeks)

**Current Progress:** Step 1 - Event System Foundation 🚀
