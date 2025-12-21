import { useQuery } from '@tanstack/react-query';
import { apiClient } from '../services/api';

export function useEngineSummary() {
  return useQuery({
    queryKey: ['engine', 'summary'],
    queryFn: () => apiClient.getEngineSummary(),
    refetchInterval: 5000, // Refresh every 5 seconds
  });
}

export function useRunnerSnapshot(runnerId: string) {
  return useQuery({
    queryKey: ['runner', runnerId, 'snapshot'],
    queryFn: () => apiClient.getRunnerSnapshot(runnerId),
    refetchInterval: 2000, // Refresh every 2 seconds
    enabled: !!runnerId,
  });
}

export function usePriceHistory(runnerId: string, count?: number) {
  return useQuery({
    queryKey: ['runner', runnerId, 'history', count],
    queryFn: () => apiClient.getPriceHistory(runnerId, count),
    refetchInterval: 5000,
    enabled: !!runnerId,
  });
}

export function useAllRunnerSnapshots(runnerIds: string[]) {
  return useQuery({
    queryKey: ['runners', 'snapshots', runnerIds],
    queryFn: async () => {
      const snapshots = await Promise.all(
        runnerIds.map((id) => apiClient.getRunnerSnapshot(id))
      );
      return snapshots;
    },
    refetchInterval: 3000, // Refresh every 3 seconds
    enabled: runnerIds.length > 0,
  });
}

export function useStrategies() {
  return useQuery({
    queryKey: ['strategies'],
    queryFn: () => apiClient.listStrategies(),
    staleTime: 60000, // Strategies don't change often, cache for 1 minute
  });
}

export function useSymbols() {
  return useQuery({
    queryKey: ['symbols'],
    queryFn: () => apiClient.listSymbols(),
    staleTime: 300000, // Symbols rarely change, cache for 5 minutes
  });
}
