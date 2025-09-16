use ::entity::shares::OwnerType;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Json},
    routing::{get, post},
    Extension, Router,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    middleware::auth::{auth_middleware, UserContext},
    repositories::Repositories,
    server::state::AppState as MainAppState,
    services::{
        share_purchase::{SharePurchaseRequest, ShareTransferRequest},
        Services,
    },
};

/// Builder for creating shares compatibility routers
struct SharesRouterBuilder {
    repositories: Repositories,
    services: Services,
}

impl SharesRouterBuilder {
    /// Create a new router builder
    pub fn new(repositories: Repositories, services: Services) -> Self {
        Self {
            repositories,
            services,
        }
    }

    /// Build the base router with all routes defined
    fn build_base_router() -> Router<UnifiedAppState> {
        Router::new()
            .route("/offer", post(create_share_offer))
            .route("/offers", get(get_all_share_offers))
            .route("/subscribe", post(subscribe_to_shares))
            .route("/transfer", post(transfer_shares))
            .route("/update", post(update_shares))
            .route("/transactions", get(get_all_transactions))
            .route("/transactions/:userId", get(get_user_transactions))
            .route(
                "/transactions/find/:sharesId",
                get(find_transaction_by_shares_id),
            )
    }

    /// Build a secure router with all routes using secure handlers
    fn build_secure_router() -> Router<UnifiedAppState> {
        Router::new()
            .route("/offer", post(secure_create_share_offer))
            .route("/offers", get(secure_get_all_share_offers))
            .route("/subscribe", post(secure_subscribe_to_shares))
            .route("/transfer", post(secure_transfer_shares))
            .route("/update", post(secure_update_shares))
            .route("/transactions", get(secure_get_all_transactions))
            .route("/transactions/:userId", get(secure_get_user_transactions))
            .route(
                "/transactions/find/:sharesId",
                get(secure_find_transaction_by_shares_id),
            )
    }

    /// Creates the shares compatibility router for NestJS API compatibility
    /// This router provides endpoints without the /api prefix for direct NestJS compatibility
    pub fn build<S>(self) -> Router<S> {
        Self::build_base_router().with_state(UnifiedAppState {
            main_state: None,
            repositories: self.repositories,
            services: self.services,
        })
    }

    /// Creates the shares compatibility router (non-generic version)
    pub fn build_concrete(self) -> Router {
        Self::build_base_router().with_state(UnifiedAppState {
            main_state: None,
            repositories: self.repositories,
            services: self.services,
        })
    }

    /// Creates a secure shares compatibility router with authentication middleware
    pub fn build_secure(self, main_state: MainAppState) -> Router {
        // Apply auth middleware first with the main state
        Self::build_secure_router()
            .layer(middleware::from_fn_with_state(
                main_state.clone(),
                auth_middleware,
            ))
            .with_state(UnifiedAppState {
                main_state: Some(main_state),
                repositories: self.repositories,
                services: self.services,
            })
    }
}

/// Creates the shares compatibility router for NestJS API compatibility
/// This router provides endpoints without the /api prefix for direct NestJS compatibility
pub fn compat_router<S>(repositories: Repositories, services: Services) -> Router<S> {
    SharesRouterBuilder::new(repositories, services).build()
}

/// Creates the shares compatibility router for NestJS API compatibility (concrete type)
pub fn router(repositories: Repositories, services: Services) -> Router {
    SharesRouterBuilder::new(repositories, services).build_concrete()
}

/// Creates a secure shares compatibility router with authentication middleware
pub fn secure_compat_router(
    main_state: MainAppState,
    repositories: Repositories,
    services: Services,
) -> Router {
    SharesRouterBuilder::new(repositories, services).build_secure(main_state)
}

/// Legacy app state for backward compatibility
#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    repositories: Repositories,
    #[allow(dead_code)]
    services: Services,
}

/// Legacy secure app state for backward compatibility
#[derive(Clone)]
pub struct SecureAppState {
    pub main_state: MainAppState,
    pub repositories: Repositories,
    pub services: Services,
}

