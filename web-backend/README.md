# Trading System Web Backend

HTTP and WebSocket server for the Trading Simulator system.

## Features

- **REST API** - Query engine status, runner snapshots, and price history
- **WebSocket Streaming** - Real-time event notifications
- **CORS Support** - Configured for frontend development
- **Request Logging** - Tracing middleware for debugging
- **Health Checks** - Server and engine health endpoints

## Quick Start

```bash
# Run the server
cargo run -p trading-web-backend

# Run tests
cargo test -p trading-web-backend

# Build release
cargo build -p trading-web-backend --release
```

The server listens on `http://127.0.0.1:3000` by default.

## API Endpoints

### Health Checks

**`GET /health`**
- Basic server health check
- Returns: `{ status, timestamp, version }`

**`GET /api/engine/health`**
- Trading engine health status
- Returns: `{ status, runners_count, healthy_runners, timestamp }`

### Coming Soon (Phase 6, Steps 6-8)

- `GET /api/engine/summary` - Engine summary
- `GET /api/runners` - List all runners
- `GET /api/runners/:id` - Runner details
- `GET /api/runners/:id/snapshot` - Get runner snapshot
- `GET /api/runners/:id/history` - Get price history
- `POST /api/runners` - Add runner
- `DELETE /api/runners/:id` - Remove runner
- `WS /ws` - WebSocket event streaming

## Testing

```bash
# Start server
cargo run -p trading-web-backend

# Test endpoints
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/api/engine/health
```

## Configuration

Server configuration is defined in `ServerConfig`:

```rust
pub struct ServerConfig {
    pub host: String,      // Default: "127.0.0.1"
    pub port: u16,         // Default: 3000
    pub enable_cors: bool, // Default: true
}
```

## Architecture

```
web-backend/
├── src/
│   ├── main.rs           # Server entry point
│   ├── lib.rs            # Core server setup
│   ├── routes/           # HTTP handlers
│   │   ├── mod.rs
│   │   ├── health.rs     # Health check endpoints
│   │   └── engine.rs     # Engine API endpoints
│   └── websocket/        # WebSocket handlers (TBD)
│       └── mod.rs
└── Cargo.toml
```

## Dependencies

- **axum** - Web framework
- **tower-http** - Middleware (CORS, tracing)
- **tokio** - Async runtime
- **serde/serde_json** - Serialization
- **tracing** - Logging/diagnostics
- **trading-engine** - Core trading engine (workspace dependency)

## Development

The backend is part of a Cargo workspace with `engine-core`. Changes to the engine are automatically reflected in the backend.

See [WEB_APP_ARCHITECTURE.md](../WEB_APP_ARCHITECTURE.md) for detailed API design.
