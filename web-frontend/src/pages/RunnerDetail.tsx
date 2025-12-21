import { useParams, Link } from 'react-router-dom';
import { useRunnerSnapshot, usePriceHistory } from '../hooks/useApi';
import {
  ComposedChart,
  Line,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Cell,
} from 'recharts';

export function RunnerDetail() {
  const { runnerId } = useParams<{ runnerId: string }>();
  const { data: snapshot, isLoading: snapshotLoading } = useRunnerSnapshot(runnerId!);
  const { data: history, isLoading: historyLoading } = usePriceHistory(runnerId!, 50);

  if (snapshotLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-xl">Loading...</div>
      </div>
    );
  }

  if (!snapshot) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-xl text-red-600">Runner not found</div>
      </div>
    );
  }

  const stateColor = {
    Idle: 'text-gray-600',
    Analyzing: 'text-yellow-600',
    InPosition: 'text-green-600',
  }[snapshot.current_state];

  const formatTimestamp = (ts: number) => {
    return new Date(ts * 1000).toLocaleString();
  };

  const formatDuration = (ms: number) => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    if (hours > 0) return `${hours}h ${minutes % 60}m`;
    if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
    return `${seconds}s`;
  };

  const chartData = history?.map((data, index) => {
    const isGreen = data.close >= data.open;
    return {
      time: new Date(data.timestamp).toLocaleTimeString(),
      index,
      open: data.open,
      high: data.high,
      low: data.low,
      close: data.close,
      isGreen,
      volume: data.volume,
    };
  }) || [];

  return (
    <div className="container mx-auto p-6">
      <div className="mb-6">
        <Link to="/" className="text-blue-600 hover:text-blue-800">
          ← Back to Dashboard
        </Link>
      </div>

      <h1 className="text-3xl font-bold mb-6">{snapshot.runner_id}</h1>

      {/* Status Grid */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
        <div className="bg-white rounded-lg shadow p-6">
          <div className="text-gray-600 text-sm font-medium">Symbol</div>
          <div className="text-2xl font-bold mt-2">{snapshot.symbol}</div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="text-gray-600 text-sm font-medium">State</div>
          <div className={`text-2xl font-bold mt-2 ${stateColor}`}>
            {snapshot.current_state}
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="text-gray-600 text-sm font-medium">Ticks Processed</div>
          <div className="text-2xl font-bold mt-2">{snapshot.stats.ticks_processed}</div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="text-gray-600 text-sm font-medium">Uptime</div>
          <div className="text-2xl font-bold mt-2">
            {formatDuration(snapshot.uptime_secs * 1000)}
          </div>
        </div>
      </div>

      {/* Position Info */}
      {snapshot.position && (
        <div className="bg-white rounded-lg shadow p-6 mb-8">
          <h2 className="text-2xl font-semibold mb-4">Current Position</h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div>
              <div className="text-gray-600 text-sm">Side</div>
              <div className={`text-lg font-bold ${
                snapshot.position.side === 'Long' ? 'text-green-600' : 'text-red-600'
              }`}>
                {snapshot.position.side}
              </div>
            </div>
            <div>
              <div className="text-gray-600 text-sm">Entry Price</div>
              <div className="text-lg font-bold">${snapshot.position.entry_price.toFixed(2)}</div>
            </div>
            <div>
              <div className="text-gray-600 text-sm">Quantity</div>
              <div className="text-lg font-bold">{snapshot.position.quantity}</div>
            </div>
            {snapshot.position.unrealized_pnl !== undefined && (
              <div>
                <div className="text-gray-600 text-sm">P&L</div>
                <div className={`text-lg font-bold ${
                  snapshot.position.unrealized_pnl >= 0 ? 'text-green-600' : 'text-red-600'
                }`}>
                  ${snapshot.position.unrealized_pnl.toFixed(2)}
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Price Chart */}
      <div className="bg-white rounded-lg shadow p-6 mb-8">
        <h2 className="text-2xl font-semibold mb-4">Price Chart (Candlesticks)</h2>
        {historyLoading ? (
          <div className="text-center py-8">Loading chart...</div>
        ) : chartData.length > 0 ? (
          <ResponsiveContainer width="100%" height={500}>
            <ComposedChart data={chartData} margin={{ top: 20, right: 30, left: 20, bottom: 20 }}>
              <defs>
                <linearGradient id="colorGreen" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#10b981" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#10b981" stopOpacity={0.8}/>
                </linearGradient>
                <linearGradient id="colorRed" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#ef4444" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#ef4444" stopOpacity={0.8}/>
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
              <XAxis
                dataKey="time"
                tick={{ fontSize: 12 }}
                interval="preserveStartEnd"
              />
              <YAxis
                domain={['auto', 'auto']}
                tick={{ fontSize: 12 }}
                tickFormatter={(value) => `$${value.toFixed(2)}`}
              />
              <Tooltip
                contentStyle={{ backgroundColor: '#fff', border: '1px solid #e5e7eb' }}
                content={({ active, payload }) => {
                  if (active && payload && payload.length) {
                    const data = payload[0].payload;
                    return (
                      <div className="bg-white p-3 border border-gray-300 rounded shadow-lg">
                        <p className="font-semibold">{data.time}</p>
                        <p>Open: <span className="font-mono">${data.open.toFixed(2)}</span></p>
                        <p>High: <span className="font-mono">${data.high.toFixed(2)}</span></p>
                        <p>Low: <span className="font-mono">${data.low.toFixed(2)}</span></p>
                        <p>Close: <span className="font-mono">${data.close.toFixed(2)}</span></p>
                        <p>Volume: <span className="font-mono">{data.volume}</span></p>
                      </div>
                    );
                  }
                  return null;
                }}
              />
              {/* Render candlesticks as lines */}
              <Line
                type="monotone"
                dataKey="high"
                stroke="#9ca3af"
                strokeWidth={1}
                dot={false}
                activeDot={false}
                isAnimationActive={false}
              />
              <Line
                type="monotone"
                dataKey="low"
                stroke="#9ca3af"
                strokeWidth={1}
                dot={false}
                activeDot={false}
                isAnimationActive={false}
              />
              <Line
                type="monotone"
                dataKey="close"
                stroke="#3b82f6"
                strokeWidth={2}
                dot={{ r: 3 }}
                isAnimationActive={false}
              />
            </ComposedChart>
          </ResponsiveContainer>
        ) : (
          <div className="text-center py-8 text-gray-500">No price data available</div>
        )}
        {chartData.length > 0 && (
          <div className="mt-4 flex items-center justify-center gap-6 text-sm text-gray-600">
            <p>Showing {chartData.length} candles</p>
            <p>Latest: ${chartData[chartData.length - 1]?.close.toFixed(2)}</p>
          </div>
        )}
      </div>

      {/* Statistics */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-2xl font-semibold mb-4">Statistics</h2>
        <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
          <div>
            <div className="text-gray-600 text-sm">Actions Executed</div>
            <div className="text-lg font-bold">{snapshot.stats.actions_executed}</div>
          </div>
          <div>
            <div className="text-gray-600 text-sm">Errors</div>
            <div className="text-lg font-bold">{snapshot.stats.errors}</div>
          </div>
          <div>
            <div className="text-gray-600 text-sm">Snapshot Time</div>
            <div className="text-sm">{formatTimestamp(snapshot.snapshot_timestamp)}</div>
          </div>
        </div>
      </div>
    </div>
  );
}