/// Unified app state that can work with or without authentication
#[derive(Clone)]
pub struct UnifiedAppState {
    pub main_state: Option<MainAppState>,
    pub repositories: Repositories,
    pub services: Services,
}

impl UnifiedAppState {
    /// Create a new unified state without authentication
    pub fn new(repositories: Repositories, services: Services) -> Self {
        Self {
            main_state: None,
            repositories,
            services,
        }
    }

    /// Create a new unified state with authentication
    pub fn with_auth(
        main_state: MainAppState,
        repositories: Repositories,
        services: Services,
    ) -> Self {
        Self {
            main_state: Some(main_state),
            repositories,
            services,
        }
    }

    /// Convert to legacy AppState for compatibility
    pub fn to_app_state(&self) -> AppState {
        AppState {
            repositories: self.repositories.clone(),
            services: self.services.clone(),
        }
    }

    /// Convert to legacy SecureAppState for compatibility (requires main_state)
    pub fn to_secure_app_state(&self) -> Option<SecureAppState> {
        self.main_state.as_ref().map(|main_state| SecureAppState {
            main_state: main_state.clone(),
            repositories: self.repositories.clone(),
            services: self.services.clone(),
        })
    }
}

// DTO structs matching NestJS API

/// OfferSharesDto - matches NestJS create share offer request
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferSharesDto {
    pub name: String,
    pub description: Option<String>,
    pub price_per_share: String, // String for exact decimal compatibility
    pub total_shares_available: String,
    pub valid_from: Option<String>, // ISO string for compatibility
    pub valid_until: Option<String>,
    pub min_purchase_quantity: Option<String>,
    pub max_purchase_quantity: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_by: Option<String>, // UUID as string for compatibility
}

/// AllSharesOffers - matches NestJS get all offers response
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllSharesOffers {
    pub offers: Vec<ShareOfferCompat>,
    pub total: u64,
    pub active: u64,
}

/// ShareOfferCompat - compatible share offer representation
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShareOfferCompat {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub price_per_share: String,
    pub total_shares_available: String,
    pub shares_sold: String,
    pub shares_remaining: String,
    pub status: String,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub min_purchase_quantity: Option<String>,
    pub max_purchase_quantity: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
}

/// SubscribeSharesDto - matches NestJS subscribe to shares request
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeSharesDto {
    pub share_offer_id: String, // UUID as string
    pub owner_id: String,
    pub owner_type: String,           // "member" or "group"
    pub quantity: String,             // Decimal as string
    pub purchased_by: Option<String>, // UUID as string
}

/// UserShareTxsResponse - matches NestJS user transactions response
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserShareTxsResponse {
    pub transactions: Vec<ShareTransactionCompat>,
    pub pagination: PaginationMeta,
    pub summary: TransactionSummary,
}

/// ShareTransactionCompat - compatible share transaction representation
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShareTransactionCompat {
    pub id: String,
    pub shares_id: String, // Note: NestJS uses sharesId, we map from our share_id
    pub owner_id: String,
    pub owner_type: String,
    pub quantity: String,
    pub total_value: String,
    pub transaction_type: String, // "purchase", "transfer", "update"
    pub created_at: String,
    pub created_by: Option<String>,
}

/// PaginationMeta - NestJS-style pagination metadata
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    pub page: u64,
    pub size: u64,
    pub total: u64,
}

/// TransactionSummary - summary of user's share transactions
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSummary {
    pub total_shares: String,
    pub total_value: String,
}

/// PaginationQuery - query parameters for pagination
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub size: Option<u64>,
}

/// TransferSharesDto - matches NestJS transfer shares request
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferSharesDto {
    pub share_id: String,
    pub new_owner_id: String,
    pub new_owner_type: String,
    pub quantity_to_transfer: Option<String>,
    pub transferred_by: Option<String>,
    pub reason: Option<String>,
}

/// UpdateSharesDto - matches NestJS update shares request
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSharesDto {
    pub share_id: String,
    pub quantity: Option<String>,
    pub total_value: Option<String>,
    pub updated_by: Option<String>,
    pub reason: Option<String>,
}

// Helper functions for type conversions

