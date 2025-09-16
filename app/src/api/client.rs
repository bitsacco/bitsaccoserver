use crate::api::dashboard_client::DashboardApiClient;
use leptos::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Shared dashboard client instance for efficiency
static DASHBOARD_CLIENT: OnceLock<DashboardApiClient> = OnceLock::new();

/// Get shared dashboard client instance
fn get_dashboard_client() -> &'static DashboardApiClient {
    DASHBOARD_CLIENT.get_or_init(|| DashboardApiClient::new())
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
                if auth_str.starts_with("Bearer ") {
                    return Some(auth_str[7..].to_string());
                }
            }
        }

        // Also check for token in cookies
        if let Some(cookie_header) = headers.get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                // Simple cookie parsing for auth token
                for part in cookie_str.split(';') {
                    let part = part.trim();
                    if part.starts_with("auth_token=") {
                        return Some(part[11..].to_string());
                    }
                    if part.starts_with("access_token=") {
                        return Some(part[13..].to_string());
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

// SSR-compatible server functions for data fetching
#[server(GetGroups, "/api", "GetJson")]
pub async fn get_groups(
    page: Option<u32>,
    limit: Option<u32>,
    search: Option<String>,
) -> Result<ApiResponse<PaginatedResponse<crate::pages::groups::GroupResponse>>, ServerFnError> {
    use crate::pages::groups::GroupResponse;
    use crate::repositories::Repositories;
    use crate::server::AppConfig;
    use std::sync::Arc;

    // Use direct database connection approach since context injection is complex
    let config =
        AppConfig::from_env().map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;

    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.acquire_timeout,
            ))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));

        sea_orm::Database::connect(opt)
            .await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };

    let repositories = Repositories::new(Arc::new(database));

    // Handle pagination parameters
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(25);

    // Create basic pagination parameters since repository filters may differ
    let offset = (page - 1) as u64;

    // Call real repository with pagination
    let all_groups = repositories
        .groups
        .find_all()
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    // Apply pagination and search filtering manually for now
    let mut filtered_groups = all_groups;

    // Apply search filter if provided
    if let Some(search_term) = &search {
        let search_lower = search_term.to_lowercase();
        filtered_groups.retain(|group| {
            group.name.to_lowercase().contains(&search_lower)
                || group
                    .description
                    .as_ref()
                    .is_some_and(|desc| desc.to_lowercase().contains(&search_lower))
        });
    }

    let total = filtered_groups.len() as u64;
    let start = offset as usize;
    let end = std::cmp::min(start + limit as usize, filtered_groups.len());
    let paginated_items = if start < filtered_groups.len() {
        filtered_groups[start..end].to_vec()
    } else {
        Vec::new()
    };

    // Convert repository entities to frontend response format
    let group_responses: Vec<GroupResponse> = paginated_items
        .into_iter()
        .map(|group| {
            GroupResponse {
                id: group.id,
                name: group.name,
                description: group.description,
                group_type: format!("{:?}", group.group_type),
                status: format!("{:?}", group.status),
                member_count: None,   // TODO: Calculate from member relationships
                children_count: None, // TODO: Calculate from child group relationships
            }
        })
        .collect();

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let response = ApiResponse {
        success: true,
        data: Some(PaginatedResponse {
            data: group_responses,
            page,
            limit,
            total,
            total_pages,
        }),
        message: Some("Groups retrieved successfully".to_string()),
        errors: None,
    };
    Ok(response)
}

