import { MetricsQueryParams, MetricsResponse } from '@/types/metrics';
import { fetchWithAuth } from '@/lib/fetch-with-auth';

/**
 * Admin endpoints for metrics and health data
 */

/**
 * Fetch system health status from admin endpoint
 */
export async function fetchSystemHealth(): Promise<{
  overall: 'healthy' | 'degraded' | 'down';
  server: any;
  services: any[];
  integrations: any[];
  lastChecked: Date;
}> {
  try {
    const response = await fetchWithAuth('/admin/health');

    if (!response.ok) {
      throw new Error(`Failed to fetch system health: ${response.statusText}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error fetching system health:', error);
    throw error;
  }
}

/**
 * Fetch organization metrics from admin endpoint
 */
export async function fetchOrganizationMetrics(
  organizationId: string,
  timeRange: '1h' | '24h' | '7d' | '30d' = '24h',
): Promise<{
  members: any;
  financial: any;
  activity: any;
  compliance: any;
}> {
  try {
    const response = await fetchWithAuth(
      `/admin/metrics/organization/${organizationId}?timeRange=${timeRange}`,
    );

    if (!response.ok) {
      throw new Error(
        `Failed to fetch organization metrics: ${response.statusText}`,
      );
    }

    return await response.json();
  } catch (error) {
    console.error('Error fetching organization metrics:', error);
    throw error;
  }
}

/**
 * Fetch chama metrics from admin endpoint
 */
export async function fetchChamaMetrics(
  chamaId: string,
  timeRange: '1h' | '24h' | '7d' | '30d' = '24h',
): Promise<{
  members: any;
  contributions: any;
  loans: any;
  activity: any;
}> {
  try {
    const response = await fetchWithAuth(
      `/admin/metrics/chama/${chamaId}?timeRange=${timeRange}`,
    );

    if (!response.ok) {
      throw new Error(`Failed to fetch chama metrics: ${response.statusText}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error fetching chama metrics:', error);
    throw error;
  }
}

/**
 * Legacy metrics function - now uses admin endpoint for system health
 * and combines organization/chama data for dashboard overview
 */
export async function fetchMetrics(
  params: MetricsQueryParams = {},
): Promise<MetricsResponse> {
  try {
    // For admin dashboard, we combine health data with aggregated metrics
    const healthData = await fetchSystemHealth();

    // Transform health data into the expected MetricsResponse format
    // This is a simplified transformation - you may want to enhance this
    const response: MetricsResponse = {
      lastUpdated: healthData.lastChecked,
      business: {
        userEngagement: {
          dailyActiveUsers: { value: 0, change: 0, trend: 'flat' as const },
          monthlyActiveUsers: { value: 0, change: 0, trend: 'flat' as const },
          newUserRegistrations: { value: 0, change: 0, trend: 'flat' as const },
          dau_mau_ratio: { value: 0, change: 0, trend: 'flat' as const },
          sessionCount: { name: 'Sessions', data: [] },
          retentionRates: {
            day1: 0,
            day7: 0,
            day30: 0,
            day90: 0,
          },
        },
        transactions: {
          successRate: {
            value: healthData.overall === 'healthy' ? 95 : 70,
            change: 0,
            trend: 'flat' as const,
          },
          volume: {
            KES: { value: 0, change: 0, trend: 'flat' as const },
            BTC: { value: 0, change: 0, trend: 'flat' as const },
          },
          total: { value: 0, change: 0, trend: 'flat' as const },
          averageDuration: { value: 0, change: 0, trend: 'flat' as const },
          timeline: { name: 'Transactions', data: [] },
          byType: {},
        },
        features: {
          usage: {
            wallet: {
              count: { value: 0, change: 0, trend: 'flat' as const },
              successRate: { value: 0, change: 0, trend: 'flat' as const },
              averageDuration: { value: 0, change: 0, trend: 'flat' as const },
            },
            swap: {
              count: { value: 0, change: 0, trend: 'flat' as const },
              successRate: { value: 0, change: 0, trend: 'flat' as const },
              averageDuration: { value: 0, change: 0, trend: 'flat' as const },
            },
            chama: {
              count: { value: 0, change: 0, trend: 'flat' as const },
              successRate: { value: 0, change: 0, trend: 'flat' as const },
              averageDuration: { value: 0, change: 0, trend: 'flat' as const },
            },
          },
          timeline: { name: 'Feature Usage', data: [] },
        },
      },
      services: {
        auth: {
          loginAttempts: { value: 0, change: 0, trend: 'flat' as const },
          successfulLogins: { value: 0, change: 0, trend: 'flat' as const },
          failedLogins: { value: 0, change: 0, trend: 'flat' as const },
          tokenOperations: { value: 0, change: 0, trend: 'flat' as const },
          timeline: { name: 'Auth', data: [] },
        },
      },
      systemHealth: healthData,
    };

    return response;
  } catch (error) {
    console.error('Error fetching metrics:', error);
    throw error;
  }
}

/**
 * Fetches metrics for a specific service
 */
export async function fetchServiceMetrics(
  service: string,
  params: Omit<MetricsQueryParams, 'services'> = {},
): Promise<MetricsResponse> {
  return fetchMetrics({
    ...params,
    services: [service],
  });
}