fn parse_uuid(uuid_str: &str) -> Result<Uuid, String> {
    Uuid::parse_str(uuid_str).map_err(|_| format!("Invalid UUID format: {}", uuid_str))
}

fn parse_decimal(decimal_str: &str) -> Result<Decimal, String> {
    decimal_str
        .parse::<Decimal>()
        .map_err(|_| format!("Invalid decimal format: {}", decimal_str))
}

fn parse_owner_type(owner_type_str: &str) -> Result<OwnerType, String> {
    match owner_type_str.to_lowercase().as_str() {
        "member" => Ok(OwnerType::Member),
        "group" => Ok(OwnerType::Group),
        _ => Err(format!(
            "Invalid owner type: {}. Must be 'member' or 'group'",
            owner_type_str
        )),
    }
}

fn format_datetime(dt: chrono::DateTime<chrono::Utc>) -> String {
    dt.to_rfc3339()
}

// Endpoint implementations

/// POST /shares/offer - Create a new share offer (NestJS compatibility)
pub async fn create_share_offer(
    State(state): State<UnifiedAppState>,
    Json(request): Json<OfferSharesDto>,
) -> impl IntoResponse {
    // Parse and validate input
    let price_per_share = match parse_decimal(&request.price_per_share) {
        Ok(price) => price,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_price",
                    "message": msg
                })),
            )
                .into_response()
        }
    };

    let total_shares = match parse_decimal(&request.total_shares_available) {
        Ok(total) => total,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_total_shares",
                    "message": msg
                })),
            )
                .into_response()
        }
    };

    let created_by = if let Some(created_by_str) = &request.created_by {
        match parse_uuid(created_by_str) {
            Ok(uuid) => Some(uuid),
            Err(msg) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_created_by",
                        "message": msg
                    })),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    // Parse optional decimal fields
    let min_quantity = if let Some(min_str) = &request.min_purchase_quantity {
        match parse_decimal(min_str) {
            Ok(min) => Some(min),
            Err(msg) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_min_quantity",
                        "message": msg
                    })),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    let max_quantity = if let Some(max_str) = &request.max_purchase_quantity {
        match parse_decimal(max_str) {
            Ok(max) => Some(max),
            Err(msg) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_max_quantity",
                        "message": msg
                    })),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    // Parse datetime fields
    let valid_from = if let Some(from_str) = &request.valid_from {
        match chrono::DateTime::parse_from_rfc3339(from_str) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_valid_from",
                        "message": "Date must be in ISO 8601 format"
                    })),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    let valid_until = if let Some(until_str) = &request.valid_until {
        match chrono::DateTime::parse_from_rfc3339(until_str) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_valid_until",
                        "message": "Date must be in ISO 8601 format"
                    })),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    // Create the offer using existing share_offers repository
    use ::entity::share_offers::{self, ShareOfferStatus};
    use sea_orm::*;

    let offer = share_offers::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(request.name),
        description: Set(request.description),
        price_per_share: Set(price_per_share),
        total_shares_available: Set(total_shares),
        shares_sold: Set(Decimal::ZERO),
        shares_remaining: Set(total_shares),
        status: Set(ShareOfferStatus::Draft),
        valid_from: Set(valid_from.map(|dt| dt.into())),
        valid_until: Set(valid_until.map(|dt| dt.into())),
        min_purchase_quantity: Set(min_quantity),
        max_purchase_quantity: Set(max_quantity),
        settings: Set(request.settings),
        metadata: Set(request.metadata),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        created_by: Set(created_by),
        updated_by: Set(created_by),
    };

    match state.repositories.share_offers.create(offer).await {
        Ok(created_offer) => {
            // Convert to compatible format
            let compat_offer = ShareOfferCompat {
                id: created_offer.id.to_string(),
                name: created_offer.name,
                description: created_offer.description,
                price_per_share: created_offer.price_per_share.to_string(),
                total_shares_available: created_offer.total_shares_available.to_string(),
                shares_sold: created_offer.shares_sold.to_string(),
                shares_remaining: created_offer.shares_remaining.to_string(),
                status: format!("{:?}", created_offer.status).to_lowercase(),
                valid_from: created_offer
                    .valid_from
                    .map(|dt| format_datetime(dt.into())),
                valid_until: created_offer
                    .valid_until
                    .map(|dt| format_datetime(dt.into())),
                min_purchase_quantity: created_offer.min_purchase_quantity.map(|d| d.to_string()),
                max_purchase_quantity: created_offer.max_purchase_quantity.map(|d| d.to_string()),
                settings: created_offer.settings,
                metadata: created_offer.metadata,
                created_at: format_datetime(created_offer.created_at.into()),
                updated_at: format_datetime(created_offer.updated_at.into()),
                created_by: created_offer.created_by.map(|id| id.to_string()),
                updated_by: created_offer.updated_by.map(|id| id.to_string()),
            };

            (StatusCode::CREATED, Json(compat_offer)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create share offer: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "creation_failed",
                    "message": "Failed to create share offer"
                })),
            )
                .into_response()
        }
    }
}

