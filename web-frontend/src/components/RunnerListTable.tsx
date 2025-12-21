import { useState } from 'react';
import { Link } from 'react-router-dom';
import type { RunnerSnapshot, RunnerStatus } from '../types/api';
import { apiClient } from '../services/api';

interface RunnerListTableProps {
  runners: RunnerSnapshot[];
  onRunnerUpdated?: () => void;
}

export function RunnerListTable({ runners, onRunnerUpdated }: RunnerListTableProps) {
  const [loadingActions, setLoadingActions] = useState<Record<string, boolean>>({});
  const [error, setError] = useState<string | null>(null);

  const handleControl = async (
    runnerId: string,
    action: 'pause' | 'resume' | 'stop'
  ) => {
    setLoadingActions((prev) => ({ ...prev, [runnerId]: true }));
    setError(null);

    try {
      let response;
      switch (action) {
        case 'pause':
          response = await apiClient.pauseRunner(runnerId);
          break;
        case 'resume':
          response = await apiClient.resumeRunner(runnerId);
          break;
        case 'stop':
          response = await apiClient.stopRunner(runnerId);
          break;
      }

      if (response.success && onRunnerUpdated) {
        onRunnerUpdated();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Control action failed');
      console.error(`Failed to ${action} runner ${runnerId}:`, err);
    } finally {
      setLoadingActions((prev) => ({ ...prev, [runnerId]: false }));
    }
  };

  const getStatusColor = (status: RunnerStatus): string => {
    switch (status) {
      case 'running':
        return 'bg-green-100 text-green-800';
      case 'paused':
        return 'bg-yellow-100 text-yellow-800';
      case 'stopped':
        return 'bg-red-100 text-red-800';
    }
  };

  const getStateColor = (state: string): string => {
    switch (state) {
      case 'Idle':
        return 'text-gray-600';
      case 'Analyzing':
        return 'text-blue-600';
      case 'InPosition':
        return 'text-purple-600';
      default:
        return 'text-gray-600';
    }
  };

  const formatUptime = (secs: number): string => {
    const hours = Math.floor(secs / 3600);
    const minutes = Math.floor((secs % 3600) / 60);
    const seconds = secs % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else if (minutes > 0) {
      return `${minutes}m ${seconds}s`;
    } else {
      return `${seconds}s`;
    }
  };

  if (runners.length === 0) {
    return (
      <div className="bg-gray-50 rounded-lg p-8 text-center text-gray-500">
        No runners currently active
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      {error && (
        <div className="mb-4 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
          {error}
        </div>
      )}

      <table className="min-w-full bg-white rounded-lg shadow overflow-hidden">
        <thead className="bg-gray-50">
          <tr>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Runner ID
            </th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Symbol
            </th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Status
            </th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              State
            </th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Position
            </th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Uptime
            </th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Stats
            </th>
            <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-200">
          {runners.map((runner) => (
            <tr key={runner.runner_id} className="hover:bg-gray-50">
              <td className="px-6 py-4 whitespace-nowrap">
                <Link
                  to={`/runner/${runner.runner_id}`}
                  className="text-blue-600 hover:text-blue-800 font-medium"
                >
                  {runner.runner_id}
                </Link>
              </td>
              <td className="px-6 py-4 whitespace-nowrap">
                <span className="font-medium">{runner.symbol}</span>
              </td>
              <td className="px-6 py-4 whitespace-nowrap">
                <span
                  className={`px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full ${getStatusColor(
                    runner.status
                  )}`}
                >
                  {runner.status}
                </span>
              </td>
              <td className="px-6 py-4 whitespace-nowrap">
                <span className={`text-sm font-medium ${getStateColor(runner.current_state)}`}>
                  {runner.current_state}
                </span>
              </td>
              <td className="px-6 py-4 whitespace-nowrap">
                {runner.position ? (
                  <div className="text-sm">
                    <div className="font-medium text-gray-900">
                      {runner.position.side}
                    </div>
                    <div className="text-gray-500">
                      ${runner.position.entry_price.toFixed(2)}
                    </div>
                  </div>
                ) : (
                  <span className="text-gray-400 text-sm">-</span>
                )}
              </td>
              <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {formatUptime(runner.uptime_secs)}
              </td>
              <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                <div>Ticks: {runner.stats.ticks_processed}</div>
                <div className="text-xs text-gray-400">
                  Errors: {runner.stats.errors}
                </div>
              </td>
              <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                <div className="flex justify-end gap-2">
                  {runner.status === 'running' && (
                    <button
                      onClick={() => handleControl(runner.runner_id, 'pause')}
                      disabled={loadingActions[runner.runner_id]}
                      className="text-yellow-600 hover:text-yellow-900 disabled:opacity-50"
                      title="Pause runner"
                    >
                      Pause
                    </button>
                  )}
                  {runner.status === 'paused' && (
                    <button
                      onClick={() => handleControl(runner.runner_id, 'resume')}
                      disabled={loadingActions[runner.runner_id]}
                      className="text-green-600 hover:text-green-900 disabled:opacity-50"
                      title="Resume runner"
                    >
                      Resume
                    </button>
                  )}
                  {runner.status !== 'stopped' && (
                    <button
                      onClick={() => handleControl(runner.runner_id, 'stop')}
                      disabled={loadingActions[runner.runner_id]}
                      className="text-red-600 hover:text-red-900 disabled:opacity-50"
                      title="Stop runner"
                    >
                      Stop
                    </button>
                  )}
                  <Link
                    to={`/runner/${runner.runner_id}`}
                    className="text-blue-600 hover:text-blue-900"
                  >
                    Details
                  </Link>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
