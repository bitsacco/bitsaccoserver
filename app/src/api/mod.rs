// Existing SSR API modules
pub mod client;
pub mod dashboard_client;

// API modules
pub mod backends;
pub mod config;
pub mod errors;
pub mod traits;
pub mod types;

// Re-export commonly used items for SSR mode
pub use client::{
    api, export_dashboard_data, get_custom_analytics, get_dashboard_metrics, get_export_status,
    get_financial_analytics, get_operational_metrics, get_user_analytics, ApiClient, ApiError,
    ApiResponse, PaginatedResponse, PaginationQuery, SearchQuery,
};

// Re-export config and error items
pub use config::{ApiConfig, Backend};
pub use errors::{ApiError as AbstractedApiError, ApiResult};

// Specific re-exports to avoid ambiguous glob imports
pub use traits::{AuthApi, GroupsApi, UsersApi, WalletsApi};
pub use types::{
    auth::{AuthRequest, AuthResponse, LoginRequest, LogoutRequest, LogoutResponse},
    common::{PaginationQuery as ApiPaginationQuery, SearchQuery as ApiSearchQuery},
    user::{FindUserRequest, UpdateUserRequest, User},
};

use axum::response::{IntoResponse, Json};
use serde_json::json;

// Rust backend API router - removed to eliminate backend implementation
// This router was responsible for backend database operations
// All API calls should now go through the adapter pattern in abstraction.rs

pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "bitsacco-server-api"
    }))
}

pub async fn api_info() -> impl IntoResponse {
    Json(json!({
        "name": "Bitsacco Dashboard",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Leptos dashboard for SACCO management system",
        "mode": "frontend_only",
        "backend": std::env::var("API_BACKEND").unwrap_or_else(|_| "nestjs".to_string()),
        "api_endpoints": "Delegated to configured backend (NestJS or Rust adapter)"
    }))
}