/// GET /shares/offers - Get all share offers (NestJS compatibility)
pub async fn get_all_share_offers(State(state): State<UnifiedAppState>) -> impl IntoResponse {
    match state.repositories.share_offers.find_all().await {
        Ok(offers) => {
            // Convert all offers to compatible format
            let compat_offers: Vec<ShareOfferCompat> = offers
                .into_iter()
                .map(|offer| ShareOfferCompat {
                    id: offer.id.to_string(),
                    name: offer.name,
                    description: offer.description,
                    price_per_share: offer.price_per_share.to_string(),
                    total_shares_available: offer.total_shares_available.to_string(),
                    shares_sold: offer.shares_sold.to_string(),
                    shares_remaining: offer.shares_remaining.to_string(),
                    status: format!("{:?}", offer.status).to_lowercase(),
                    valid_from: offer.valid_from.map(|dt| format_datetime(dt.into())),
                    valid_until: offer.valid_until.map(|dt| format_datetime(dt.into())),
                    min_purchase_quantity: offer.min_purchase_quantity.map(|d| d.to_string()),
                    max_purchase_quantity: offer.max_purchase_quantity.map(|d| d.to_string()),
                    settings: offer.settings,
                    metadata: offer.metadata,
                    created_at: format_datetime(offer.created_at.into()),
                    updated_at: format_datetime(offer.updated_at.into()),
                    created_by: offer.created_by.map(|id| id.to_string()),
                    updated_by: offer.updated_by.map(|id| id.to_string()),
                })
                .collect();

            // Count active offers
            let active_count = compat_offers
                .iter()
                .filter(|offer| offer.status == "active")
                .count() as u64;

            let response = AllSharesOffers {
                offers: compat_offers.clone(),
                total: compat_offers.len() as u64,
                active: active_count,
            };

            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get share offers: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "fetch_failed",
                    "message": "Failed to fetch share offers"
                })),
            )
                .into_response()
        }
    }
}

