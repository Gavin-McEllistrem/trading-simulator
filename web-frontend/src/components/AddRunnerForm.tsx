import { useState } from 'react';
import type { FormEvent } from 'react';
import { useStrategies, useSymbols } from '../hooks/useApi';
import type { AddRunnerRequest } from '../types/api';

interface AddRunnerFormProps {
  onSuccess?: () => void;
  onCancel?: () => void;
}

export function AddRunnerForm({ onSuccess, onCancel }: AddRunnerFormProps) {
  const [formData, setFormData] = useState<AddRunnerRequest>({
    runner_id: '',
    symbol: '',
    strategy_path: '',
    window_size: 200,
  });
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Fetch available strategies and symbols
  const { data: strategiesData, isLoading: strategiesLoading } = useStrategies();
  const strategies = strategiesData?.strategies || [];

  const { data: symbolsData, isLoading: symbolsLoading } = useSymbols();
  const symbols = symbolsData?.symbols || [];

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    setIsSubmitting(true);

    try {
      const { apiClient } = await import('../services/api');
      await apiClient.addRunner(formData);

      // Reset form
      setFormData({
        runner_id: '',
        symbol: '',
        strategy_path: '',
        window_size: 200,
      });

      onSuccess?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create runner');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="bg-white p-6 rounded-lg shadow-md">
      <h2 className="text-xl font-bold mb-4">Add New Runner</h2>

      {error && (
        <div className="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
          {error}
        </div>
      )}

      <div className="space-y-4">
        <div>
          <label htmlFor="runner_id" className="block text-sm font-medium text-gray-700 mb-1">
            Runner ID
          </label>
          <input
            type="text"
            id="runner_id"
            value={formData.runner_id}
            onChange={(e) => setFormData({ ...formData, runner_id: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            required
            disabled={isSubmitting}
          />
        </div>

        <div>
          <label htmlFor="symbol" className="block text-sm font-medium text-gray-700 mb-1">
            Symbol
          </label>
          {symbolsLoading ? (
            <div className="text-sm text-gray-500 py-2">Loading symbols...</div>
          ) : symbols.length === 0 ? (
            <div className="text-sm text-red-600 py-2">
              No symbols available
            </div>
          ) : (
            <select
              id="symbol"
              value={formData.symbol}
              onChange={(e) => setFormData({ ...formData, symbol: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
              disabled={isSubmitting}
            >
              <option value="">Select a symbol...</option>
              {/* Group symbols by category */}
              {Array.from(new Set(symbols.map(s => s.category))).map(category => (
                <optgroup key={category} label={category}>
                  {symbols
                    .filter(s => s.category === category)
                    .map(symbol => (
                      <option key={symbol.symbol} value={symbol.symbol}>
                        {symbol.symbol} - {symbol.name}
                      </option>
                    ))}
                </optgroup>
              ))}
            </select>
          )}
        </div>

        <div>
          <label htmlFor="strategy_path" className="block text-sm font-medium text-gray-700 mb-1">
            Strategy
          </label>
          {strategiesLoading ? (
            <div className="text-sm text-gray-500 py-2">Loading strategies...</div>
          ) : strategies.length === 0 ? (
            <div className="text-sm text-red-600 py-2">
              No strategies found in lua-strategies directory
            </div>
          ) : (
            <select
              id="strategy_path"
              value={formData.strategy_path}
              onChange={(e) => setFormData({ ...formData, strategy_path: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
              disabled={isSubmitting}
            >
              <option value="">Select a strategy...</option>
              {strategies.map((strategy) => (
                <option key={strategy.path} value={strategy.path}>
                  {strategy.name} ({strategy.category})
                </option>
              ))}
            </select>
          )}
        </div>

        <div>
          <label htmlFor="window_size" className="block text-sm font-medium text-gray-700 mb-1">
            Window Size (optional)
          </label>
          <input
            type="number"
            id="window_size"
            value={formData.window_size}
            onChange={(e) => setFormData({ ...formData, window_size: parseInt(e.target.value) || 200 })}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            disabled={isSubmitting}
            min="1"
          />
        </div>

        <div className="flex gap-3 pt-4">
          <button
            type="submit"
            disabled={isSubmitting}
            className="flex-1 bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 disabled:bg-blue-300 disabled:cursor-not-allowed transition-colors"
          >
            {isSubmitting ? 'Creating...' : 'Create Runner'}
          </button>

          {onCancel && (
            <button
              type="button"
              onClick={onCancel}
              disabled={isSubmitting}
              className="flex-1 bg-gray-200 text-gray-800 py-2 px-4 rounded-md hover:bg-gray-300 disabled:bg-gray-100 disabled:cursor-not-allowed transition-colors"
            >
              Cancel
            </button>
          )}
        </div>
      </div>
    </form>
  );
}
