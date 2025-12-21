// API Response Types

export interface HealthResponse {
  status: string;
  timestamp: number;
  version: string;
}

export interface EngineHealthResponse {
  status: string;
  runners_count: number;
  healthy_runners: number;
  timestamp: number;
}

export interface RunnerSummary {
  runner_id: string;
  symbol: string;
}

export interface EngineSummaryResponse {
  status: string;
  total_runners: number;
  healthy_runners: number;
  active_symbols: string[];
  runners: RunnerSummary[];
  timestamp: number;
}

export interface Position {
  entry_price: number;
  quantity: number;
  side: 'Long' | 'Short';
  entry_timestamp: number;
  stop_loss?: number;
  take_profit?: number;
  unrealized_pnl?: number;
}

export interface ContextSnapshot {
  strings: Record<string, string>;
  numbers: Record<string, number>;
  integers: Record<string, number>;
  booleans: Record<string, boolean>;
}

export interface RunnerStats {
  ticks_processed: number;
  actions_executed: number;
  errors: number;
  avg_tick_duration: Duration;
  min_tick_duration: Duration;
  max_tick_duration: Duration;
}

export interface Duration {
  secs: number;
  nanos: number;
}

export type RunnerStatus = 'running' | 'paused' | 'stopped';

export interface RunnerSnapshot {
  runner_id: string;
  symbol: string;
  status: RunnerStatus;
  current_state: 'Idle' | 'Analyzing' | 'InPosition';
  position: Position | null;
  context: ContextSnapshot;
  stats: RunnerStats;
  uptime_secs: number;
  snapshot_timestamp: number;
}

export interface MarketData {
  symbol: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  timestamp: number;
  bid: number;
  ask: number;
}

export interface ApiError {
  status: string;
  error: {
    code: string;
    message: string;
  };
  timestamp: number;
}

export interface AddRunnerRequest {
  runner_id: string;
  symbol: string;
  strategy_path: string;
  window_size?: number;
}

export interface AddRunnerResponse {
  runner_id: string;
  symbol: string;
  message: string;
}

export interface ControlResponse {
  success: boolean;
  message: string;
}

export interface StrategyInfo {
  name: string;
  path: string;
  category: string;
}

export interface StrategyListResponse {
  strategies: StrategyInfo[];
}

export interface SymbolInfo {
  symbol: string;
  name: string;
  category: string;
}

export interface SymbolListResponse {
  symbols: SymbolInfo[];
}