/// POST /shares/subscribe - Subscribe to shares (maps to purchase) (NestJS compatibility)
pub async fn subscribe_to_shares(
    State(state): State<UnifiedAppState>,
    Json(request): Json<SubscribeSharesDto>,
) -> impl IntoResponse {
    // Parse and validate input
    let share_offer_id = match parse_uuid(&request.share_offer_id) {
        Ok(id) => id,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_share_offer_id",
                    "message": msg
                })),
            )
                .into_response()
        }
    };

    let owner_id = match parse_uuid(&request.owner_id) {
        Ok(id) => id,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_owner_id",
                    "message": msg
                })),
            )
                .into_response()
        }
    };

    let owner_type = match parse_owner_type(&request.owner_type) {
        Ok(ot) => ot,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_owner_type",
                    "message": msg
                })),
            )
                .into_response()
        }
    };

    let quantity = match parse_decimal(&request.quantity) {
        Ok(qty) => qty,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_quantity",
                    "message": msg
                })),
            )
                .into_response()
        }
    };

    let purchased_by = if let Some(purchased_by_str) = &request.purchased_by {
        match parse_uuid(purchased_by_str) {
            Ok(id) => Some(id),
            Err(msg) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_purchased_by",
                        "message": msg
                    })),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    // Map to our internal purchase request
    let purchase_request = SharePurchaseRequest {
        share_offer_id,
        owner_id,
        owner_type,
        quantity,
        purchased_by,
    };

    // Execute the purchase
    match state
        .services
        .share_purchase
        .purchase_shares(purchase_request)
        .await
    {
        Ok(result) => {
            // Convert result to NestJS compatible format
            let transaction = ShareTransactionCompat {
                id: result.share_record.id.to_string(),
                shares_id: result.share_record.id.to_string(), // NestJS uses sharesId
                owner_id: result.share_record.owner_id.to_string(),
                owner_type: format!("{:?}", result.share_record.owner_type).to_lowercase(),
                quantity: result.share_record.share_quantity.to_string(),
                total_value: result.share_record.total_value.to_string(),
                transaction_type: "purchase".to_string(),
                created_at: format_datetime(result.share_record.created_at.into()),
                created_by: result.share_record.created_by.map(|id| id.to_string()),
            };

            // Create response in NestJS format
            let response = UserShareTxsResponse {
                transactions: vec![transaction],
                pagination: PaginationMeta {
                    page: 0,
                    size: 1,
                    total: 1,
                },
                summary: TransactionSummary {
                    total_shares: result.share_record.share_quantity.to_string(),
                    total_value: result.share_record.total_value.to_string(),
                },
            };

            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to subscribe to shares: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "subscription_failed",
                    "message": format!("Failed to subscribe to shares: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// POST /shares/transfer - Transfer shares (NestJS compatibility)
pub async fn transfer_shares(
    State(state): State<UnifiedAppState>,
    Json(request): Json<TransferSharesDto>,
) -> impl IntoResponse {
    // Parse and validate input
    let share_id = match parse_uuid(&request.share_id) {
        Ok(id) => id,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_share_id",
                    "message": msg
                })),
            )
                .into_response()
        }
    };

    let new_owner_id = match parse_uuid(&request.new_owner_id) {
        Ok(id) => id,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_new_owner_id",
                    "message": msg
                })),
            )
                .into_response()
        }
    };

    let new_owner_type = match parse_owner_type(&request.new_owner_type) {
        Ok(ot) => ot,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_new_owner_type",
                    "message": msg
                })),
            )
                .into_response()
        }
    };

    let quantity_to_transfer = if let Some(qty_str) = &request.quantity_to_transfer {
        match parse_decimal(qty_str) {
            Ok(qty) => Some(qty),
            Err(msg) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_quantity_to_transfer",
                        "message": msg
                    })),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    let transferred_by = if let Some(transferred_by_str) = &request.transferred_by {
        match parse_uuid(transferred_by_str) {
            Ok(id) => Some(id),
            Err(msg) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_transferred_by",
                        "message": msg
                    })),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    // Map to our internal transfer request
    let transfer_request = ShareTransferRequest {
        share_id,
        new_owner_id,
        new_owner_type,
        quantity_to_transfer,
        transferred_by,
        reason: request.reason,
    };

    // Execute the transfer
    match state.services.share_purchase.transfer_shares(transfer_request).await {
        Ok(result) => {
            Json(serde_json::json!({
                "message": "Shares transferred successfully",
                "originalShare": {
                    "id": result.original_share.as_ref().map(|s| s.id.to_string()).unwrap_or_default(),
                    "quantity": result.original_share.as_ref().map(|s| s.share_quantity.to_string()).unwrap_or_default(),
                    "ownerId": result.original_share.as_ref().map(|s| s.owner_id.to_string()).unwrap_or_default(),
                    "ownerType": result.original_share.as_ref().map(|s| format!("{:?}", s.owner_type).to_lowercase()).unwrap_or_default()
                },
                "newShare": serde_json::json!({
                    "id": result.new_share.id.to_string(),
                    "quantity": result.new_share.share_quantity.to_string(),
                    "ownerId": result.new_share.owner_id.to_string(),
                    "ownerType": format!("{:?}", result.new_share.owner_type).to_lowercase()
                }),
                "quantityTransferred": result.quantity_transferred.to_string()
            })).into_response()
        },
        Err(e) => {
            tracing::error!("Failed to transfer shares: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "transfer_failed",
                    "message": format!("Failed to transfer shares: {}", e)
                })),
            ).into_response()
        }
    }
}