#[server(GetMembers, "/api", "GetJson")]
pub async fn get_members(
    page: Option<u32>,
    limit: Option<u32>,
    search: Option<String>,
) -> Result<ApiResponse<PaginatedResponse<crate::pages::members::MemberResponse>>, ServerFnError> {
    use crate::pages::members::MemberResponse;
    use crate::repositories::Repositories;
    use crate::server::AppConfig;
    use std::sync::Arc;

    // Use direct database connection approach since context injection is complex
    let config =
        AppConfig::from_env().map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;

    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.acquire_timeout,
            ))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));

        sea_orm::Database::connect(opt)
            .await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };

    let repositories = Repositories::new(Arc::new(database));

    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(25);

    // Create basic pagination parameters
    let offset = (page - 1) as u64;

    // Call real repository
    let all_members = repositories
        .members
        .find_all()
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    // Apply search filtering manually for now
    let mut filtered_members = all_members;

    // Apply search filter if provided
    if let Some(search_term) = &search {
        let search_lower = search_term.to_lowercase();
        filtered_members.retain(|member| {
            member.name.to_lowercase().contains(&search_lower)
                || member
                    .email
                    .as_ref()
                    .is_some_and(|email| email.to_lowercase().contains(&search_lower))
                || member.member_number.to_lowercase().contains(&search_lower)
        });
    }

    let total = filtered_members.len() as u64;
    let start = offset as usize;
    let end = std::cmp::min(start + limit as usize, filtered_members.len());
    let paginated_items = if start < filtered_members.len() {
        filtered_members[start..end].to_vec()
    } else {
        Vec::new()
    };

    // Convert repository entities to frontend response format
    let member_responses: Vec<MemberResponse> = paginated_items
        .into_iter()
        .map(|member| {
            // For now, groups field will be None as we need to implement relationship loading
            // TODO: Implement loading member groups via repository relationships
            let groups = None;

            MemberResponse {
                id: member.id,
                member_number: member.member_number,
                name: member.name,
                email: member.email,
                phone: member.phone,
                status: format!("{:?}", member.status),
                shares_count: None, // TODO: Calculate from shares relationship
                total_shares_value: None, // TODO: Calculate from shares relationship
                groups,
            }
        })
        .collect();

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let response = ApiResponse {
        success: true,
        data: Some(PaginatedResponse {
            data: member_responses,
            page,
            limit,
            total,
            total_pages,
        }),
        message: Some("Members retrieved successfully".to_string()),
        errors: None,
    };
    Ok(response)
}

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

#[server(GetShares, "/api")]
pub async fn get_shares(
    page: Option<u32>,
    limit: Option<u32>,
    search: Option<String>,
) -> Result<ApiResponse<PaginatedResponse<crate::pages::shares::ShareOfferResponse>>, ServerFnError>
{
    use crate::pages::shares::ShareOfferResponse;
    use crate::repositories::Repositories;
    use crate::server::AppConfig;
    use std::sync::Arc;

    // Use direct database connection approach since context injection is complex
    let config =
        AppConfig::from_env().map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;

    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.acquire_timeout,
            ))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));

        sea_orm::Database::connect(opt)
            .await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };

    let repositories = Repositories::new(Arc::new(database));

    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(25);

    // Create basic pagination parameters
    let offset = (page - 1) as u64;

    // Call real repository
    let all_offers = repositories
        .share_offers
        .find_all()
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    // Apply search filtering manually for now
    let mut filtered_offers = all_offers;

    // Apply search filter if provided
    if let Some(search_term) = &search {
        let search_lower = search_term.to_lowercase();
        filtered_offers.retain(|offer| {
            offer.name.to_lowercase().contains(&search_lower)
                || offer
                    .description
                    .as_ref()
                    .is_some_and(|desc| desc.to_lowercase().contains(&search_lower))
        });
    }

    let total = filtered_offers.len() as u64;
    let start = offset as usize;
    let end = std::cmp::min(start + limit as usize, filtered_offers.len());
    let paginated_items = if start < filtered_offers.len() {
        filtered_offers[start..end].to_vec()
    } else {
        Vec::new()
    };

    // Convert repository entities to frontend response format
    let share_responses: Vec<ShareOfferResponse> = paginated_items
        .into_iter()
        .map(|offer| {
            // Calculate progress percentage
            let sold_quantity = offer.shares_sold;
            let progress = if offer.total_shares_available > rust_decimal::Decimal::ZERO {
                (sold_quantity / offer.total_shares_available * rust_decimal::Decimal::from(100))
                    .to_f64()
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            ShareOfferResponse {
                id: offer.id,
                title: offer.name,
                description: offer.description,
                total_quantity: offer.total_shares_available,
                available_quantity: offer.shares_remaining,
                price_per_share: offer.price_per_share,
                total_value: offer.total_shares_available * offer.price_per_share,
                status: format!("{:?}", offer.status),
                expires_at: offer
                    .valid_until
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
                progress,
            }
        })
        .collect();

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let response = ApiResponse {
        success: true,
        data: Some(PaginatedResponse {
            data: share_responses,
            page,
            limit,
            total,
            total_pages,
        }),
        message: Some("Share offers retrieved successfully".to_string()),
        errors: None,
    };
    Ok(response)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemberRequest {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub member_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemberRequest {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub member_number: Option<String>,
}

#[server(CreateMember, "/api")]
pub async fn create_member(
    request: CreateMemberRequest,
) -> Result<ApiResponse<crate::pages::members::MemberResponse>, ServerFnError> {
    use crate::pages::members::MemberResponse;
    use crate::repositories::Repositories;
    use crate::server::AppConfig;
    use entity::members::{self, ActiveModel};
    use sea_orm::Set;
    use std::sync::Arc;
    use uuid::Uuid;

    // Use direct database connection approach since context injection is complex
    let config =
        AppConfig::from_env().map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;

    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.acquire_timeout,
            ))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));

        sea_orm::Database::connect(opt)
            .await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };

    let repositories = Repositories::new(Arc::new(database));

    // Generate member number if not provided
    let member_number = if let Some(number) = request.member_number {
        number
    } else {
        // Generate a unique member number
        let count = repositories
            .members
            .count()
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
        format!("M{:06}", count + 1)
    };

    // Create new member
    let new_member = ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(request.name),
        email: Set(request.email),
        phone: Set(request.phone),
        member_number: Set(member_number),
        status: Set(members::MemberStatus::Active),
        created_at: Set(chrono::Utc::now().fixed_offset()),
        updated_at: Set(chrono::Utc::now().fixed_offset()),
        ..Default::default()
    };

    let saved_member = repositories
        .members
        .create(new_member)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    // Convert to response format
    let member_response = MemberResponse {
        id: saved_member.id,
        member_number: saved_member.member_number,
        name: saved_member.name,
        email: saved_member.email,
        phone: saved_member.phone,
        status: format!("{:?}", saved_member.status),
        groups: None,
        shares_count: Some(0),
        total_shares_value: Some(rust_decimal::Decimal::ZERO),
    };

    let response = ApiResponse {
        data: Some(member_response),
        success: true,
        message: Some("Member created successfully".to_string()),
        errors: None,
    };

    Ok(response)
}

