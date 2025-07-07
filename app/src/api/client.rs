use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use rust_decimal::prelude::ToPrimitive;

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
    let config = AppConfig::from_env()
        .map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;
    
    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.database.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));
        
        sea_orm::Database::connect(opt).await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };
    
    let repositories = Repositories::new(Arc::new(database));

    // Handle pagination parameters
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(25);

    // Create basic pagination parameters since repository filters may differ
    let offset = (page - 1) as u64;

    // Call real repository with pagination
    let all_groups = repositories.groups.find_all().await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
    
    // Apply pagination and search filtering manually for now
    let mut filtered_groups = all_groups;
    
    // Apply search filter if provided
    if let Some(search_term) = &search {
        let search_lower = search_term.to_lowercase();
        filtered_groups.retain(|group| {
            group.name.to_lowercase().contains(&search_lower)
                || group.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&search_lower))
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
    let group_responses: Vec<GroupResponse> = paginated_items.into_iter().map(|group| {
        GroupResponse {
            id: group.id,
            name: group.name,
            description: group.description,
            group_type: format!("{:?}", group.group_type),
            status: format!("{:?}", group.status),
            member_count: None, // TODO: Calculate from member relationships
            children_count: None, // TODO: Calculate from child group relationships
        }
    }).collect();

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
    let config = AppConfig::from_env()
        .map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;
    
    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.database.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));
        
        sea_orm::Database::connect(opt).await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };
    
    let repositories = Repositories::new(Arc::new(database));

    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(25);

    // Create basic pagination parameters
    let offset = (page - 1) as u64;

    // Call real repository
    let all_members = repositories.members.find_all().await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
    
    // Apply search filtering manually for now
    let mut filtered_members = all_members;
    
    // Apply search filter if provided
    if let Some(search_term) = &search {
        let search_lower = search_term.to_lowercase();
        filtered_members.retain(|member| {
            member.name.to_lowercase().contains(&search_lower)
                || member.email.as_ref().map_or(false, |email| email.to_lowercase().contains(&search_lower))
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
    let member_responses: Vec<MemberResponse> = paginated_items.into_iter().map(|member| {
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
    }).collect();

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
    use crate::services::Services;
    use crate::server::AppConfig;
    use crate::repositories::Repositories;
    use crate::services::auth::KeycloakConfig;
    use std::sync::Arc;

    // Use direct service initialization approach since context injection is complex
    let config = AppConfig::from_env()
        .map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;
    
    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.database.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));
        
        sea_orm::Database::connect(opt).await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };
    
    let repositories = Repositories::new(Arc::new(database.clone()));
    
    let keycloak_config = KeycloakConfig {
        realm: config.keycloak.realm.clone(),
        client_id: config.keycloak.client_id.clone(),
        client_secret: config.keycloak.client_secret.clone(),
        server_url: config.keycloak.auth_server_url.clone(),
    };
    
    let services = Services::new(Arc::new(database), repositories, keycloak_config);

    // Call real analytics service methods
    let shareholder_summary = services.analytics.get_shareholder_summary().await
        .map_err(|e| ServerFnError::new(format!("Analytics error: {}", e)))?;
    
    let market_analytics = services.analytics.get_market_analytics().await
        .map_err(|e| ServerFnError::new(format!("Analytics error: {}", e)))?;
    
    let offer_analytics = services.analytics.get_offer_analytics().await
        .map_err(|e| ServerFnError::new(format!("Analytics error: {}", e)))?;
    
    let transaction_analytics = services.analytics.get_transaction_analytics().await
        .map_err(|e| ServerFnError::new(format!("Analytics error: {}", e)))?;

    // Convert analytics service response to dashboard metrics format
    let dashboard_metrics = crate::pages::dashboard::DashboardMetrics {
        shareholders: crate::pages::dashboard::ShareholderSummary {
            total_shareholders: shareholder_summary.total_shareholders,
            member_shareholders: shareholder_summary.member_shareholders,
            group_shareholders: shareholder_summary.group_shareholders,
            active_shareholders: shareholder_summary.active_shareholders,
        },
        market: crate::pages::dashboard::MarketAnalytics {
            total_market_value: market_analytics.total_market_value,
            total_shares_in_circulation: market_analytics.total_shares_in_circulation,
            average_share_price: market_analytics.average_share_price,
        },
        offers: crate::pages::dashboard::ShareOfferAnalytics {
            total_offers: offer_analytics.total_offers,
            active_offers: offer_analytics.active_offers,
            completed_offers: offer_analytics.completed_offers,
            average_completion_rate: offer_analytics.average_completion_rate,
        },
        transactions: crate::pages::dashboard::TransactionAnalytics {
            total_transactions: transaction_analytics.total_transactions,
            total_transaction_value: transaction_analytics.total_transaction_value,
            average_transaction_size: transaction_analytics.average_transaction_size,
        },
    };

    let response = ApiResponse {
        success: true,
        data: Some(dashboard_metrics),
        message: Some("Dashboard metrics retrieved successfully".to_string()),
        errors: None,
    };

    Ok(response)
}