/// POST /shares/update - Update shares (NestJS compatibility)
pub async fn update_shares(
    State(state): State<UnifiedAppState>,
    Json(request): Json<UpdateSharesDto>,
) -> impl IntoResponse {
    // Parse and validate input
    let share_id = match parse_uuid(&request.share_id) {
        Ok(id) => id,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_share_id",
                    "message": msg
                })),
            )
                .into_response()
        }
    };

    // For now, return a basic implementation. In a real scenario,
    // we'd need to implement a shares update service method
    match state.repositories.shares.find_by_id(share_id).await {
        Ok(Some(share)) => Json(serde_json::json!({
            "message": "Share update functionality not fully implemented",
            "shareId": share.id.to_string(),
            "currentQuantity": share.share_quantity.to_string(),
            "currentValue": share.total_value.to_string(),
            "status": "pending_implementation"
        }))
        .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "share_not_found",
                "message": "Share not found"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to find share: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "update_failed",
                    "message": "Failed to update share"
                })),
            )
                .into_response()
        }
    }
}

/// GET /shares/transactions - Get all share transactions (NestJS compatibility)
pub async fn get_all_transactions(
    State(state): State<UnifiedAppState>,
    Query(query): Query<PaginationQuery>,
) -> impl IntoResponse {
    // Get all shares as transactions (our current model doesn't have separate transactions)
    match state.repositories.shares.find_all().await {
        Ok(shares) => {
            let transactions: Vec<ShareTransactionCompat> = shares
                .into_iter()
                .map(|share| {
                    ShareTransactionCompat {
                        id: share.id.to_string(),
                        shares_id: share.id.to_string(),
                        owner_id: share.owner_id.to_string(),
                        owner_type: format!("{:?}", share.owner_type).to_lowercase(),
                        quantity: share.share_quantity.to_string(),
                        total_value: share.total_value.to_string(),
                        transaction_type: "purchase".to_string(), // Default to purchase
                        created_at: format_datetime(share.created_at.into()),
                        created_by: share.created_by.map(|id| id.to_string()),
                    }
                })
                .collect();

            // Apply pagination
            let page = query.page.unwrap_or(0);
            let size = query.size.unwrap_or(100);
            let start = (page * size) as usize;
            let end = std::cmp::min(start + size as usize, transactions.len());

            let paginated_transactions = if start < transactions.len() {
                transactions[start..end].to_vec()
            } else {
                vec![]
            };

            let response = UserShareTxsResponse {
                transactions: paginated_transactions,
                pagination: PaginationMeta {
                    page,
                    size,
                    total: transactions.len() as u64,
                },
                summary: TransactionSummary {
                    total_shares: transactions
                        .iter()
                        .map(|t| parse_decimal(&t.quantity).unwrap_or(Decimal::ZERO))
                        .sum::<Decimal>()
                        .to_string(),
                    total_value: transactions
                        .iter()
                        .map(|t| parse_decimal(&t.total_value).unwrap_or(Decimal::ZERO))
                        .sum::<Decimal>()
                        .to_string(),
                },
            };

            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get all transactions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "fetch_failed",
                    "message": "Failed to fetch transactions"
                })),
            )
                .into_response()
        }
    }
}