#[server(UpdateMember, "/api")]
pub async fn update_member(
    request: UpdateMemberRequest,
) -> Result<ApiResponse<crate::pages::members::MemberResponse>, ServerFnError> {
    use crate::pages::members::MemberResponse;
    use crate::repositories::Repositories;
    use crate::server::AppConfig;
    use entity::members::ActiveModel;
    use sea_orm::Set;
    use std::sync::Arc;

    // Use direct database connection approach since context injection is complex
    let config =
        AppConfig::from_env().map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;

    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.acquire_timeout,
            ))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));

        sea_orm::Database::connect(opt)
            .await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };

    let repositories = Repositories::new(Arc::new(database));

    // Build ActiveModel for partial update
    let mut update_model = ActiveModel {
        id: Set(request.id),
        updated_at: Set(chrono::Utc::now().fixed_offset()),
        ..Default::default()
    };

    if let Some(name) = request.name {
        update_model.name = Set(name);
    }
    if let Some(email) = request.email {
        update_model.email = Set(Some(email));
    }
    if let Some(phone) = request.phone {
        update_model.phone = Set(Some(phone));
    }
    if let Some(member_number) = request.member_number {
        update_model.member_number = Set(member_number);
    }

    let updated_member = repositories
        .members
        .update(request.id, update_model)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    // Convert to response format
    let member_response = MemberResponse {
        id: updated_member.id,
        member_number: updated_member.member_number,
        name: updated_member.name,
        email: updated_member.email,
        phone: updated_member.phone,
        status: format!("{:?}", updated_member.status),
        groups: None,
        shares_count: Some(0),
        total_shares_value: Some(rust_decimal::Decimal::ZERO),
    };

    let response = ApiResponse {
        data: Some(member_response),
        success: true,
        message: Some("Member updated successfully".to_string()),
        errors: None,
    };

    Ok(response)
}

#[server(DeleteMember, "/api")]
pub async fn delete_member(member_id: uuid::Uuid) -> Result<ApiResponse<()>, ServerFnError> {
    use crate::repositories::Repositories;
    use crate::server::AppConfig;
    use std::sync::Arc;

    // Use direct database connection approach since context injection is complex
    let config =
        AppConfig::from_env().map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;

    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.acquire_timeout,
            ))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));

        sea_orm::Database::connect(opt)
            .await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };

    let repositories = Repositories::new(Arc::new(database));

    repositories
        .members
        .delete(member_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let response = ApiResponse {
        data: Some(()),
        success: true,
        message: Some("Member deleted successfully".to_string()),
        errors: None,
    };

    Ok(response)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub group_type: String,
    pub parent_group_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub group_type: Option<String>,
    pub parent_group_id: Option<uuid::Uuid>,
}

