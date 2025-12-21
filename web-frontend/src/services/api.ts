import type {
  HealthResponse,
  EngineHealthResponse,
  EngineSummaryResponse,
  RunnerSnapshot,
  MarketData,
  AddRunnerRequest,
  AddRunnerResponse,
  ControlResponse,
  StrategyListResponse,
  SymbolListResponse,
} from '../types/api';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private async fetch<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({
        error: { code: 'UNKNOWN', message: response.statusText },
      }));
      throw new Error(error.error?.message || 'API request failed');
    }

    return response.json();
  }

  // Health endpoints
  async getHealth(): Promise<HealthResponse> {
    return this.fetch<HealthResponse>('/health');
  }

  async getEngineHealth(): Promise<EngineHealthResponse> {
    return this.fetch<EngineHealthResponse>('/api/engine/health');
  }

  async getEngineSummary(): Promise<EngineSummaryResponse> {
    return this.fetch<EngineSummaryResponse>('/api/engine/summary');
  }

  // Runner endpoints
  async getRunnerSnapshot(runnerId: string): Promise<RunnerSnapshot> {
    return this.fetch<RunnerSnapshot>(`/api/runners/${runnerId}/snapshot`);
  }

  async getPriceHistory(
    runnerId: string,
    count?: number
  ): Promise<MarketData[]> {
    const params = count ? `?count=${count}` : '';
    return this.fetch<MarketData[]>(
      `/api/runners/${runnerId}/history${params}`
    );
  }

  async removeRunner(runnerId: string): Promise<void> {
    await this.fetch<void>(`/api/runners/${runnerId}`, {
      method: 'DELETE',
    });
  }

  async addRunner(request: AddRunnerRequest): Promise<AddRunnerResponse> {
    return this.fetch<AddRunnerResponse>('/api/runners', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  // Runner control endpoints
  async pauseRunner(runnerId: string): Promise<ControlResponse> {
    return this.fetch<ControlResponse>(`/api/runners/${runnerId}/pause`, {
      method: 'POST',
    });
  }

  async resumeRunner(runnerId: string): Promise<ControlResponse> {
    return this.fetch<ControlResponse>(`/api/runners/${runnerId}/resume`, {
      method: 'POST',
    });
  }

  async stopRunner(runnerId: string): Promise<ControlResponse> {
    return this.fetch<ControlResponse>(`/api/runners/${runnerId}/stop`, {
      method: 'POST',
    });
  }

  // Strategy endpoints
  async listStrategies(): Promise<StrategyListResponse> {
    return this.fetch<StrategyListResponse>('/api/strategies');
  }

  // Symbol endpoints
  async listSymbols(): Promise<SymbolListResponse> {
    return this.fetch<SymbolListResponse>('/api/symbols');
  }
}

export const apiClient = new ApiClient();