/// GET /shares/transactions/:userId - Get user's share transactions with pagination (NestJS compatibility)
pub async fn get_user_transactions(
    State(state): State<UnifiedAppState>,
    Path(user_id_str): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> impl IntoResponse {
    let user_id = match parse_uuid(&user_id_str) {
        Ok(id) => id,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_user_id",
                    "message": msg
                })),
            )
                .into_response()
        }
    };

    // Get shares for the user (both as member and potentially as group member)
    match state
        .repositories
        .shares
        .find_by_owner(user_id, OwnerType::Member)
        .await
    {
        Ok(shares) => {
            let transactions: Vec<ShareTransactionCompat> = shares
                .into_iter()
                .map(|share| ShareTransactionCompat {
                    id: share.id.to_string(),
                    shares_id: share.id.to_string(),
                    owner_id: share.owner_id.to_string(),
                    owner_type: format!("{:?}", share.owner_type).to_lowercase(),
                    quantity: share.share_quantity.to_string(),
                    total_value: share.total_value.to_string(),
                    transaction_type: "purchase".to_string(),
                    created_at: format_datetime(share.created_at.into()),
                    created_by: share.created_by.map(|id| id.to_string()),
                })
                .collect();

            // Apply pagination
            let page = query.page.unwrap_or(0);
            let size = query.size.unwrap_or(100);
            let start = (page * size) as usize;
            let end = std::cmp::min(start + size as usize, transactions.len());

            let paginated_transactions = if start < transactions.len() {
                transactions[start..end].to_vec()
            } else {
                vec![]
            };

            let response = UserShareTxsResponse {
                transactions: paginated_transactions,
                pagination: PaginationMeta {
                    page,
                    size,
                    total: transactions.len() as u64,
                },
                summary: TransactionSummary {
                    total_shares: transactions
                        .iter()
                        .map(|t| parse_decimal(&t.quantity).unwrap_or(Decimal::ZERO))
                        .sum::<Decimal>()
                        .to_string(),
                    total_value: transactions
                        .iter()
                        .map(|t| parse_decimal(&t.total_value).unwrap_or(Decimal::ZERO))
                        .sum::<Decimal>()
                        .to_string(),
                },
            };

            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get user transactions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "fetch_failed",
                    "message": "Failed to fetch user transactions"
                })),
            )
                .into_response()
        }
    }
}

/// GET /shares/transactions/find/:sharesId - Find specific transaction by shares ID (NestJS compatibility)
pub async fn find_transaction_by_shares_id(
    State(state): State<UnifiedAppState>,
    Path(shares_id_str): Path<String>,
) -> impl IntoResponse {
    let shares_id = match parse_uuid(&shares_id_str) {
        Ok(id) => id,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_shares_id",
                    "message": msg
                })),
            )
                .into_response()
        }
    };

    match state.repositories.shares.find_by_id(shares_id).await {
        Ok(Some(share)) => {
            let transaction = ShareTransactionCompat {
                id: share.id.to_string(),
                shares_id: share.id.to_string(),
                owner_id: share.owner_id.to_string(),
                owner_type: format!("{:?}", share.owner_type).to_lowercase(),
                quantity: share.share_quantity.to_string(),
                total_value: share.total_value.to_string(),
                transaction_type: "purchase".to_string(),
                created_at: format_datetime(share.created_at.into()),
                created_by: share.created_by.map(|id| id.to_string()),
            };

            Json(transaction).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "transaction_not_found",
                "message": "Transaction not found"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to find transaction: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "fetch_failed",
                    "message": "Failed to find transaction"
                })),
            )
                .into_response()
        }
    }
}

// Secure endpoint implementations with authentication

/// POST /shares/offer - Create a new share offer (Secure version with authentication)
pub async fn secure_create_share_offer(
    State(state): State<UnifiedAppState>,
    Extension(user_context): Extension<UserContext>,
    Json(offer_request): Json<OfferSharesDto>,
) -> impl IntoResponse {
    // Set the created_by field to the authenticated user
    let mut secure_request = offer_request;
    secure_request.created_by = Some(user_context.user_id.to_string());

    // Call the original implementation with the modified request
    create_share_offer(State(state.clone()), Json(secure_request))
        .await
        .into_response()
}

/// GET /shares/offers - Get all share offers (Secure version with authentication)  
pub async fn secure_get_all_share_offers(
    State(state): State<UnifiedAppState>,
    Extension(_user_context): Extension<UserContext>,
) -> impl IntoResponse {
    // Call the original implementation
    get_all_share_offers(State(state.clone()))
        .await
        .into_response()
}