#[server(GetShares, "/api")]
pub async fn get_shares(
    page: Option<u32>,
    limit: Option<u32>,
    search: Option<String>,
) -> Result<ApiResponse<PaginatedResponse<crate::pages::shares::ShareOfferResponse>>, ServerFnError> {
    use crate::pages::shares::ShareOfferResponse;
    use crate::repositories::Repositories;
    use crate::server::AppConfig;
    use std::sync::Arc;

    // Use direct database connection approach since context injection is complex
    let config = AppConfig::from_env()
        .map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;
    
    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.database.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));
        
        sea_orm::Database::connect(opt).await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };
    
    let repositories = Repositories::new(Arc::new(database));

    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(25);

    // Create basic pagination parameters
    let offset = (page - 1) as u64;

    // Call real repository
    let all_offers = repositories.share_offers.find_all().await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
    
    // Apply search filtering manually for now
    let mut filtered_offers = all_offers;
    
    // Apply search filter if provided
    if let Some(search_term) = &search {
        let search_lower = search_term.to_lowercase();
        filtered_offers.retain(|offer| {
            offer.name.to_lowercase().contains(&search_lower)
                || offer.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&search_lower))
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
    let share_responses: Vec<ShareOfferResponse> = paginated_items.into_iter().map(|offer| {
        // Calculate progress percentage
        let sold_quantity = offer.shares_sold;
        let progress = if offer.total_shares_available > rust_decimal::Decimal::ZERO {
            (sold_quantity / offer.total_shares_available * rust_decimal::Decimal::from(100)).to_f64().unwrap_or(0.0)
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
            expires_at: offer.valid_until.map(|dt| dt.format("%Y-%m-%d").to_string()).unwrap_or_default(),
            progress,
        }
    }).collect();

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
    API_CLIENT.get_or_init(|| ApiClient::new())
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
    use std::sync::Arc;
    use entity::members::{self, ActiveModel};
    use sea_orm::{Set};
    use uuid::Uuid;

    // Use direct database connection approach since context injection is complex
    let config = AppConfig::from_env()
        .map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;
    
    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.database.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));
        
        sea_orm::Database::connect(opt).await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };
    
    let repositories = Repositories::new(Arc::new(database));

    // Generate member number if not provided
    let member_number = if let Some(number) = request.member_number {
        number
    } else {
        // Generate a unique member number
        let count = repositories.members.count().await
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

    let saved_member = repositories.members.create(new_member).await
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
    use std::sync::Arc;
    use entity::members::{ActiveModel};
    use sea_orm::{Set};

    // Use direct database connection approach since context injection is complex
    let config = AppConfig::from_env()
        .map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;
    
    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.database.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));
        
        sea_orm::Database::connect(opt).await
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

    let updated_member = repositories.members.update(request.id, update_model).await
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
pub async fn delete_member(
    member_id: uuid::Uuid,
) -> Result<ApiResponse<()>, ServerFnError> {
    use crate::repositories::Repositories;
    use crate::server::AppConfig;
    use std::sync::Arc;

    // Use direct database connection approach since context injection is complex
    let config = AppConfig::from_env()
        .map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;
    
    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.database.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));
        
        sea_orm::Database::connect(opt).await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };
    
    let repositories = Repositories::new(Arc::new(database));

    repositories.members.delete(member_id).await
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
    use std::sync::Arc;
    use entity::groups::{self, ActiveModel};
    use sea_orm::{Set};
    use uuid::Uuid;

    // Use direct database connection approach since context injection is complex
    let config = AppConfig::from_env()
        .map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;
    
    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.database.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));
        
        sea_orm::Database::connect(opt).await
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

    let saved_group = repositories.groups.create(new_group).await
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
    use std::sync::Arc;
    use entity::groups::{self, ActiveModel};
    use sea_orm::{Set};

    // Use direct database connection approach since context injection is complex
    let config = AppConfig::from_env()
        .map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;
    
    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.database.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));
        
        sea_orm::Database::connect(opt).await
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

    let updated_group = repositories.groups.update(request.id, update_model).await
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
pub async fn delete_group(
    group_id: uuid::Uuid,
) -> Result<ApiResponse<()>, ServerFnError> {
    use crate::repositories::Repositories;
    use crate::server::AppConfig;
    use std::sync::Arc;

    // Use direct database connection approach since context injection is complex
    let config = AppConfig::from_env()
        .map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;
    
    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.database.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime));
        
        sea_orm::Database::connect(opt).await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?
    };
    
    let repositories = Repositories::new(Arc::new(database));

    repositories.groups.delete(group_id).await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let response = ApiResponse {
        data: Some(()),
        success: true,
        message: Some("Group deleted successfully".to_string()),
        errors: None,
    };

    Ok(response)
}