use crate::api::dashboard_client::DashboardApiClient;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Shared dashboard client instance for efficiency
static DASHBOARD_CLIENT: OnceLock<DashboardApiClient> = OnceLock::new();

/// Get shared dashboard client instance
fn get_dashboard_client() -> &'static DashboardApiClient {
    DASHBOARD_CLIENT.get_or_init(DashboardApiClient::new)
}

/// Extract JWT token from request context (SSR only)
#[cfg(feature = "ssr")]
async fn extract_auth_token_from_request() -> Option<String> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    // Try to extract headers from request context
    if let Ok(headers) = extract::<HeaderMap>().await {
        if let Some(auth_header) = headers.get("authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    return Some(token.to_string());
                }
            }
        }

        // Also check for token in cookies
        if let Some(cookie_header) = headers.get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                // Simple cookie parsing for auth token
                for part in cookie_str.split(';') {
                    let part = part.trim();
                    if let Some(token) = part.strip_prefix("auth_token=") {
                        return Some(token.to_string());
                    }
                    if let Some(token) = part.strip_prefix("access_token=") {
                        return Some(token.to_string());
                    }
                }
            }
        }
    }

    None
}

/// Extract JWT token from request context (client-side - returns None)
#[cfg(not(feature = "ssr"))]
async fn extract_auth_token_from_request() -> Option<String> {
    // On client side, we can't extract from server request context
    None
}

