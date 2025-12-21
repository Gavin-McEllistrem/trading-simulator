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
