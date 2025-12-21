# Web Backend - Implementation Plan & Summary

**Date:** 2025-12-20
**Status:** Scaffolding Complete, Ready for REST API Implementation
**Phase:** 6 - Web App Infrastructure

---

## Table of Contents

1. [Overview](#overview)
2. [Current Status](#current-status)
3. [Architecture](#architecture)
4. [Implementation Plan](#implementation-plan)
5. [API Specification](#api-specification)
6. [WebSocket Design](#websocket-design)
7. [Testing Strategy](#testing-strategy)
8. [Performance Targets](#performance-targets)
9. [Security Considerations](#security-considerations)
10. [Deployment](#deployment)

---

## Overview

The web backend is a Rust-based HTTP and WebSocket server built with **axum** that exposes the trading engine's functionality via REST APIs and real-time event streaming. It acts as the bridge between the frontend dashboard and the core trading engine.

### Purpose

- **REST API** - Query engine state, runner snapshots, and manage runners
- **WebSocket Streaming** - Real-time event notifications (ticks, state changes, positions)
- **Introspection** - Expose internal engine state for debugging and monitoring
- **Control** - Add/remove runners, configure strategies

### Technology Stack

| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| Web Framework | axum | 0.7 | HTTP routing and handlers |
| Async Runtime | tokio | 1.43 | Async I/O and concurrency |
| Middleware | tower-http | 0.5 | CORS, logging, tracing |
| Serialization | serde/serde_json | 1.0 | JSON encoding/decoding |
| Logging | tracing | 0.1 | Structured logging |
| WebSockets | axum (built-in) | 0.7 | Real-time event streaming |
| Engine Core | trading-engine | 0.1.0 | Local workspace dependency |

---

## Current Status

### ✅ Completed (Scaffolding Phase)

**Infrastructure:**
- [x] Cargo workspace structure
- [x] Project directory layout
- [x] Dependency configuration
- [x] Build system verified
- [x] README and documentation

**Server Core:**
- [x] Basic server setup (`lib.rs`)
- [x] Entry point (`main.rs`)
- [x] Server configuration struct
- [x] Router builder
- [x] Application state pattern

**Middleware:**
- [x] CORS middleware (permissive for development)
- [x] Request logging/tracing (tower-http)
- [x] Error handling foundations

**Endpoints Implemented:**
- [x] `GET /health` - Server health check
- [x] `GET /api/engine/health` - Engine health status

**Testing:**
- [x] Unit tests for health endpoints
- [x] Configuration tests
- [x] Router build tests
- [x] 4 tests passing

**Verification:**
- [x] Server starts successfully
- [x] Endpoints return valid JSON
- [x] Request logging works
- [x] Build system works in workspace

### 📋 Pending (Implementation Phase)

**Phase 6, Step 6-7: REST API Endpoints**
- [ ] Integrate TradingEngine into AppState
- [ ] Implement engine summary endpoint
- [ ] Implement runner list endpoint
- [ ] Implement runner details endpoint
- [ ] Implement snapshot query endpoint
- [ ] Implement price history endpoint
- [ ] Implement add runner endpoint
- [ ] Implement remove runner endpoint
- [ ] Error handling and validation
- [ ] Integration tests

**Phase 6, Step 8: WebSocket Streaming**
- [ ] WebSocket connection handler
- [ ] Event subscription system
- [ ] Client-side filtering
- [ ] Event throttling/batching
- [ ] Connection lifecycle management
- [ ] WebSocket integration tests

**Phase 6, Steps 9-10: Polish & Documentation**
- [ ] End-to-end integration tests
- [ ] Performance benchmarks
- [ ] API documentation (OpenAPI/Swagger)
- [ ] Deployment guide
- [ ] Frontend integration examples

---

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (React/Vue)                      │
│  - Dashboard UI                                              │
│  - Charts & Visualizations                                   │
│  - Real-time Updates                                         │
└──────────────┬───────────────────────────┬───────────────────┘
               │ HTTP (REST)               │ WebSocket
               ↓                           ↓
┌──────────────────────────────────────────────────────────────┐
│              Web Backend (axum + tokio)                       │
│  ┌────────────────────┐  ┌─────────────────────────────┐    │
│  │   REST API Routes  │  │  WebSocket Handler          │    │
│  │  - Engine stats    │  │  - Event subscription       │    │
│  │  - Runner CRUD     │  │  - Real-time streaming      │    │
│  │  - Snapshots       │  │  - Client filtering         │    │
│  └─────────┬──────────┘  └──────────┬──────────────────┘    │
│            │                        │                        │
│            └────────┬───────────────┘                        │
│                     ↓                                        │
│            ┌─────────────────┐                               │
│            │   AppState      │                               │
│            │  - TradingEngine│                               │
│            │  - EventAggr.   │                               │
│            └─────────┬───────┘                               │
└──────────────────────┼────────────────────────────────────────┘
                       │
                       ↓
┌──────────────────────────────────────────────────────────────┐
│           TradingEngine (engine-core)                        │
│  ┌──────────────────┐  ┌──────────────────┐                 │
│  │  Introspection   │  │  Event System    │                 │
│  │  - get_snapshot  │  │  - Event stream  │                 │
│  │  - get_history   │  │  - Subscribers   │                 │
│  └──────────────────┘  └──────────────────┘                 │
│                                                               │
│  ┌────────────────────────────────────────┐                 │
│  │  SymbolRunner 1  │  SymbolRunner 2  │ ...               │
│  │  (BTC-EMA)       │  (ETH-RSI)       │                    │
│  └────────────────────────────────────────┘                 │
└──────────────────────────────────────────────────────────────┘
```

### Request Flow

**REST API Request:**
```
1. Client → HTTP GET /api/runners/btc_ema/snapshot
2. axum Router → extract_runner_snapshot(runner_id, State(app_state))
3. Handler → app_state.engine.get_runner_snapshot("btc_ema").await
4. TradingEngine → Send command to runner → Receive snapshot
5. Handler → Serialize snapshot to JSON
6. axum → Return JSON response
```

**WebSocket Event Stream:**
```
1. Client → WS /ws (upgrade HTTP to WebSocket)
2. Handler → Accept connection
3. Handler → Subscribe to engine.event_aggregator
4. Loop:
   - Event → Aggregator → WebSocket handler
   - Filter events based on client subscription
   - Serialize to JSON
   - Send via WebSocket
5. Client disconnects → Cleanup subscription
```

### Directory Structure

```
web-backend/
├── Cargo.toml                  # Dependencies and metadata
├── README.md                   # User-facing documentation
├── BACKEND_PLAN.md            # This file
├── .gitignore
├── src/
│   ├── main.rs                # Entry point, server startup
│   ├── lib.rs                 # Core server, router, middleware
│   │
│   ├── routes/                # HTTP route handlers
│   │   ├── mod.rs            # Route module exports
│   │   ├── health.rs         # Health check endpoints ✅
│   │   ├── engine.rs         # Engine management endpoints ✅ (partial)
│   │   ├── runners.rs        # Runner CRUD endpoints (TODO)
│   │   └── snapshots.rs      # Snapshot/history endpoints (TODO)
│   │
│   ├── websocket/            # WebSocket handlers
│   │   ├── mod.rs           # WebSocket module exports
│   │   ├── handler.rs       # Connection handler (TODO)
│   │   ├── events.rs        # Event filtering/formatting (TODO)
│   │   └── subscription.rs  # Subscription management (TODO)
│   │
│   ├── state.rs             # AppState and shared state (TODO)
│   ├── error.rs             # Error types and handlers (TODO)
│   └── config.rs            # Configuration loading (TODO)
│
└── tests/                    # Integration tests
    ├── health_tests.rs       # Health endpoint tests (TODO)
    ├── api_tests.rs          # REST API tests (TODO)
    └── websocket_tests.rs    # WebSocket tests (TODO)
```

---

## Implementation Plan

### Phase 1: Application State & Engine Integration (Est: 2-3 hours)

**Goal:** Connect the web backend to the TradingEngine

**Tasks:**
1. Create `src/state.rs` with AppState structure
   ```rust
   pub struct AppState {
       engine: Arc<Mutex<TradingEngine>>,
       event_aggregator: Arc<Mutex<EventAggregator>>,
   }
   ```

2. Update `main.rs` to initialize TradingEngine
   - Create engine instance
   - Create event aggregator
   - Wrap in Arc<Mutex<>>
   - Pass to router as state

3. Update `lib.rs` router to accept AppState
   - Update `build_router(state: AppState)`
   - Pass state to routes via `with_state()`

4. Create `src/error.rs` for error handling
   - Define `ApiError` enum
   - Implement `IntoResponse` for ApiError
   - Add helpers for common errors

**Tests:**
- Engine initialization in AppState
- State cloning/sharing
- Error type conversions

**Deliverable:** Web server with live TradingEngine instance

---

### Phase 2: REST API - Engine Endpoints (Est: 3-4 hours)

**Goal:** Implement engine-level API endpoints

**Tasks:**

1. **`GET /api/engine/summary`** - Engine overview
   ```rust
   pub struct EngineSummary {
       total_runners: usize,
       healthy_runners: usize,
       runners: Vec<RunnerSummary>,
       uptime_secs: u64,
       timestamp: i64,
   }
   ```

2. **`GET /api/runners`** - List all runners
   ```rust
   pub struct RunnerInfo {
       runner_id: String,
       symbol: String,
       state: String,
       has_position: bool,
       ticks_processed: u64,
       uptime_secs: u64,
   }
   ```

3. Update `routes/engine.rs` with implementations
4. Add validation and error handling
5. Add request/response logging

**Tests:**
- Engine summary with 0 runners
- Engine summary with multiple runners
- Runner list filtering
- Error handling for engine lock failures

**Deliverable:** Working engine query endpoints

---

### Phase 3: REST API - Runner Management (Est: 4-5 hours)

**Goal:** Implement runner CRUD operations

**Tasks:**

1. Create `src/routes/runners.rs`

2. **`GET /api/runners/:id`** - Get runner details
   - Extract runner_id from path
   - Query engine for runner info
   - Return 404 if not found

3. **`GET /api/runners/:id/snapshot`** - Get full snapshot
   - Use `engine.get_runner_snapshot(id).await`
   - Serialize RunnerSnapshot to JSON
   - Add caching headers

4. **`GET /api/runners/:id/history`** - Get price history
   - Optional query param: `?count=N`
   - Use `engine.get_price_history(id, count).await`
   - Return Vec<MarketData> as JSON

5. **`POST /api/runners`** - Add new runner
   ```rust
   pub struct AddRunnerRequest {
       runner_id: String,
       symbol: String,
       strategy_path: String,
       window_size: Option<usize>,
   }
   ```
   - Validate request body
   - Load Lua strategy
   - Call `engine.add_runner_with_config()`
   - Return 201 Created with runner info

6. **`DELETE /api/runners/:id`** - Remove runner
   - Call `engine.remove_runner(id)`
   - Return 204 No Content
   - Return 404 if runner not found

**Tests:**
- Get runner details (success/not found)
- Get snapshot with position/without position
- Get price history (all / last N)
- Add runner (success/validation errors)
- Remove runner (success/not found)
- Concurrent access to engine

**Deliverable:** Full REST API for runner management

---

### Phase 4: WebSocket Event Streaming (Est: 4-5 hours)

**Goal:** Real-time event streaming to clients

**Tasks:**

1. Create `src/websocket/handler.rs`
   - WebSocket upgrade handler
   - Connection acceptance
   - Ping/pong keepalive

2. Create `src/websocket/subscription.rs`
   - Subscribe to EventAggregator
   - Filter events by runner_id
   - Format events as JSON

3. Create `src/websocket/events.rs`
   - Event serialization
   - Event batching/throttling
   - Backpressure handling

4. **`WS /ws`** - WebSocket endpoint
   ```
   Client → Connect → Send subscription message:
   { "type": "subscribe", "runners": ["btc_ema", "eth_rsi"] }

   Server → Stream events:
   { "type": "TickReceived", "runner_id": "btc_ema", "data": {...} }
   { "type": "StateTransition", "runner_id": "btc_ema", "from": "Idle", "to": "Analyzing" }
   ```

5. Add connection lifecycle management
   - Track active connections
   - Cleanup on disconnect
   - Graceful shutdown

**Tests:**
- WebSocket connection/disconnection
- Event subscription/filtering
- Multiple concurrent connections
- Event delivery order
- Backpressure handling

**Deliverable:** Real-time event streaming via WebSocket

---

### Phase 5: Integration Tests & Documentation (Est: 3-4 hours)

**Goal:** Comprehensive testing and documentation

**Tasks:**

1. **Integration Tests**
   - `tests/api_tests.rs` - Full REST API flows
   - `tests/websocket_tests.rs` - WebSocket scenarios
   - End-to-end scenarios (add runner → receive events → query snapshot)

2. **API Documentation**
   - Generate OpenAPI/Swagger spec
   - Add endpoint examples
   - Document error codes
   - Request/response schemas

3. **Performance Benchmarks**
   - Measure endpoint latency
   - WebSocket event throughput
   - Concurrent connection handling
   - Memory usage under load

4. **Deployment Guide**
   - Configuration options
   - Environment variables
   - Reverse proxy setup (nginx)
   - TLS/HTTPS configuration
   - Docker containerization

**Tests:**
- All integration tests passing
- Load tests (100+ concurrent connections)
- Stress tests (1000+ events/sec)

**Deliverable:** Production-ready backend with full documentation

---

## API Specification

### REST Endpoints Summary

| Method | Endpoint | Description | Status |
|--------|----------|-------------|--------|
| GET | `/health` | Server health | ✅ Implemented |
| GET | `/api/engine/health` | Engine health | ✅ Implemented |
| GET | `/api/engine/summary` | Engine summary | 📋 Planned |
| GET | `/api/runners` | List runners | 📋 Planned |
| GET | `/api/runners/:id` | Runner details | 📋 Planned |
| GET | `/api/runners/:id/snapshot` | Runner snapshot | 📋 Planned |
| GET | `/api/runners/:id/history` | Price history | 📋 Planned |
| POST | `/api/runners` | Add runner | 📋 Planned |
| DELETE | `/api/runners/:id` | Remove runner | 📋 Planned |

### WebSocket Endpoints

| Endpoint | Protocol | Description | Status |
|----------|----------|-------------|--------|
| `/ws` | WebSocket | Event streaming | 📋 Planned |

### Response Formats

**Success Response:**
```json
{
  "status": "ok",
  "data": { ... },
  "timestamp": 1766282234
}
```

**Error Response:**
```json
{
  "status": "error",
  "error": {
    "code": "RUNNER_NOT_FOUND",
    "message": "Runner 'btc_ema' not found",
    "details": null
  },
  "timestamp": 1766282234
}
```

### Detailed Endpoint Specifications

#### `GET /api/engine/summary`

**Response:**
```json
{
  "status": "ok",
  "data": {
    "total_runners": 6,
    "healthy_runners": 6,
    "runners": [
      {
        "runner_id": "btc_ema",
        "symbol": "BTCUSDT",
        "state": "InPosition",
        "has_position": true,
        "ticks_processed": 1523,
        "uptime_secs": 3600
      },
      ...
    ],
    "uptime_secs": 3600
  },
  "timestamp": 1766282234
}
```

#### `GET /api/runners/:id/snapshot`

**Response:**
```json
{
  "status": "ok",
  "data": {
    "runner_id": "btc_ema",
    "symbol": "BTCUSDT",
    "current_state": "InPosition",
    "position": {
      "entry_price": 50000.0,
      "quantity": 0.1,
      "side": "Long",
      "entry_timestamp": 1766280000,
      "stop_loss": 49000.0,
      "take_profit": 52000.0,
      "unrealized_pnl": 150.0
    },
    "context": {
      "strings": { "signal": "bullish" },
      "numbers": { "ema_10": 50100.0, "ema_20": 49900.0 },
      "integers": {},
      "booleans": { "strong_trend": true }
    },
    "stats": {
      "ticks_processed": 1523,
      "actions_executed": 12,
      "errors": 0,
      "avg_tick_duration": { "secs": 0, "nanos": 850000 },
      "min_tick_duration": { "secs": 0, "nanos": 120000 },
      "max_tick_duration": { "secs": 0, "nanos": 3500000 }
    },
    "uptime_secs": 3600,
    "snapshot_timestamp": 1766282234
  },
  "timestamp": 1766282234
}
```

#### `POST /api/runners`

**Request:**
```json
{
  "runner_id": "sol_ema",
  "symbol": "SOLUSDT",
  "strategy_path": "lua-strategies/examples/ema_crossover.lua",
  "window_size": 200
}
```

**Response (201 Created):**
```json
{
  "status": "ok",
  "data": {
    "runner_id": "sol_ema",
    "symbol": "SOLUSDT",
    "state": "Idle",
    "created_at": 1766282234
  },
  "timestamp": 1766282234
}
```

#### WebSocket Message Format

**Client → Server (Subscribe):**
```json
{
  "type": "subscribe",
  "runners": ["btc_ema", "eth_rsi"],
  "event_types": ["StateTransition", "PositionOpened", "PositionClosed"]
}
```

**Server → Client (Event):**
```json
{
  "type": "StateTransition",
  "runner_id": "btc_ema",
  "timestamp": 1766282234,
  "data": {
    "from": "Idle",
    "to": "Analyzing",
    "trigger": "OpportunityDetected"
  }
}
```

---

## WebSocket Design

### Connection Lifecycle

```
1. Client connects to ws://localhost:3000/ws
2. Server accepts WebSocket upgrade
3. Server creates subscription to EventAggregator
4. Client sends subscription message (optional filtering)
5. Server starts streaming events
6. Loop: Events → Filter → JSON → Send
7. Client disconnects
8. Server cleanup: Unsubscribe from aggregator
```

### Event Filtering

Clients can filter events by:
- **runner_id** - Only events from specific runners
- **event_type** - Only specific event types (e.g., only position events)
- **symbol** - Only events for specific symbols

**Default:** All events from all runners

### Throttling & Backpressure

**Problem:** High-frequency events (100-1000/sec) can overwhelm WebSocket

**Solutions:**
1. **Event Batching** - Batch events every 100ms
2. **Sampling** - Send every Nth tick event
3. **Prioritization** - Always send important events (positions), sample ticks
4. **Client-side buffering** - Queue events if client is slow

### Error Handling

**Scenarios:**
- Client disconnects unexpectedly → Cleanup subscription
- Network error → Close connection gracefully
- Engine shutdown → Notify all clients, close connections
- Invalid subscription message → Send error, keep connection open

---

## Testing Strategy

### Unit Tests

**Location:** `src/` (inline with modules)

**Coverage:**
- Route handlers (mocked engine)
- Error type conversions
- Request validation
- Response serialization
- WebSocket message formatting

**Example:**
```rust
#[tokio::test]
async fn test_get_runner_snapshot_not_found() {
    let state = AppState::with_mock_engine();
    let response = get_runner_snapshot(
        Path("nonexistent".to_string()),
        State(state)
    ).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
```

### Integration Tests

**Location:** `tests/`

**Coverage:**
- Full server lifecycle (start → request → shutdown)
- Multi-endpoint flows
- Concurrent requests
- WebSocket connections
- Engine integration

**Example:**
```rust
#[tokio::test]
async fn test_add_runner_then_query_snapshot() {
    let server = spawn_test_server().await;

    // Add runner
    let response = server.post("/api/runners")
        .json(&add_runner_request)
        .send()
        .await?;
    assert_eq!(response.status(), 201);

    // Query snapshot
    let snapshot = server.get("/api/runners/test_runner/snapshot")
        .send()
        .await?
        .json::<RunnerSnapshot>()
        .await?;
    assert_eq!(snapshot.runner_id, "test_runner");
}
```

### Load Tests

**Tool:** `wrk` or `artillery`

**Scenarios:**
- 100 concurrent HTTP requests
- 100 concurrent WebSocket connections
- 1000 events/sec throughput
- Sustained load for 5 minutes

**Metrics:**
- Latency (p50, p95, p99)
- Throughput (req/sec)
- Error rate
- Memory usage
- CPU usage

---

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| REST API Latency (p95) | <10ms | wrk benchmark |
| REST API Latency (p99) | <50ms | wrk benchmark |
| WebSocket Event Latency | <5ms | Custom benchmark |
| Concurrent Connections | 1000+ | Load test |
| Events/sec Throughput | 10,000+ | Event generator |
| Memory per Connection | <1MB | profiling |
| CPU Usage (100 runners) | <50% | htop |

### Optimization Strategies

1. **Minimize Lock Contention**
   - Use `Arc<Mutex<>>` sparingly
   - Consider RwLock for read-heavy workloads
   - Lock-free data structures where possible

2. **Reduce Allocations**
   - Reuse buffers
   - Object pooling for event messages
   - Avoid cloning large structures

3. **Async Efficiency**
   - Avoid blocking operations in async context
   - Use `spawn_blocking` for CPU-intensive work
   - Batch database/file operations

4. **WebSocket Optimization**
   - Message compression (permessage-deflate)
   - Binary encoding (MessagePack instead of JSON)
   - Event batching

---

## Security Considerations

### Authentication & Authorization

**Current:** None (development only)

**Production:**
- API key authentication
- JWT tokens
- Role-based access control (read-only vs admin)

### Input Validation

- Validate all request bodies
- Sanitize path parameters
- Limit request size
- Rate limiting per client

### CORS Configuration

**Development:** Permissive (Any origin)

**Production:**
- Whitelist specific frontend origins
- Restrict allowed methods
- Set appropriate headers

### TLS/HTTPS

**Production Requirements:**
- TLS 1.2+ only
- Valid SSL certificate
- HSTS headers
- Secure WebSocket (wss://)

### Error Handling

**Do NOT expose:**
- Internal error details
- Stack traces
- File paths
- Database errors

**Do expose:**
- Generic error messages
- Error codes for client handling
- Request ID for debugging

---

## Deployment

### Configuration

**Environment Variables:**
```bash
TRADING_BACKEND_HOST=0.0.0.0
TRADING_BACKEND_PORT=3000
RUST_LOG=trading_web_backend=info,tower_http=debug
CORS_ALLOWED_ORIGINS=https://dashboard.example.com
```

**Config File (config.toml):**
```toml
[server]
host = "0.0.0.0"
port = 3000

[cors]
allowed_origins = ["https://dashboard.example.com"]
max_age = 3600

[engine]
max_runners = 100
data_source = "binance"
```

### Reverse Proxy (nginx)

```nginx
server {
    listen 80;
    server_name api.trading-system.example.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p trading-web-backend

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/trading-web-backend /usr/local/bin/
EXPOSE 3000
CMD ["trading-web-backend"]
```

### Systemd Service

```ini
[Unit]
Description=Trading System Web Backend
After=network.target

[Service]
Type=simple
User=trading
WorkingDirectory=/opt/trading-system
ExecStart=/opt/trading-system/trading-web-backend
Restart=on-failure
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

---

## Monitoring & Observability

### Metrics to Track

**Server Metrics:**
- Request count by endpoint
- Response time by endpoint
- Error rate by endpoint
- Active WebSocket connections

**Engine Metrics:**
- Total runners
- Healthy runners
- Events/sec processed
- Engine uptime

**Resource Metrics:**
- CPU usage
- Memory usage
- Network I/O
- Open file descriptors

### Logging

**Structured Logging (tracing):**
```rust
tracing::info!(
    runner_id = %runner_id,
    endpoint = "/api/runners/:id/snapshot",
    duration_ms = duration.as_millis(),
    "Snapshot query completed"
);
```

**Log Levels:**
- ERROR: Critical failures
- WARN: Degraded performance, recoverable errors
- INFO: Important events (new runner, position opened)
- DEBUG: Request/response details
- TRACE: Internal flow (for development)

### Health Checks

**Liveness:** `GET /health` → 200 OK (server is running)

**Readiness:** `GET /api/engine/health` → 200 OK (engine is ready)

---

## Dependencies & Versions

```toml
[dependencies]
# Core
trading-engine = { path = "../engine-core" }

# Web framework
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "cors", "trace"] }

# Async runtime
tokio = { version = "1.43", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Time
chrono = "0.4"

[dev-dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio-test = "0.4"
```

---

## Timeline Estimate

| Phase | Description | Estimated Time | Dependencies |
|-------|-------------|----------------|--------------|
| ✅ 0 | Scaffolding | 2 hours | None |
| 1 | App State & Integration | 2-3 hours | Phase 0 |
| 2 | Engine Endpoints | 3-4 hours | Phase 1 |
| 3 | Runner Management | 4-5 hours | Phase 2 |
| 4 | WebSocket Streaming | 4-5 hours | Phase 3 |
| 5 | Tests & Documentation | 3-4 hours | Phase 4 |

**Total Estimated Time:** 18-23 hours

**Current Status:** Phase 0 complete (2 hours spent)
**Remaining:** 16-21 hours

---

## Open Questions

1. **Authentication Strategy**
   - Do we need authentication for MVP?
   - API keys vs JWT tokens?
   - Single user or multi-tenant?

2. **Frontend Framework**
   - React, Vue, or Svelte?
   - TypeScript or JavaScript?
   - UI component library (MUI, Ant Design, etc.)?

3. **Data Persistence**
   - Should backend store runner configurations?
   - Database for historical data?
   - Session persistence across restarts?

4. **Deployment Environment**
   - Cloud (AWS/GCP/Azure) or self-hosted?
   - Single server or distributed?
   - Load balancer needed?

5. **Rate Limiting**
   - Per-client limits?
   - Endpoint-specific limits?
   - Implementation (tower middleware vs external)?

---

## Success Criteria

### Phase 1-3 (REST API)
- [ ] All endpoints return valid JSON
- [ ] Error handling works correctly
- [ ] Engine integration functional
- [ ] Can add/remove runners via API
- [ ] Snapshots include all expected data
- [ ] 20+ integration tests passing

### Phase 4 (WebSocket)
- [ ] WebSocket connections stable
- [ ] Events stream in real-time (<5ms latency)
- [ ] Client filtering works
- [ ] Handles 100+ concurrent connections
- [ ] Graceful connection lifecycle

### Phase 5 (Production Ready)
- [ ] Load tests pass (1000+ req/sec)
- [ ] Memory usage stable under load
- [ ] Complete API documentation
- [ ] Deployment guide verified
- [ ] All tests passing (unit + integration + load)

---

## References

- [WEB_APP_ARCHITECTURE.md](../WEB_APP_ARCHITECTURE.md) - Overall web app design
- [trading-system-roadmap.md](../trading-system-roadmap.md) - Full project roadmap
- [Phase 6 Event System Report](../changes/2025-12-20-phase6-event-system.md) - Event system details
- [Phase 6 State Introspection Report](../changes/2025-12-20-phase6-state-introspection.md) - Introspection API details
- [axum Documentation](https://docs.rs/axum/latest/axum/) - Web framework reference
- [tower-http Documentation](https://docs.rs/tower-http/latest/tower_http/) - Middleware reference

---

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-12-20 | 0.1.0 | Initial plan document | Claude |

---

**End of Backend Plan**