#[derive(Debug, Clone)]
pub enum ApiError {
    NetworkError(String),
    Unauthorized,
    NotFound,
    BadRequest(String),
    ServerError(String),
    ParseError(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::Unauthorized => write!(f, "Unauthorized"),
            ApiError::NotFound => write!(f, "Resource not found"),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::ServerError(msg) => write!(f, "Server error: {}", msg),
            ApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

// Common query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort: Option<String>,
    pub order: Option<String>, // "asc" or "desc"
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(25),
            sort: None,
            order: Some("asc".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    #[serde(flatten)]
    pub pagination: PaginationQuery,
}

// Common response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub limit: u32,
    pub total: u64,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub errors: Option<Vec<String>>,
}

/// Server function for getting dashboard metrics from NestJS API
#[server(GetDashboardMetrics, "/api", "GetJson")]
pub async fn get_dashboard_metrics(
) -> Result<ApiResponse<crate::pages::dashboard::DashboardMetrics>, ServerFnError> {
    // Extract auth token from the current request
    let auth_token = extract_auth_token_from_request().await;
    let client = get_dashboard_client();

    match client.get_overview_with_auth(auth_token.as_deref()).await {
        Ok(nestjs_response) => {
            if nestjs_response.success {
                let converted_metrics =
                    convert_nestjs_overview_to_dashboard_metrics(nestjs_response.data);

                Ok(ApiResponse {
                    success: true,
                    data: Some(converted_metrics),
                    message: Some("Dashboard metrics retrieved from NestJS API".to_string()),
                    errors: None,
                })
            } else {
                leptos::logging::warn!(
                    "Dashboard: NestJS API returned success=false: {:?}",
                    nestjs_response.message
                );
                Err(ServerFnError::new(format!(
                    "NestJS API error: {:?}",
                    nestjs_response.message
                )))
            }
        }
        Err(api_error) => {
            leptos::logging::error!(
                "Dashboard: Failed to connect to NestJS API: {:?}",
                api_error
            );
            Err(ServerFnError::new(format!(
                "API connection failed: {:?}",
                api_error
            )))
        }
    }
}

/// Convert NestJS overview response to current dashboard metrics format
fn convert_nestjs_overview_to_dashboard_metrics(
    overview: crate::api::dashboard_client::DashboardOverviewResponse,
) -> crate::pages::dashboard::DashboardMetrics {
    use rust_decimal::Decimal;

    crate::pages::dashboard::DashboardMetrics {
        shareholders: crate::pages::dashboard::ShareholderSummary {
            total_shareholders: overview.summary.total_members,
            member_shareholders: overview.summary.total_members, // Estimate for now
            group_shareholders: 0, // TODO: Get from NestJS when available
            active_shareholders: overview.summary.active_members_today,
        },
        market: crate::pages::dashboard::MarketAnalytics {
            // Use actual data from NestJS API without capping
            total_market_value_kes: Decimal::from_f64_retain(overview.summary.total_volume.amount)
                .unwrap_or_default(),
            total_shares_in_circulation: Decimal::from(overview.summary.total_members), // Actual member count
            share_price_kes: Decimal::from(1000), // Fixed at 1000 KES per share
        },
        offers: crate::pages::dashboard::ShareOfferAnalytics {
            total_offers: 10,              // TODO: Get from NestJS shares metrics
            active_offers: 5,              // TODO: Get from NestJS shares metrics
            completed_offers: 5,           // TODO: Get from NestJS shares metrics
            average_completion_rate: 75.0, // TODO: Calculate from actual data
        },
        transactions: crate::pages::dashboard::TransactionAnalytics {
            total_transactions: overview.summary.transaction_count.total,
            total_transaction_value_kes: Decimal::from_f64_retain(
                overview.summary.total_volume.amount,
            )
            .unwrap_or_default(),
            average_transaction_size_kes: {
                let avg = if overview.summary.transaction_count.total > 0 {
                    overview.summary.total_volume.amount
                        / overview.summary.transaction_count.total as f64
                } else {
                    0.0
                };
                Decimal::from_f64_retain(avg).unwrap_or_else(|| {
                    leptos::logging::warn!(
                        "Dashboard: Failed to convert average transaction size to Decimal: {}",
                        avg
                    );
                    Decimal::ZERO
                })
            },
        },
    }
}

/// Server function for getting user analytics from NestJS API
#[server(GetUserAnalytics, "/api", "GetJson")]
pub async fn get_user_analytics(
) -> Result<ApiResponse<crate::api::dashboard_client::UserAnalyticsResponse>, ServerFnError> {
    // TEMPORARY: Bypass API call to prevent stack overflow
    Err(ServerFnError::new("Temporarily disabled".to_string()))
}

/// Server function for getting financial analytics from NestJS API
#[server(GetFinancialAnalytics, "/api", "GetJson")]
pub async fn get_financial_analytics(
) -> Result<ApiResponse<crate::api::dashboard_client::FinancialAnalyticsResponse>, ServerFnError> {
    let client = get_dashboard_client();

    match client.get_financial_analytics().await {
        Ok(nestjs_response) => {
            if nestjs_response.success {
                let response = ApiResponse {
                    success: true,
                    data: Some(nestjs_response.data),
                    message: Some(
                        "Financial analytics retrieved successfully from NestJS API".to_string(),
                    ),
                    errors: None,
                };
                Ok(response)
            } else {
                let response = ApiResponse {
                    success: false,
                    data: None,
                    message: Some("Failed to retrieve financial analytics".to_string()),
                    errors: nestjs_response.errors,
                };
                Ok(response)
            }
        }
        Err(api_error) => {
            leptos::logging::error!(
                "Failed to connect to NestJS financial analytics API: {:?}",
                api_error
            );
            Err(ServerFnError::new(format!("API Error: {}", api_error)))
        }
    }
}

/// Server function for getting operational metrics from NestJS API
#[server(GetOperationalMetrics, "/api", "GetJson")]
pub async fn get_operational_metrics(
) -> Result<ApiResponse<crate::api::dashboard_client::OperationalMetricsResponse>, ServerFnError> {
    let client = get_dashboard_client();

    match client.get_operational_metrics().await {
        Ok(nestjs_response) => {
            if nestjs_response.success {
                let response = ApiResponse {
                    success: true,
                    data: Some(nestjs_response.data),
                    message: Some(
                        "Operational metrics retrieved successfully from NestJS API".to_string(),
                    ),
                    errors: None,
                };
                Ok(response)
            } else {
                let response = ApiResponse {
                    success: false,
                    data: None,
                    message: Some("Failed to retrieve operational metrics".to_string()),
                    errors: nestjs_response.errors,
                };
                Ok(response)
            }
        }
        Err(api_error) => {
            leptos::logging::error!(
                "Failed to connect to NestJS operational metrics API: {:?}",
                api_error
            );
            Err(ServerFnError::new(format!("API Error: {}", api_error)))
        }
    }
}

/// Server function for custom analytics with date range
#[server(GetCustomAnalytics, "/api", "GetJson")]
pub async fn get_custom_analytics(
    start_date: String,
    end_date: String,
    metrics: Vec<String>,
    granularity: String,
) -> Result<ApiResponse<serde_json::Value>, ServerFnError> {
    let client = get_dashboard_client();

    // Convert Vec<String> to Vec<&str>
    let metrics_slice: Vec<&str> = metrics.iter().map(|s| s.as_str()).collect();

    match client
        .get_custom_analytics(&start_date, &end_date, &metrics_slice, &granularity)
        .await
    {
        Ok(nestjs_response) => {
            if nestjs_response.success {
                let response = ApiResponse {
                    success: true,
                    data: Some(nestjs_response.data),
                    message: Some(
                        "Custom analytics retrieved successfully from NestJS API".to_string(),
                    ),
                    errors: None,
                };
                Ok(response)
            } else {
                let response = ApiResponse {
                    success: false,
                    data: None,
                    message: Some("Failed to retrieve custom analytics".to_string()),
                    errors: nestjs_response.errors,
                };
                Ok(response)
            }
        }
        Err(api_error) => {
            leptos::logging::error!(
                "Failed to connect to NestJS custom analytics API: {:?}",
                api_error
            );
            Err(ServerFnError::new(format!("API Error: {}", api_error)))
        }
    }
}

/// Server function for exporting dashboard data
#[server(ExportDashboardData, "/api")]
pub async fn export_dashboard_data(
    export_request: crate::api::dashboard_client::ExportRequest,
) -> Result<ApiResponse<crate::api::dashboard_client::ExportResponse>, ServerFnError> {
    let client = get_dashboard_client();

    match client.export_dashboard_data(&export_request).await {
        Ok(nestjs_response) => {
            if nestjs_response.success {
                let response = ApiResponse {
                    success: true,
                    data: Some(nestjs_response.data),
                    message: Some("Export request submitted successfully".to_string()),
                    errors: None,
                };
                Ok(response)
            } else {
                let response = ApiResponse {
                    success: false,
                    data: None,
                    message: Some("Failed to submit export request".to_string()),
                    errors: nestjs_response.errors,
                };
                Ok(response)
            }
        }
        Err(api_error) => {
            leptos::logging::error!(
                "Failed to submit export request to NestJS API: {:?}",
                api_error
            );
            Err(ServerFnError::new(format!("API Error: {}", api_error)))
        }
    }
}

/// Server function for getting export status
#[server(GetExportStatus, "/api", "GetJson")]
pub async fn get_export_status(
    export_id: String,
) -> Result<ApiResponse<crate::api::dashboard_client::ExportStatus>, ServerFnError> {
    let client = get_dashboard_client();

    match client.get_export_status(&export_id).await {
        Ok(nestjs_response) => {
            if nestjs_response.success {
                let response = ApiResponse {
                    success: true,
                    data: Some(nestjs_response.data),
                    message: Some("Export status retrieved successfully".to_string()),
                    errors: None,
                };
                Ok(response)
            } else {
                let response = ApiResponse {
                    success: false,
                    data: None,
                    message: Some("Failed to get export status".to_string()),
                    errors: nestjs_response.errors,
                };
                Ok(response)
            }
        }
        Err(api_error) => {
            leptos::logging::error!(
                "Failed to get export status from NestJS API: {:?}",
                api_error
            );
            Err(ServerFnError::new(format!("API Error: {}", api_error)))
        }
    }
}

// SSR-compatible client stub for backend API modules
#[derive(Clone)]
pub struct ApiClient {
    #[allow(dead_code)]
    base_url: String,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: "/api".to_string(),
        }
    }

    pub async fn get<T: for<'a> Deserialize<'a>>(&self, _path: &str) -> Result<T, ApiError> {
        // In SSR mode, API calls should use server functions instead
        Err(ApiError::ServerError(
            "Use server functions for SSR-compatible API calls".to_string(),
        ))
    }

    pub async fn post<T: Serialize, R: for<'a> Deserialize<'a>>(
        &self,
        _path: &str,
        _data: &T,
    ) -> Result<R, ApiError> {
        Err(ApiError::ServerError(
            "Use server functions for SSR-compatible API calls".to_string(),
        ))
    }

    pub async fn put<T: Serialize, R: for<'a> Deserialize<'a>>(
        &self,
        _path: &str,
        _data: &T,
    ) -> Result<R, ApiError> {
        Err(ApiError::ServerError(
            "Use server functions for SSR-compatible API calls".to_string(),
        ))
    }

    pub async fn delete<R: for<'a> Deserialize<'a>>(&self, _path: &str) -> Result<R, ApiError> {
        Err(ApiError::ServerError(
            "Use server functions for SSR-compatible API calls".to_string(),
        ))
    }
}

// Global instance for convenience
static API_CLIENT: std::sync::OnceLock<ApiClient> = std::sync::OnceLock::new();

pub fn api() -> &'static ApiClient {
    API_CLIENT.get_or_init(ApiClient::new)
}

// NOTE: All Groups, Members, Shares server functions have been removed
// These operations should now be handled by the backend adapter pattern
// using the abstraction layer in api/abstraction.rs
//
// For CRUD operations on Groups, Members, and Shares:
// 1. Use the AbstractedApiClient in api/abstraction.rs
// 2. Configure API_BACKEND=nestjs environment variable
// 3. Backend adapter will route to appropriate implementation (NestJS or Rust with friendly errors)
//
// This maintains clean separation between UI and backend implementation
