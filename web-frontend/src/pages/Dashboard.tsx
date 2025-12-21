import { useState } from 'react';
import { useEngineSummary, useAllRunnerSnapshots } from '../hooks/useApi';
import { AddRunnerForm } from '../components/AddRunnerForm';
import { RunnerListTable } from '../components/RunnerListTable';

export function Dashboard() {
  const [showAddForm, setShowAddForm] = useState(false);
  const { data: summary, isLoading, error, refetch } = useEngineSummary();

  // Get runner IDs from summary
  const runnerIds = summary?.runners.map((r) => r.runner_id) || [];

  // Fetch snapshots for all runners
  const {
    data: snapshots,
    isLoading: snapshotsLoading,
    refetch: refetchSnapshots,
  } = useAllRunnerSnapshots(runnerIds);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-xl">Loading...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-xl text-red-600">
          Error: {error instanceof Error ? error.message : 'Failed to load data'}
        </div>
      </div>
    );
  }

  if (!summary) {
    return null;
  }

  return (
    <div className="container mx-auto p-6">
      <h1 className="text-3xl font-bold mb-6">Trading System Dashboard</h1>

      {/* Engine Status */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
        <div className="bg-white rounded-lg shadow p-6">
          <div className="text-gray-600 text-sm font-medium">Total Runners</div>
          <div className="text-3xl font-bold mt-2">{summary.total_runners}</div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="text-gray-600 text-sm font-medium">Healthy Runners</div>
          <div className="text-3xl font-bold mt-2 text-green-600">
            {summary.healthy_runners}
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="text-gray-600 text-sm font-medium">Active Symbols</div>
          <div className="text-3xl font-bold mt-2">{summary.active_symbols.length}</div>
        </div>
      </div>

      {/* Symbols */}
      <div className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">Active Symbols</h2>
        <div className="flex flex-wrap gap-2">
          {summary.active_symbols.map((symbol) => (
            <span
              key={symbol}
              className="bg-blue-100 text-blue-800 px-3 py-1 rounded-full text-sm font-medium"
            >
              {symbol}
            </span>
          ))}
        </div>
      </div>

      {/* Add Runner Form */}
      {showAddForm && (
        <div className="mb-8">
          <AddRunnerForm
            onSuccess={() => {
              setShowAddForm(false);
              refetch();
            }}
            onCancel={() => setShowAddForm(false)}
          />
        </div>
      )}

      {/* Runners List */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-semibold">Runners</h2>
          <button
            onClick={() => setShowAddForm(!showAddForm)}
            className="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 transition-colors"
          >
            {showAddForm ? 'Cancel' : 'Add Runner'}
          </button>
        </div>

        {snapshotsLoading && summary.runners.length > 0 ? (
          <div className="bg-white rounded-lg shadow p-8 text-center text-gray-500">
            Loading runner details...
          </div>
        ) : (
          <RunnerListTable
            runners={snapshots || []}
            onRunnerUpdated={() => {
              refetch();
              refetchSnapshots();
            }}
          />
        )}
      </div>
    </div>
  );
}
