# Web Application Expansion Plan

**Date**: December 21, 2025
**Phase**: 6 - Enhanced Web Interface
**Goal**: Transform the web app into a comprehensive trading system development environment

## Overview

Expand the current web application from basic monitoring to a full-featured trading system IDE with three main sections:

1. **Runner Manager** - Start/stop/spawn runners with symbol and strategy selection
2. **Strategy Editor** - Write and test Lua strategies with syntax highlighting
3. **Tuner/Backtester** - Test strategies on historical data across multiple timeframes

## Current State

### Existing Features (Phase 6 - Steps 1-10) ✅ COMPLETE
- ✅ Dashboard with engine summary and enhanced runner list table
- ✅ Runner detail page with price charts
- ✅ Enhanced runner creation form with dropdowns
- ✅ Live Binance US data streaming
- ✅ REST API with 14 endpoints (engine, runners, control, reference data)
- ✅ Auto-refresh (5s dashboard, 2s runner details)
- ✅ **Runner control system**: Pause/Resume/Stop functionality
- ✅ **Strategy dropdown**: Dynamic listing from lua-strategies/
- ✅ **Symbol dropdown**: 18 curated symbols across 4 categories
- ✅ **Status indicators**: Color-coded runner states (Running/Paused/Stopped)
- ✅ **Enhanced UI**: RunnerListTable with control buttons, status badges, uptime

### Completed in Phase 6 Step 10
- ✅ Runner pause/resume/stop control (backend + frontend)
- ✅ RunnerStatus enum and state management
- ✅ Strategy file listing and selection
- ✅ Symbol categorization and dropdown
- ✅ Enhanced runner list table with actions
- ✅ Real-time status updates

### Remaining Limitations (Future Phases)
- ❌ No code editor for strategies (Phase 7 - Priority 3)
- ❌ No indicator visibility in UI (Phase 6 Step 10 - Priority 4)
- ❌ No state transition logging/visualization (Phase 6 Step 10 - Priority 4)
- ❌ No historical backtesting (Phase 7 - Priorities 5-7)
- ❌ Limited chart types (Phase 6 Step 10 - Priority 4)
- ❌ No live Lua syntax validation (Phase 7 - Priority 2)

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Web Frontend (React)                      │
├─────────────┬─────────────────┬─────────────────────────────┤
│   Runner    │    Strategy     │         Tuner               │
│   Manager   │    Editor       │      (Backtester)           │
└─────┬───────┴────────┬────────┴──────────┬──────────────────┘
      │                │                   │
      ↓                ↓                   ↓
┌─────────────────────────────────────────────────────────────┐
│              REST API (Axum Backend)                         │
│  - Runner CRUD          - Strategy CRUD  - Backtest API     │
│  - Start/Stop/Pause     - Validation     - Historical Data  │
│  - State introspection  - Syntax check   - Performance      │
└─────┬───────────────────┬──────────────────┬────────────────┘
      │                   │                  │
      ↓                   ↓                  ↓
┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐
│   Trading    │  │  Strategy    │  │   Backtest Engine    │
│   Engine     │  │  Storage     │  │  (Historical Mode)   │
└──────────────┘  └──────────────┘  └──────────────────────┘
```

## Section 1: Runner Manager

### Features

#### 1.1 Runner List View (Enhanced Dashboard)
**Current**: Basic list with runner ID and symbol
**Target**:
- Table view with columns: Status, Runner ID, Symbol, Strategy, State, Position, P&L, Uptime, Actions
- Status indicator: 🟢 Running, 🟡 Paused, 🔴 Stopped, ⚪ Error
- Quick actions: View, Pause/Resume, Stop, Delete
- Bulk actions: Start all, Stop all, Delete selected
- Filters: By symbol, by strategy, by state, by status
- Search: By runner ID or symbol

#### 1.2 Runner Creation (Enhanced Form) ✅ COMPLETE
**Previous**: Manual text input for strategy path
**Implemented**:
- ✅ **Symbol Selection**: Dropdown with 18 curated symbols across 4 categories
  - Crypto - Major (5): BTC, ETH, BNB, SOL, XRP
  - Crypto - Alt (5): ADA, DOGE, AVAX, DOT, MATIC
  - Stocks - Tech (5): AAPL, MSFT, GOOGL, AMZN, TSLA
  - Forex - Major (3): EUR/USD, GBP/USD, USD/JPY
- ✅ **Strategy Selection**: Dropdown showing all available strategies from `lua-strategies/`
  - Multi-path resolution (current dir, parent, grandparent)
  - Strategy metadata (name, path, category)
- **Future Enhancements**:
  - Preview pane showing strategy code
  - "Create New Strategy" button → opens Strategy Editor
  - Advanced configuration (window size, risk parameters)
  - Validation for parameters

#### 1.3 Runner Control ✅ COMPLETE
**Implemented Endpoints**:
- ✅ `POST /api/runners/:id/pause` - Pause a running runner (stops processing ticks)
- ✅ `POST /api/runners/:id/resume` - Resume a paused runner
- ✅ `POST /api/runners/:id/stop` - Stop a running runner
- ✅ `GET /api/strategies` - List available strategies
- ✅ `GET /api/symbols` - List curated symbols

**Backend Implementation**:
- ✅ RunnerStatus enum: Running, Paused, Stopped
- ✅ Control commands via channels (Pause, Resume, Stop)
- ✅ Graceful pause/resume (preserves all state)
- ✅ Non-blocking command processing with tokio::select!
- ✅ Status included in RunnerSnapshot

**Frontend Implementation**:
- ✅ Enhanced RunnerListTable with control buttons
- ✅ Status color indicators (green/yellow/red)
- ✅ Conditional button rendering based on status
- ✅ Loading states for control actions
- ✅ Error handling and user feedback

## Section 2: Strategy Editor

### Features

#### 2.1 Strategy File Management
**New Endpoints Needed**:
- `GET /api/strategies` - List all available strategies
- `GET /api/strategies/:name` - Get strategy code
- `POST /api/strategies` - Create new strategy
- `PUT /api/strategies/:name` - Update strategy
- `DELETE /api/strategies/:name` - Delete strategy
- `POST /api/strategies/:name/validate` - Validate Lua syntax and structure

**Response Format**:
```json
{
  "name": "ema_crossover",
  "path": "lua-strategies/examples/ema_crossover.lua",
  "size": 1234,
  "modified": 1703123456,
  "indicators_used": ["ema"],
  "context_fields": ["prev_ema_10", "prev_ema_20"],
  "is_valid": true
}
```

#### 2.2 Code Editor UI
**Technology**: Monaco Editor (same as VS Code)

**Features**:
- Syntax highlighting for Lua
- Auto-completion for:
  - Lua keywords
  - Trading API functions (indicators, actions, market_data fields)
  - Context fields (autocomplete from declared fields)
- Error highlighting (syntax errors, missing functions)
- Line numbers
- Minimap
- Bracket matching
- Find/Replace

#### 2.3 Strategy Template System
**Built-in Templates**:
1. **Empty Strategy** - Skeleton with 3 required functions
2. **EMA Crossover** - Current example
3. **RSI Mean Reversion** - Current example
4. **Range Breakout** - Current example
5. **Bollinger Band Bounce** - New template
6. **MACD Strategy** - New template

**Template Picker**: Modal with preview and description

#### 2.4 Strategy Metadata
**Top of editor**: Form fields for:
- Strategy name
- Description
- Author
- Indicators used (multi-select: SMA, EMA, RSI, MACD, BB)
- Context fields needed (array of field names)
- Recommended symbols
- Risk level (Low, Medium, High)

**Stored as Lua comments**:
```lua
-- @name EMA Crossover
-- @description Simple moving average crossover strategy
-- @author Trading System
-- @indicators ema
-- @context prev_ema_10,prev_ema_20
-- @symbols BTCUSDT,ETHUSDT
-- @risk low
```

#### 2.5 Live Validation
**As you type**:
- Parse Lua syntax (use Lua parser library)
- Check required functions exist (detect_opportunity, filter_commitment, manage_position)
- Verify indicator calls match available indicators
- Warn about unused context fields
- Suggest best practices (e.g., nil checks before using indicators)

**Validation Errors Panel**:
- List of errors with line numbers
- Click to jump to line
- Severity: Error (red), Warning (yellow), Info (blue)

## Section 3: Tuner (Backtester)

### Features

#### 3.1 Backtest Configuration
**UI Form**:
- **Strategy**: Select from dropdown
- **Symbol**: Select symbol to test
- **Time Range**:
  - Preset: Last 24h, 7d, 30d, 90d, 1y
  - Custom: Start date/time, End date/time
- **Randomized Testing**:
  - Number of runs (e.g., 10 random start points)
  - Duration per run (e.g., 24 hours each)
- **Advanced**:
  - Initial capital
  - Commission rate
  - Slippage model
  - Window size

#### 3.2 Historical Data Integration
**Backend Requirements**:
- Fetch historical klines from Binance API
  - Endpoint: `/api/v3/klines`
  - Store in cache to avoid re-fetching
- Create `HistoricalFeed` struct implementing `MarketDataSource`
- Support different timeframes (1m, 5m, 15m, 1h, 4h, 1d)

**New Backend Module**: `engine-core/src/backtest/`
- `historical_feed.rs` - Binance historical data fetching
- `backtest_runner.rs` - Run strategy on historical data
- `performance.rs` - Calculate metrics (Sharpe, drawdown, win rate)

#### 3.3 Backtest Execution
**Process**:
1. User submits backtest config
2. Backend fetches historical data (or uses cache)
3. For each random run:
   - Pick random start time within range
   - Create SymbolRunner with HistoricalFeed
   - Run until end time or duration limit
   - Collect all trades and metrics
4. Aggregate results across all runs
5. Return performance report

**New Endpoints**:
- `POST /api/backtest/run` - Start backtest
- `GET /api/backtest/:id/status` - Check progress
- `GET /api/backtest/:id/results` - Get results
- `DELETE /api/backtest/:id` - Cancel running backtest

**Backtest ID**: UUID for tracking long-running backtests

#### 3.4 Performance Metrics
**Calculated Metrics**:
- **Returns**: Total return %, annualized return
- **Risk**: Sharpe ratio, Sortino ratio, max drawdown
- **Trade Stats**:
  - Total trades, winning trades, losing trades
  - Win rate %, average win, average loss
  - Profit factor (gross profit / gross loss)
  - Largest win, largest loss
- **Timing**: Average hold time, longest hold time
- **Consistency**:
  - Standard deviation of returns
  - % of runs profitable
  - Best run, worst run

**Response Format**:
```json
{
  "backtest_id": "uuid",
  "strategy": "ema_crossover",
  "symbol": "BTCUSDT",
  "start_time": 1703000000,
  "end_time": 1703100000,
  "num_runs": 10,
  "metrics": {
    "total_return_pct": 12.5,
    "sharpe_ratio": 1.8,
    "max_drawdown_pct": 5.2,
    "win_rate_pct": 62.0,
    "total_trades": 45,
    "avg_hold_time_mins": 120
  },
  "runs": [
    {
      "run_id": 1,
      "start_time": 1703012345,
      "return_pct": 8.3,
      "trades": 4,
      "pnl": 830.0
    }
  ],
  "trades": [
    {
      "run_id": 1,
      "entry_time": 1703012400,
      "exit_time": 1703014000,
      "side": "Long",
      "entry_price": 42000.0,
      "exit_price": 42500.0,
      "quantity": 0.1,
      "pnl": 50.0,
      "pnl_pct": 1.19
    }
  ]
}
```

#### 3.5 Results Visualization
**Charts**:
1. **Equity Curve**:
   - Line chart showing portfolio value over time
   - Overlay all runs with average highlighted
2. **Drawdown Chart**:
   - Area chart showing underwater periods
3. **Returns Distribution**:
   - Histogram of trade returns
4. **Monthly Returns Heatmap**:
   - Calendar view of returns by month
5. **Trade Analysis**:
   - Scatter plot: Hold time vs P&L
   - Bar chart: Winning vs losing trades by hour of day

**Tables**:
1. **Run Summary**: Table of all runs with key metrics
2. **Trade Log**: Detailed list of all trades
3. **Statistics Panel**: Key metrics in card layout

## Section 4: Enhanced Runner Detail View

### Features

#### 4.1 Indicators Panel
**Display**:
- List all indicators used by strategy
- For each indicator:
  - Name (e.g., "EMA 10", "RSI 14")
  - Current value
  - Previous value (for comparison)
  - Sparkline chart (last 20 values)
  - Color coding: Green (bullish), Red (bearish), Gray (neutral)

**Example**:
```
Indicators Used:
┌─────────────────────────────────────┐
│ EMA 10      49,650.23  ↑ +0.15%    │
│ ▁▂▃▄▅▆▇█▇▆▅▄▃▂▁                     │
├─────────────────────────────────────┤
│ EMA 20      49,580.45  ↓ -0.08%    │
│ ▃▄▅▆▇▇▇▆▅▄▃▂▁▁▁                     │
├─────────────────────────────────────┤
│ RSI 14      62.3       Neutral      │
│ ▄▅▆▇▆▅▄▃▄▅▆▇▆▅▄                     │
└─────────────────────────────────────┘
```

**Backend Changes**:
- Add `get_indicator_values()` to SymbolRunner
- Store last N indicator calculations
- Include in snapshot response

#### 4.2 State Logic Viewer
**Display**: Real-time log of strategy decisions

**Format**:
```
Tick #145 - BTCUSDT @ 49,650.23 (2025-12-21 10:15:23)
├─ State: Idle
├─ Function: detect_opportunity()
├─ EMA 10: 49,650.23
├─ EMA 20: 49,580.45
├─ Logic: EMA 10 > EMA 20 (bullish crossover detected)
└─ Result: Signal detected → Transition to Analyzing

