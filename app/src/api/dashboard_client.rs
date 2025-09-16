use crate::api::ApiError;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;

/// Configuration for dashboard API client
#[derive(Debug, Clone)]
pub struct DashboardApiConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub cache_duration: Duration,
}

impl Default for DashboardApiConfig {
    fn default() -> Self {
        Self {
            base_url: std::env::var("NESTJS_API_URL")
                .unwrap_or_else(|_| "http://localhost:4000".to_string()),
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            cache_duration: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Standard API Response wrapper from NestJS backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestJsApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
    pub errors: Option<Vec<String>>,
    pub timestamp: String,
    pub meta: Option<ResponseMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub cached: bool,
    #[serde(rename = "cacheAge")]
    pub cache_age: u32,
    #[serde(rename = "dataSource")]
    pub data_source: String, // "realtime" | "aggregated" | "cached"
}

/// Dashboard Overview Response matching NestJS API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverviewResponse {
    pub summary: DashboardSummary,
    pub trends: DashboardTrends,
    pub alerts: SystemAlerts,
    #[serde(rename = "quickStats")]
    pub quick_stats: QuickStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    #[serde(rename = "totalMembers")]
    pub total_members: u64,
    #[serde(rename = "activeMembersToday")]
    pub active_members_today: u64,
    #[serde(rename = "activeChamas")]
    pub active_chamas: u64,
    #[serde(rename = "totalChamas")]
    pub total_chamas: u64,
    #[serde(rename = "totalVolume")]
    pub total_volume: MonetaryAmount,
    #[serde(rename = "transactionCount")]
    pub transaction_count: TransactionCount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonetaryAmount {
    pub amount: f64,
    pub currency: String, // "KES"
    pub period: String,   // "all-time" | "30d" | "7d"
}

impl Default for MonetaryAmount {
    fn default() -> Self {
        Self {
            amount: 0.0,
            currency: "KES".to_string(),
            period: "all-time".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCount {
    pub total: u64,
    pub successful: u64,
    pub failed: u64,
    pub pending: u64,
}

impl Default for TransactionCount {
    fn default() -> Self {
        Self {
            total: 0,
            successful: 0,
            failed: 0,
            pending: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTrends {
    #[serde(rename = "memberGrowth")]
    pub member_growth: Vec<TrendDataPoint>,
    #[serde(rename = "volumeTrend")]
    pub volume_trend: Vec<TrendDataPoint>,
    #[serde(rename = "transactionTrend")]
    pub transaction_trend: Vec<TrendDataPoint>,
    #[serde(rename = "chamaGrowth")]
    pub chama_growth: Vec<TrendDataPoint>,
}

impl Default for DashboardTrends {
    fn default() -> Self {
        Self {
            member_growth: Vec::new(),
            volume_trend: Vec::new(),
            transaction_trend: Vec::new(),
            chama_growth: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDataPoint {
    pub date: String,
    pub value: f64,
    pub change: Option<f64>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAlerts {
    #[serde(rename = "systemHealth")]
    pub system_health: String, // "healthy" | "warning" | "critical"
    #[serde(rename = "errorRate")]
    pub error_rate: f64,
    #[serde(rename = "avgResponseTime")]
    pub avg_response_time: u32,
    #[serde(rename = "criticalAlerts")]
    pub critical_alerts: Vec<AlertItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertItem {
    pub id: String,
    pub severity: String, // "warning" | "error" | "critical"
    pub message: String,
    pub timestamp: String,
    pub service: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickStats {
    #[serde(rename = "todayTransactions")]
    pub today_transactions: u64,
    #[serde(rename = "todayVolume")]
    pub today_volume: f64,
    #[serde(rename = "activeSessionsNow")]
    pub active_sessions_now: u64,
    #[serde(rename = "newMembersToday")]
    pub new_members_today: u64,
}

impl Default for QuickStats {
    fn default() -> Self {
        Self {
            today_transactions: 0,
            today_volume: 0.0,
            active_sessions_now: 0,
            new_members_today: 0,
        }
    }
}

/// User Analytics Response matching NestJS API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnalyticsResponse {
    pub engagement: UserEngagement,
    pub retention: RetentionMetrics,
    pub demographics: Demographics,
    #[serde(rename = "featureUsage")]
    pub feature_usage: FeatureUsage,
    #[serde(rename = "membershipActivity")]
    pub membership_activity: MembershipActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEngagement {
    #[serde(rename = "dailyActiveUsers")]
    pub daily_active_users: u64,
    #[serde(rename = "monthlyActiveUsers")]
    pub monthly_active_users: u64,
    #[serde(rename = "weeklyActiveUsers")]
    pub weekly_active_users: u64,
    pub dau_mau_ratio: f64,
    #[serde(rename = "sessionMetrics")]
    pub session_metrics: SessionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    #[serde(rename = "averageDuration")]
    pub average_duration: u32,
    #[serde(rename = "totalSessions")]
    pub total_sessions: u64,
    #[serde(rename = "sessionsToday")]
    pub sessions_today: u64,
    #[serde(rename = "peakConcurrentUsers")]
    pub peak_concurrent_users: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionMetrics {
    pub day1: f64,
    pub day7: f64,
    pub day30: f64,
    pub day90: f64,
    #[serde(rename = "cohortData")]
    pub cohort_data: Vec<CohortData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortData {
    #[serde(rename = "cohortMonth")]
    pub cohort_month: String,
    #[serde(rename = "newUsers")]
    pub new_users: u64,
    pub retention: CohortRetention,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortRetention {
    pub month1: f64,
    pub month2: f64,
    pub month3: f64,
    pub month6: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Demographics {
    #[serde(rename = "byRegion")]
    pub by_region: std::collections::HashMap<String, u64>,
    #[serde(rename = "byDeviceType")]
    pub by_device_type: std::collections::HashMap<String, u64>,
    #[serde(rename = "byAppVersion")]
    pub by_app_version: std::collections::HashMap<String, u64>,
    #[serde(rename = "registrationTrend")]
    pub registration_trend: Vec<TrendDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureUsage {
    #[serde(rename = "topFeatures")]
    pub top_features: Vec<FeatureUsageData>,
    pub adoption: Vec<FeatureAdoptionData>,
    #[serde(rename = "successRates")]
    pub success_rates: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureUsageData {
    #[serde(rename = "featureId")]
    pub feature_id: String,
    #[serde(rename = "featureName")]
    pub feature_name: String,
    #[serde(rename = "usageCount")]
    pub usage_count: u64,
    #[serde(rename = "uniqueUsers")]
    pub unique_users: u64,
    #[serde(rename = "averageDuration")]
    pub average_duration: f64,
    #[serde(rename = "successRate")]
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureAdoptionData {
    #[serde(rename = "featureId")]
    pub feature_id: String,
    #[serde(rename = "featureName")]
    pub feature_name: String,
    #[serde(rename = "adoptionRate")]
    pub adoption_rate: f64,
    #[serde(rename = "timeToAdoption")]
    pub time_to_adoption: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipActivity {
    #[serde(rename = "newRegistrations")]
    pub new_registrations: NewRegistrations,
    #[serde(rename = "chamaParticipation")]
    pub chama_participation: ChamaParticipation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRegistrations {
    pub today: u64,
    #[serde(rename = "thisWeek")]
    pub this_week: u64,
    #[serde(rename = "thisMonth")]
    pub this_month: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamaParticipation {
    #[serde(rename = "activeMembersInChamas")]
    pub active_members_in_chamas: u64,
    #[serde(rename = "averageChamasPerMember")]
    pub average_chamas_per_member: f64,
    #[serde(rename = "chamaMembershipTrend")]
    pub chama_membership_trend: Vec<TrendDataPoint>,
}

/// Financial Analytics Response matching NestJS API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialAnalyticsResponse {
    pub transactions: TransactionMetrics,
    pub swaps: SwapMetrics,
    pub chamas: ChamaMetrics,
    pub shares: SharesMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMetrics {
    pub volume: VolumeMetrics,
    pub counts: CountMetrics,
    pub performance: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMetrics {
    pub total: f64,
    pub today: f64,
    #[serde(rename = "thisWeek")]
    pub this_week: f64,
    #[serde(rename = "thisMonth")]
    pub this_month: f64,
    #[serde(rename = "byCurrency")]
    pub by_currency: std::collections::HashMap<String, CurrencyVolume>,
    #[serde(rename = "byOperation")]
    pub by_operation: std::collections::HashMap<String, OperationVolume>,
    pub trend: Vec<TrendDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyVolume {
    pub total: f64,
    pub today: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationVolume {
    pub total: f64,
    pub count: u64,
    #[serde(rename = "averageAmount")]
    pub average_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountMetrics {
    pub total: u64,
    pub successful: u64,
    pub failed: u64,
    pub pending: u64,
    #[serde(rename = "averagePerDay")]
    pub average_per_day: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    #[serde(rename = "averageDuration")]
    pub average_duration: f64,
    #[serde(rename = "successRate")]
    pub success_rate: f64,
    #[serde(rename = "errorsByType")]
    pub errors_by_type: std::collections::HashMap<String, u64>,
    #[serde(rename = "durationTrend")]
    pub duration_trend: Vec<TrendDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapMetrics {
    pub onramp: SwapAnalytics,
    pub offramp: SwapAnalytics,
    #[serde(rename = "fxRates")]
    pub fx_rates: FxRates,
    pub volume: SwapVolume,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapAnalytics {
    pub count: u64,
    pub successful: u64,
    #[serde(rename = "successRate")]
    pub success_rate: f64,
    #[serde(rename = "totalKes")]
    pub total_kes: f64,
    #[serde(rename = "totalSats")]
    pub total_sats: u64,
    #[serde(rename = "averageAmount")]
    pub average_amount: f64,
    pub trend: Vec<TrendDataPoint>,
    #[serde(rename = "byPaymentMethod")]
    pub by_payment_method: std::collections::HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxRates {
    pub current: CurrentFxRate,
    pub history: Vec<FxRateHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentFxRate {
    #[serde(rename = "buyRate")]
    pub buy_rate: f64,
    #[serde(rename = "sellRate")]
    pub sell_rate: f64,
    pub spread: f64,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxRateHistory {
    pub timestamp: String,
    #[serde(rename = "buyRate")]
    pub buy_rate: f64,
    #[serde(rename = "sellRate")]
    pub sell_rate: f64,
    pub spread: f64,
    pub volume: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapVolume {
    #[serde(rename = "totalOnrampKes")]
    pub total_onramp_kes: f64,
    #[serde(rename = "totalOfframpKes")]
    pub total_offramp_kes: f64,
    #[serde(rename = "totalOnrampSats")]
    pub total_onramp_sats: u64,
    #[serde(rename = "totalOfframpSats")]
    pub total_offramp_sats: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamaMetrics {
    pub financial: ChamaFinancial,
    pub distribution: ChamaDistribution,
    pub activity: ChamaActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamaFinancial {
    #[serde(rename = "totalBalance")]
    pub total_balance: f64,
    #[serde(rename = "totalDeposits")]
    pub total_deposits: f64,
    #[serde(rename = "totalWithdrawals")]
    pub total_withdrawals: f64,
    #[serde(rename = "netFlow")]
    pub net_flow: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamaDistribution {
    #[serde(rename = "balanceDistribution")]
    pub balance_distribution: Vec<DistributionData>,
    #[serde(rename = "memberBalanceDistribution")]
    pub member_balance_distribution: Vec<DistributionData>,
    #[serde(rename = "depositPatterns")]
    pub deposit_patterns: Vec<PatternData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionData {
    pub range: String,
    pub count: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternData {
    pub pattern: String,
    pub frequency: u64,
    #[serde(rename = "averageAmount")]
    pub average_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamaActivity {
    #[serde(rename = "depositsToday")]
    pub deposits_today: u64,
    #[serde(rename = "withdrawalsToday")]
    pub withdrawals_today: u64,
    #[serde(rename = "pendingWithdrawals")]
    pub pending_withdrawals: u64,
    #[serde(rename = "averageBalance")]
    pub average_balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharesMetrics {
    pub ownership: SharesOwnership,
    pub trading: SharesTrading,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharesOwnership {
    #[serde(rename = "totalShares")]
    pub total_shares: u64,
    #[serde(rename = "distributedShares")]
    pub distributed_shares: u64,
    #[serde(rename = "availableShares")]
    pub available_shares: u64,
    #[serde(rename = "ownershipConcentration")]
    pub ownership_concentration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharesTrading {
    #[serde(rename = "totalTransfers")]
    pub total_transfers: u64,
    #[serde(rename = "transferVolume")]
    pub transfer_volume: u64,
    #[serde(rename = "averageTransferSize")]
    pub average_transfer_size: f64,
    #[serde(rename = "transferTrend")]
    pub transfer_trend: Vec<TrendDataPoint>,
}

/// Operational Metrics Response matching NestJS API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalMetricsResponse {
    pub system: SystemMetrics,
    pub services: std::collections::HashMap<String, ServiceMetrics>,
    pub resources: ResourceMetrics,
    pub infrastructure: InfrastructureMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub health: SystemHealth,
    pub performance: SystemPerformance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: String, // "healthy" | "warning" | "critical"
    pub uptime: f64,
    #[serde(rename = "lastRestart")]
    pub last_restart: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformance {
    #[serde(rename = "responseTime")]
    pub response_time: ResponseTimeMetrics,
    pub throughput: ThroughputMetrics,
    pub errors: ErrorMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeMetrics {
    pub average: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub trend: Vec<TrendDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    #[serde(rename = "requestsPerSecond")]
    pub requests_per_second: f64,
    #[serde(rename = "requestsPerMinute")]
    pub requests_per_minute: f64,
    #[serde(rename = "peakRps")]
    pub peak_rps: f64,
    pub trend: Vec<TrendDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    #[serde(rename = "errorRate")]
    pub error_rate: f64,
    #[serde(rename = "totalErrors")]
    pub total_errors: u64,
    #[serde(rename = "errorsByType")]
    pub errors_by_type: std::collections::HashMap<String, u64>,
    #[serde(rename = "errorsByEndpoint")]
    pub errors_by_endpoint: std::collections::HashMap<String, u64>,
    pub trend: Vec<TrendDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub status: String, // "online" | "offline" | "degraded"
    #[serde(rename = "responseTime")]
    pub response_time: f64,
    #[serde(rename = "errorRate")]
    pub error_rate: f64,
    #[serde(rename = "lastHealthCheck")]
    pub last_health_check: String,
    pub dependencies: Vec<ServiceDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    pub name: String,
    pub status: String, // "online" | "offline" | "degraded"
    pub critical: bool,
    #[serde(rename = "lastChecked")]
    pub last_checked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub server: ServerMetrics,
    pub database: DatabaseMetrics,
    pub cache: CacheMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    #[serde(rename = "cpuUsage")]
    pub cpu_usage: f64,
    #[serde(rename = "memoryUsage")]
    pub memory_usage: f64,
    #[serde(rename = "diskUsage")]
    pub disk_usage: f64,
    #[serde(rename = "networkActivity")]
    pub network_activity: NetworkActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkActivity {
    #[serde(rename = "bytesIn")]
    pub bytes_in: u64,
    #[serde(rename = "bytesOut")]
    pub bytes_out: u64,
    #[serde(rename = "connectionsActive")]
    pub connections_active: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    #[serde(rename = "connectionPool")]
    pub connection_pool: ConnectionPool,
    pub performance: DatabasePerformance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPool {
    pub active: u32,
    pub idle: u32,
    pub waiting: u32,
    #[serde(rename = "maxConnections")]
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePerformance {
    #[serde(rename = "queryTime")]
    pub query_time: f64,
    #[serde(rename = "slowQueries")]
    pub slow_queries: u64,
    pub deadlocks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    #[serde(rename = "hitRate")]
    pub hit_rate: f64,
    #[serde(rename = "memoryUsage")]
    pub memory_usage: u64,
    pub evictions: u64,
    #[serde(rename = "keyCount")]
    pub key_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureMetrics {
    #[serde(rename = "loadBalancer")]
    pub load_balancer: LoadBalancerMetrics,
    pub monitoring: MonitoringMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerMetrics {
    #[serde(rename = "activeServers")]
    pub active_servers: u32,
    #[serde(rename = "totalServers")]
    pub total_servers: u32,
    #[serde(rename = "requestDistribution")]
    pub request_distribution: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringMetrics {
    #[serde(rename = "alertsActive")]
    pub alerts_active: u64,
    #[serde(rename = "alertsResolved")]
    pub alerts_resolved: u64,
    #[serde(rename = "monitoringCoverage")]
    pub monitoring_coverage: f64,
}

/// Live metrics update structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveMetricsUpdate {
    pub timestamp: String,
    pub metrics: LiveMetrics,
    pub alerts: Option<Vec<AlertItem>>,
    #[serde(rename = "type")]
    pub update_type: String, // "metrics-update" | "alert" | "system-event"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveMetrics {
    #[serde(rename = "activeUsers")]
    pub active_users: u64,
    #[serde(rename = "transactionsInProgress")]
    pub transactions_in_progress: u64,
    #[serde(rename = "systemLoad")]
    pub system_load: f64,
    #[serde(rename = "errorRate")]
    pub error_rate: f64,
    #[serde(rename = "responseTime")]
    pub response_time: f64,
}

/// Export request and response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: String, // "csv" | "xlsx" | "pdf" | "json"
    #[serde(rename = "dataType")]
    pub data_type: String, // "overview" | "users" | "financial" | "operations" | "all"
    #[serde(rename = "dateRange")]
    pub date_range: Option<DateRange>,
    pub filters: Option<std::collections::HashMap<String, serde_json::Value>>,
    #[serde(rename = "includeCharts")]
    pub include_charts: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    #[serde(rename = "exportId")]
    pub export_id: String,
    pub status: String, // "processing" | "completed" | "failed"
    #[serde(rename = "estimatedCompletion")]
    pub estimated_completion: Option<String>,
    #[serde(rename = "downloadUrl")]
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStatus {
    #[serde(rename = "exportId")]
    pub export_id: String,
    pub status: String, // "processing" | "completed" | "failed"
    pub progress: f64,
    #[serde(rename = "downloadUrl")]
    pub download_url: Option<String>,
    pub error: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<String>,
}

/// Shared HTTP client instance for efficiency
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// Dashboard API Client
pub struct DashboardApiClient {
    config: DashboardApiConfig,
}

impl DashboardApiClient {
    pub fn new() -> Self {
        Self {
            config: DashboardApiConfig::default(),
        }
    }

    pub fn with_config(config: DashboardApiConfig) -> Self {
        Self { config }
    }

    /// Get shared HTTP client instance
    fn get_http_client() -> &'static reqwest::Client {
        HTTP_CLIENT.get_or_init(|| reqwest::Client::new())
    }

    /// Get dashboard overview metrics
    pub async fn get_overview(
        &self,
    ) -> Result<NestJsApiResponse<DashboardOverviewResponse>, ApiError> {
        self.make_request::<DashboardOverviewResponse>("/dashboard/overview", None)
            .await
    }

    /// Get dashboard overview metrics with authentication
    pub async fn get_overview_with_auth(
        &self,
        auth_token: Option<&str>,
    ) -> Result<NestJsApiResponse<DashboardOverviewResponse>, ApiError> {
        self.make_request::<DashboardOverviewResponse>("/dashboard/overview", auth_token)
            .await
    }

    /// Get user analytics data
    pub async fn get_user_analytics(
        &self,
    ) -> Result<NestJsApiResponse<UserAnalyticsResponse>, ApiError> {
        self.make_request::<UserAnalyticsResponse>("/dashboard/users", None)
            .await
    }

    /// Get financial analytics data
    pub async fn get_financial_analytics(
        &self,
    ) -> Result<NestJsApiResponse<FinancialAnalyticsResponse>, ApiError> {
        self.make_request::<FinancialAnalyticsResponse>("/dashboard/financial", None)
            .await
    }

    /// Get operational metrics data
    pub async fn get_operational_metrics(
        &self,
    ) -> Result<NestJsApiResponse<OperationalMetricsResponse>, ApiError> {
        self.make_request::<OperationalMetricsResponse>("/dashboard/operations", None)
            .await
    }

    /// Get custom analytics with date range
    pub async fn get_custom_analytics(
        &self,
        start_date: &str,
        end_date: &str,
        metrics: &[&str],
        granularity: &str,
    ) -> Result<NestJsApiResponse<serde_json::Value>, ApiError> {
        let metrics_string = metrics.join(",");
        let query_params = vec![
            ("start_date", start_date),
            ("end_date", end_date),
            ("metrics", &metrics_string),
            ("granularity", granularity),
        ];

        self.make_request_with_params::<serde_json::Value>(
            "/dashboard/analytics/custom",
            &query_params,
        )
        .await
    }

    /// Export dashboard data
    pub async fn export_dashboard_data(
        &self,
        export_request: &ExportRequest,
    ) -> Result<NestJsApiResponse<ExportResponse>, ApiError> {
        self.make_post_request("/dashboard/export", export_request)
            .await
    }

    /// Get export status
    pub async fn get_export_status(
        &self,
        export_id: &str,
    ) -> Result<NestJsApiResponse<ExportStatus>, ApiError> {
        let endpoint = format!("/dashboard/export/{}/status", export_id);
        self.make_request::<ExportStatus>(&endpoint, None).await
    }

    /// Make HTTP request with authentication
    async fn make_request<T>(
        &self,
        endpoint: &str,
        auth_token: Option<&str>,
    ) -> Result<NestJsApiResponse<T>, ApiError>
    where
        T: for<'de> Deserialize<'de>,
    {
        #[cfg(feature = "ssr")]
        {
            use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

            let client = Self::get_http_client();
            let url = format!("{}{}", self.config.base_url, endpoint);

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            if let Some(token) = auth_token {
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {}", token))
                        .map_err(|e| ApiError::BadRequest(format!("Invalid auth token: {}", e)))?,
                );
            }

            let response = client
                .get(&url)
                .headers(headers)
                .timeout(self.config.timeout)
                .send()
                .await
                .map_err(|e| ApiError::NetworkError(e.to_string()))?;

            if response.status().is_success() {
                response
                    .json::<NestJsApiResponse<T>>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))
            } else {
                match response.status().as_u16() {
                    401 => Err(ApiError::Unauthorized),
                    404 => Err(ApiError::NotFound),
                    400..=499 => Err(ApiError::BadRequest(format!(
                        "Client error: {}",
                        response.status()
                    ))),
                    500..=599 => Err(ApiError::ServerError(format!(
                        "Server error: {}",
                        response.status()
                    ))),
                    _ => Err(ApiError::NetworkError(format!(
                        "Unexpected status: {}",
                        response.status()
                    ))),
                }
            }
        }

        #[cfg(not(feature = "ssr"))]
        {
            // Client-side implementation would go here
            // For now, return an error since client-side requests to NestJS need CORS setup
            Err(ApiError::ServerError(
                "Client-side API calls not implemented yet".to_string(),
            ))
        }
    }

    /// Make HTTP request with query parameters
    async fn make_request_with_params<T>(
        &self,
        endpoint: &str,
        query_params: &[(&str, &str)],
    ) -> Result<NestJsApiResponse<T>, ApiError>
    where
        T: for<'de> Deserialize<'de>,
    {
        #[cfg(feature = "ssr")]
        {
            use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

            let client = Self::get_http_client();
            let url = format!("{}{}", self.config.base_url, endpoint);

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            let response = client
                .get(&url)
                .headers(headers)
                .query(query_params)
                .timeout(self.config.timeout)
                .send()
                .await
                .map_err(|e| ApiError::NetworkError(e.to_string()))?;

            if response.status().is_success() {
                response
                    .json::<NestJsApiResponse<T>>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))
            } else {
                match response.status().as_u16() {
                    401 => Err(ApiError::Unauthorized),
                    404 => Err(ApiError::NotFound),
                    400..=499 => Err(ApiError::BadRequest(format!(
                        "Client error: {}",
                        response.status()
                    ))),
                    500..=599 => Err(ApiError::ServerError(format!(
                        "Server error: {}",
                        response.status()
                    ))),
                    _ => Err(ApiError::NetworkError(format!(
                        "Unexpected status: {}",
                        response.status()
                    ))),
                }
            }
        }

        #[cfg(not(feature = "ssr"))]
        {
            Err(ApiError::ServerError(
                "Client-side API calls not implemented yet".to_string(),
            ))
        }
    }

    /// Make HTTP POST request with JSON body
    async fn make_post_request<T, U>(
        &self,
        endpoint: &str,
        data: &T,
    ) -> Result<NestJsApiResponse<U>, ApiError>
    where
        T: Serialize,
        U: for<'de> Deserialize<'de>,
    {
        #[cfg(feature = "ssr")]
        {
            use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

            let client = Self::get_http_client();
            let url = format!("{}{}", self.config.base_url, endpoint);

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            let response = client
                .post(&url)
                .headers(headers)
                .json(data)
                .timeout(self.config.timeout)
                .send()
                .await
                .map_err(|e| ApiError::NetworkError(e.to_string()))?;

            if response.status().is_success() {
                response
                    .json::<NestJsApiResponse<U>>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))
            } else {
                match response.status().as_u16() {
                    401 => Err(ApiError::Unauthorized),
                    404 => Err(ApiError::NotFound),
                    400..=499 => Err(ApiError::BadRequest(format!(
                        "Client error: {}",
                        response.status()
                    ))),
                    500..=599 => Err(ApiError::ServerError(format!(
                        "Server error: {}",
                        response.status()
                    ))),
                    _ => Err(ApiError::NetworkError(format!(
                        "Unexpected status: {}",
                        response.status()
                    ))),
                }
            }
        }

        #[cfg(not(feature = "ssr"))]
        {
            Err(ApiError::ServerError(
                "Client-side API calls not implemented yet".to_string(),
            ))
        }
    }
}

impl Default for DashboardApiClient {
    fn default() -> Self {
        Self::new()
    }
}