#[server(CreateGroup, "/api")]
pub async fn create_group(
    request: CreateGroupRequest,
) -> Result<ApiResponse<crate::pages::groups::GroupResponse>, ServerFnError> {
    use crate::pages::groups::GroupResponse;
    use crate::repositories::Repositories;
    use crate::server::AppConfig;
    use entity::groups::{self, ActiveModel};
    use sea_orm::Set;
    use std::sync::Arc;
    use uuid::Uuid;

    // Use direct database connection approach since context injection is complex
    let config =
        AppConfig::from_env().map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;

    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.acquire_timeout,
            ))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));

        sea_orm::Database::connect(opt)
            .await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };

    let repositories = Repositories::new(Arc::new(database));

    // Parse group type
    let parsed_group_type = match request.group_type.as_str() {
        "Organization" => groups::GroupType::Organization,
        "Chama" => groups::GroupType::Chama,
        _ => groups::GroupType::Chama, // Default fallback
    };

    // Create new group
    let new_group = ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(request.name),
        description: Set(request.description),
        group_type: Set(parsed_group_type),
        parent_id: Set(request.parent_group_id),
        status: Set(groups::GroupStatus::Active),
        created_at: Set(chrono::Utc::now().fixed_offset()),
        updated_at: Set(chrono::Utc::now().fixed_offset()),
        ..Default::default()
    };

    let saved_group = repositories
        .groups
        .create(new_group)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    // Convert to response format
    let group_response = GroupResponse {
        id: saved_group.id,
        name: saved_group.name,
        description: saved_group.description,
        group_type: format!("{:?}", saved_group.group_type),
        status: format!("{:?}", saved_group.status),
        member_count: Some(0),
        children_count: Some(0),
    };

    let response = ApiResponse {
        data: Some(group_response),
        success: true,
        message: Some("Group created successfully".to_string()),
        errors: None,
    };

    Ok(response)
}

#[server(UpdateGroup, "/api")]
pub async fn update_group(
    request: UpdateGroupRequest,
) -> Result<ApiResponse<crate::pages::groups::GroupResponse>, ServerFnError> {
    use crate::pages::groups::GroupResponse;
    use crate::repositories::Repositories;
    use crate::server::AppConfig;
    use entity::groups::{self, ActiveModel};
    use sea_orm::Set;
    use std::sync::Arc;

    // Use direct database connection approach since context injection is complex
    let config =
        AppConfig::from_env().map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;

    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.acquire_timeout,
            ))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));

        sea_orm::Database::connect(opt)
            .await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };

    let repositories = Repositories::new(Arc::new(database));

    // Build ActiveModel for partial update
    let mut update_model = ActiveModel {
        id: Set(request.id),
        updated_at: Set(chrono::Utc::now().fixed_offset()),
        ..Default::default()
    };

    if let Some(name) = request.name {
        update_model.name = Set(name);
    }
    if let Some(description) = request.description {
        update_model.description = Set(Some(description));
    }
    if let Some(group_type_str) = request.group_type {
        let parsed_group_type = match group_type_str.as_str() {
            "Organization" => groups::GroupType::Organization,
            "Chama" => groups::GroupType::Chama,
            _ => groups::GroupType::Chama,
        };
        update_model.group_type = Set(parsed_group_type);
    }
    if let Some(parent_id) = request.parent_group_id {
        update_model.parent_id = Set(Some(parent_id));
    }

    let updated_group = repositories
        .groups
        .update(request.id, update_model)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    // Convert to response format
    let group_response = GroupResponse {
        id: updated_group.id,
        name: updated_group.name,
        description: updated_group.description,
        group_type: format!("{:?}", updated_group.group_type),
        status: format!("{:?}", updated_group.status),
        member_count: Some(0),
        children_count: Some(0),
    };

    let response = ApiResponse {
        data: Some(group_response),
        success: true,
        message: Some("Group updated successfully".to_string()),
        errors: None,
    };

    Ok(response)
}

#[server(DeleteGroup, "/api")]
pub async fn delete_group(group_id: uuid::Uuid) -> Result<ApiResponse<()>, ServerFnError> {
    use crate::repositories::Repositories;
    use crate::server::AppConfig;
    use std::sync::Arc;

    // Use direct database connection approach since context injection is complex
    let config =
        AppConfig::from_env().map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;

    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.acquire_timeout,
            ))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));

        sea_orm::Database::connect(opt)
            .await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };

    let repositories = Repositories::new(Arc::new(database));

    repositories
        .groups
        .delete(group_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let response = ApiResponse {
        data: Some(()),
        success: true,
        message: Some("Group deleted successfully".to_string()),
        errors: None,
    };

    Ok(response)
}