Tick #146 - BTCUSDT @ 49,655.10 (2025-12-21 10:15:24)
├─ State: Analyzing
├─ Function: filter_commitment()
├─ Signal: bullish, confidence 0.8
├─ Logic: Confidence > 0.7 threshold
└─ Result: Enter long @ 49,655.10, quantity 0.1
```

**Implementation**:
- Add logging to Lua strategy execution
- Capture function calls, variable values, decisions
- Store in circular buffer (last 100 ticks)
- New endpoint: `GET /api/runners/:id/logic-log`

#### 4.3 Advanced Charting
**Multiple Chart Types**:
1. **Candlestick Chart** (current) - OHLC data
2. **Indicators Overlay** - EMA, SMA, Bollinger Bands on price chart
3. **Oscillator Panel** - RSI, MACD in separate pane below
4. **Volume Bars** - Volume histogram at bottom
5. **Position Markers** - Entry/exit points marked on chart

**Chart Controls**:
- Timeframe selector: 1m, 5m, 15m, 1h, 4h, 1d
- Zoom: Mouse wheel or pinch
- Pan: Click and drag
- Crosshair: Hover to see exact values
- Drawing tools: Trend lines, support/resistance (future)

**Technology**: Consider upgrading from Recharts to:
- **TradingView Lightweight Charts** - Better performance, more features
- **Plotly** - More interactive, publication-quality

#### 4.4 Position Details (Enhanced)
**Current**: Basic entry price, quantity, P&L
**Target**:
- Entry/exit markers on chart
- Position history table
- Per-position metrics:
  - Hold time
  - Max favorable excursion (MFE)
  - Max adverse excursion (MAE)
  - Exit reason (stop loss, take profit, strategy signal)
- Real-time P&L updates with color animation

## Implementation Plan

### Phase 6 - Step 10: Enhanced Frontend Features ✅ COMPLETE

#### Priority 1: Runner Manager Enhancements ✅ COMPLETE
1. ✅ Add runner control API endpoints (pause/resume/stop)
2. ✅ Implement runner status states in backend (RunnerStatus enum)
3. ✅ Build enhanced runner list table UI (RunnerListTable component)
4. ✅ Add strategy dropdown to creation form (with multi-path resolution)
5. ✅ Add symbol dropdown to creation form (18 curated symbols)
6. ✅ Implement runner control buttons (pause/resume/stop with loading states)

**Actual Implementation**: ~1,000 LOC (600 backend, 400 frontend)
**Tests**: 16 (6 backend snapshot, 10 frontend manual)

#### Priority 2: Strategy File Management
1. Add strategy CRUD endpoints
2. Build strategy list API
3. Implement Lua syntax validator
4. Create strategy metadata parser

**Estimated LOC**: ~400 (350 backend, 50 frontend)
**Tests**: 10 (8 backend, 2 frontend)

#### Priority 3: Strategy Editor UI
1. Integrate Monaco Editor
2. Build template picker
3. Add live validation
4. Implement save/load functionality
5. Create error panel

**Estimated LOC**: ~800 frontend
**Tests**: 12 manual

#### Priority 4: Enhanced Runner Detail
1. Add indicator tracking to SymbolRunner
2. Create indicators panel UI
3. Implement state logic logging
4. Build logic viewer UI
5. Upgrade charts (consider TradingView Lightweight Charts)

**Estimated LOC**: ~700 (200 backend, 500 frontend)
**Tests**: 10 (4 backend, 6 frontend)

### Phase 7: Backtesting System (Week 9-10)

#### Priority 5: Historical Data System
1. Create HistoricalFeed implementation
2. Add Binance historical data fetching
3. Implement data caching
4. Build backtest runner engine

**Estimated LOC**: ~1,200 backend
**Tests**: 20 (integration tests with real historical data)

#### Priority 6: Performance Analytics
1. Implement metrics calculation
2. Create performance report generator
3. Build trade analyzer
4. Add statistical functions (Sharpe, Sortino, drawdown)

**Estimated LOC**: ~800 backend
**Tests**: 25 (unit tests for each metric)

#### Priority 7: Backtesting UI
1. Build backtest configuration form
2. Create results visualization
3. Implement equity curve chart
4. Add trade log table
5. Build metrics dashboard

**Estimated LOC**: ~1,000 frontend
**Tests**: 15 manual

## API Endpoints Summary

### New Endpoints Required

**Runner Control**:
- `POST /api/runners/:id/start`
- `POST /api/runners/:id/pause`
- `POST /api/runners/:id/resume`
- `POST /api/runners/:id/stop`
- `GET /api/runners/:id/indicators`
- `GET /api/runners/:id/logic-log`

**Strategy Management**:
- `GET /api/strategies`
- `GET /api/strategies/:name`
- `POST /api/strategies`
- `PUT /api/strategies/:name`
- `DELETE /api/strategies/:name`
- `POST /api/strategies/:name/validate`

**Backtesting**:
- `POST /api/backtest/run`
- `GET /api/backtest/:id/status`
- `GET /api/backtest/:id/results`
- `DELETE /api/backtest/:id`

**Historical Data**:
- `GET /api/history/:symbol?start=X&end=Y&interval=1m`

## Technology Stack Updates

### Backend Dependencies (add to web-backend/Cargo.toml)
```toml
mlua = "0.9"           # Lua parser for validation
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = "0.4"         # Date/time handling for backtesting
```

### Frontend Dependencies (add to web-frontend/package.json)
```json
{
  "@monaco-editor/react": "^4.6.0",
  "@tradingview/lightweight-charts": "^4.1.0",
  "react-table": "^7.8.0",
  "date-fns": "^3.0.0",
  "recharts": "^2.10.0",
  "react-hot-toast": "^2.4.1"
}
```

## File Structure Changes

```
trading-simulator/
├── engine-core/
│   └── src/
│       ├── backtest/           # NEW
│       │   ├── mod.rs
│       │   ├── historical_feed.rs
│       │   ├── runner.rs
│       │   └── metrics.rs
│       └── runner/
│           └── control.rs      # NEW - runner lifecycle control
├── web-backend/
│   └── src/
│       ├── routes/
│       │   ├── strategies.rs   # NEW
│       │   └── backtest.rs     # NEW
│       └── services/
│           ├── strategy_validator.rs  # NEW
│           └── backtest_service.rs    # NEW
└── web-frontend/
    └── src/
        ├── pages/
        │   ├── RunnerManager.tsx      # NEW (enhanced dashboard)
        │   ├── StrategyEditor.tsx     # NEW
        │   ├── Tuner.tsx              # NEW
        │   └── RunnerDetail.tsx       # Enhanced
        ├── components/
        │   ├── runners/
        │   │   ├── RunnerTable.tsx    # NEW
        │   │   ├── RunnerControls.tsx # NEW
        │   │   └── CreateRunnerForm.tsx # Enhanced
        │   ├── strategy/
        │   │   ├── CodeEditor.tsx     # NEW
        │   │   ├── TemplatePicker.tsx # NEW
        │   │   └── ValidationPanel.tsx # NEW
        │   ├── backtest/
        │   │   ├── ConfigForm.tsx     # NEW
        │   │   ├── ResultsView.tsx    # NEW
        │   │   └── EquityCurve.tsx    # NEW
        │   └── charts/
        │       ├── CandlestickChart.tsx # Enhanced
        │       ├── IndicatorPanel.tsx   # NEW
        │       └── PositionMarkers.tsx  # NEW
        └── services/
            ├── strategyApi.ts   # NEW
            └── backtestApi.ts   # NEW