/// POST /shares/subscribe - Subscribe to shares (Secure version with authentication)
pub async fn secure_subscribe_to_shares(
    State(state): State<UnifiedAppState>,
    Extension(user_context): Extension<UserContext>,
    Json(subscribe_request): Json<SubscribeSharesDto>,
) -> impl IntoResponse {
    // Validate that the user can only subscribe for themselves (if owner_type is member)
    if subscribe_request.owner_type.to_lowercase() == "member" {
        let owner_id = match Uuid::parse_str(&subscribe_request.owner_id) {
            Ok(id) => id,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_owner_id",
                        "message": "Invalid owner ID format"
                    })),
                )
                    .into_response()
            }
        };

        if owner_id != user_context.user_id {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "unauthorized",
                    "message": "Users can only subscribe to shares for themselves"
                })),
            )
                .into_response();
        }
    }

    // Set the purchased_by field to the authenticated user
    let mut secure_request = subscribe_request;
    secure_request.purchased_by = Some(user_context.user_id.to_string());

    // Call the original implementation
    subscribe_to_shares(State(state.clone()), Json(secure_request))
        .await
        .into_response()
}

/// POST /shares/transfer - Transfer shares (Secure version with authentication)
pub async fn secure_transfer_shares(
    State(state): State<UnifiedAppState>,
    Extension(user_context): Extension<UserContext>,
    Json(transfer_request): Json<TransferSharesDto>,
) -> impl IntoResponse {
    // Validate ownership of the share being transferred
    let share_id = match Uuid::parse_str(&transfer_request.share_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_share_id",
                    "message": "Invalid share ID format"
                })),
            )
                .into_response()
        }
    };

    // Check if the user owns the share they're trying to transfer
    match state.repositories.shares.find_by_id(share_id).await {
        Ok(Some(share)) => {
            if share.owner_id != user_context.user_id {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({
                        "error": "unauthorized",
                        "message": "You can only transfer shares you own"
                    })),
                )
                    .into_response();
            }
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "share_not_found",
                    "message": "Share not found"
                })),
            )
                .into_response()
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "database_error",
                    "message": "Failed to verify share ownership"
                })),
            )
                .into_response()
        }
    }

    // Set the transferred_by field to the authenticated user
    let mut secure_request = transfer_request;
    secure_request.transferred_by = Some(user_context.user_id.to_string());

    // Call the original implementation
    transfer_shares(State(state.clone()), Json(secure_request))
        .await
        .into_response()
}

// Placeholder implementations for remaining secure handlers
pub async fn secure_update_shares(
    State(state): State<UnifiedAppState>,
    Extension(_user_context): Extension<UserContext>,
    Json(update_request): Json<UpdateSharesDto>,
) -> impl IntoResponse {
    update_shares(State(state.clone()), Json(update_request))
        .await
        .into_response()
}

pub async fn secure_get_all_transactions(
    State(state): State<UnifiedAppState>,
    Extension(user_context): Extension<UserContext>,
    Query(query): Query<PaginationQuery>,
) -> impl IntoResponse {
    if !user_context.roles.contains(&"admin".to_string()) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "unauthorized",
                "message": "Only administrators can view all transactions"
            })),
        )
            .into_response();
    }

    get_all_transactions(State(state.clone()), Query(query))
        .await
        .into_response()
}

pub async fn secure_get_user_transactions(
    State(state): State<UnifiedAppState>,
    Extension(user_context): Extension<UserContext>,
    Path(user_id_str): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> impl IntoResponse {
    let requested_user_id = match Uuid::parse_str(&user_id_str) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_user_id",
                    "message": "Invalid user ID format"
                })),
            )
                .into_response()
        }
    };

    if requested_user_id != user_context.user_id
        && !user_context.roles.contains(&"admin".to_string())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "unauthorized",
                "message": "You can only view your own transactions"
            })),
        )
            .into_response();
    }

    get_user_transactions(State(state.clone()), Path(user_id_str), Query(query))
        .await
        .into_response()
}

pub async fn secure_find_transaction_by_shares_id(
    State(state): State<UnifiedAppState>,
    Extension(_user_context): Extension<UserContext>,
    Path(shares_id_str): Path<String>,
) -> impl IntoResponse {
    find_transaction_by_shares_id(State(state.clone()), Path(shares_id_str))
        .await
        .into_response()
}