```

## Success Metrics

### User Experience
- Create a new strategy in < 2 minutes
- Run a backtest in < 30 seconds (for 1 week of 1m data)
- Start/stop runners with < 500ms latency
- Real-time indicator updates at < 100ms

### Performance
- Handle 100+ concurrent backtests
- Support 50+ active runners simultaneously
- Strategy validation < 100ms
- Chart rendering < 1 second for 1000 candles

### Code Quality
- 80%+ test coverage for backtest engine
- All API endpoints documented
- TypeScript strict mode enabled
- Zero Clippy warnings

## Risks and Mitigation

### Risk 1: Historical Data Volume
**Issue**: Fetching large amounts of historical data is slow
**Mitigation**:
- Implement caching layer (Redis or local file cache)
- Limit backtest duration (max 90 days)
- Use pagination for results

### Risk 2: Lua Validation Complexity
**Issue**: Parsing Lua without executing is difficult
**Mitigation**:
- Use mlua library for syntax checking
- Run strategy in sandbox for full validation
- Provide clear error messages

### Risk 3: Concurrent Backtests Resource Usage
**Issue**: Many backtests running simultaneously could overload server
**Mitigation**:
- Queue system with max concurrent limit (e.g., 5)
- Priority queue for user-submitted vs automated tests
- Resource limits per backtest

### Risk 4: Chart Performance with Large Datasets
**Issue**: Rendering 1000+ candles with indicators is slow
**Mitigation**:
- Use TradingView Lightweight Charts (WebGL accelerated)
- Implement data windowing (only render visible range)
- Downsample data for larger timeframes

## Future Enhancements (Phase 8+)

1. **AI Strategy Tuner**
   - Use ML to optimize strategy parameters
   - Genetic algorithms for parameter search
   - Integration with strategy editor

2. **WebSocket Real-time Updates**
   - Push runner state changes to UI
   - Live indicator updates without polling
   - Real-time backtest progress

3. **Multi-User Support**
   - User authentication
   - Per-user strategies and runners
   - Shared strategy marketplace

4. **Advanced Analytics**
   - Monte Carlo simulations
   - Walk-forward optimization
   - Strategy correlation analysis
   - Portfolio-level backtesting

5. **Mobile App**
   - React Native companion app
   - Push notifications for trades
   - Quick runner control

## Conclusion

This expansion transforms the trading system web app from a monitoring tool into a comprehensive trading strategy development environment. Users can create, test, and deploy strategies entirely through the web interface, with powerful backtesting and visualization tools to validate their ideas before risking capital.

**Total Estimated Effort**:
- Backend: ~2,800 LOC
- Frontend: ~3,300 LOC
- Tests: ~100 tests
- Time: 2-3 weeks for Phase 6 Step 10 + Phase 7

**Next Steps**:
1. Review and approve this plan
2. Start with Priority 1 (Runner Manager) for immediate value
3. Iterate based on user feedback
4. Build toward full backtesting system
