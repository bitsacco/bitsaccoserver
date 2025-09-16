use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::services::lnurl::lightning_address::{
    LightningAddressError, LnurlPayCallbackRequest, LnurlPayCallbackResponse,
};
use crate::{repositories::Repositories, services::Services};

/// LNURL error response format
#[derive(Debug, Serialize)]
struct LnurlErrorResponse {
    status: String,
    reason: String,
}

impl LnurlErrorResponse {
    fn new(reason: String) -> Self {
        Self {
            status: "ERROR".to_string(),
            reason,
        }
    }
}

/// Convert LightningAddressError to HTTP response
impl IntoResponse for LightningAddressError {
    fn into_response(self) -> Response {
        let (status, reason) = match self {
            LightningAddressError::AddressNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "Lightning address not found".to_string(),
            ),
            LightningAddressError::AddressInactive => (
                StatusCode::NOT_FOUND,
                "Lightning address is inactive".to_string(),
            ),
            LightningAddressError::InvalidUsername(reason) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid username: {}", reason),
            ),
            LightningAddressError::InvalidAmount { reason } => (
                StatusCode::BAD_REQUEST,
                format!("Invalid amount: {}", reason),
            ),
            LightningAddressError::AmountOutOfRange { amount, min, max } => (
                StatusCode::BAD_REQUEST,
                format!("Amount {} msat out of range [{}, {}]", amount, min, max),
            ),
            LightningAddressError::UsernameUnavailable { .. } => {
                (StatusCode::CONFLICT, "Username not available".to_string())
            }
            LightningAddressError::Validation(reason) => (StatusCode::BAD_REQUEST, reason),
            LightningAddressError::Configuration(reason) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Configuration error: {}", reason),
            ),
            LightningAddressError::Repository(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let error_response = LnurlErrorResponse::new(reason);
        (status, Json(error_response)).into_response()
    }
}

/// GET /.well-known/lnurlp/{username}
///
/// This endpoint implements the LNURL-pay discovery mechanism.
/// When someone wants to pay to alice@domain.com, their wallet:
/// 1. Makes a GET request to https://domain.com/.well-known/lnurlp/alice
/// 2. Gets back the LNURL-pay response with callback URL and limits
/// 3. Uses that information to request an invoice from the callback
pub async fn well_known_lnurlp(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, LightningAddressError> {
    // Add CORS headers for .well-known endpoint
    let mut headers = HeaderMap::new();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "GET".parse().unwrap());
    headers.insert(
        "Access-Control-Allow-Headers",
        "Content-Type".parse().unwrap(),
    );
    headers.insert("Content-Type", "application/json".parse().unwrap());

    // Resolve the lightning address to LNURL-pay response
    let lnurl_response = state
        .services
        .lightning_address
        .resolve_address(&username, None)
        .await?;

    Ok((headers, Json(lnurl_response)))
}

/// GET /api/lnurl/pay/callback/{username}?amount={amount}&comment={comment}
///
/// This is the LNURL-pay callback endpoint that generates invoices.
/// Called by wallets after they get the initial LNURL-pay response.
pub async fn lnurl_pay_callback(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(params): Query<LnurlPayCallbackRequest>,
) -> Result<Json<LnurlPayCallbackResponse>, LightningAddressError> {
    // Get the lightning address
    let address = state
        .services
        .lightning_address
        .get_address_by_username(&username, None)
        .await?;

    if !address.is_active {
        return Err(LightningAddressError::AddressInactive);
    }

    // Validate amount is within limits
    if params.amount < address.min_sendable_msat || params.amount > address.max_sendable_msat {
        return Err(LightningAddressError::AmountOutOfRange {
            amount: params.amount,
            min: address.min_sendable_msat,
            max: address.max_sendable_msat,
        });
    }

    // TODO: Validate comment length if provided
    if let Some(ref comment) = params.comment {
        if comment.len() > 280 {
            // TODO: Use config value
            return Err(LightningAddressError::Validation(
                "Comment too long".to_string(),
            ));
        }
    }

    // TODO: Generate Lightning invoice using wallet service
    // For now, return a placeholder response
    let mock_invoice = "lnbc1500n1pn9n8jkpp5..."; // This would be a real invoice

    // Mark address as used
    state
        .services
        .lightning_address
        .mark_address_used(address.id)
        .await?;

    // TODO: Create LNURL transaction record
    // This would track the payment request for later verification

    let response = LnurlPayCallbackResponse {
        payment_request: mock_invoice.to_string(),
        routes: None,
        success_action: Some(json!({
            "tag": "message",
            "message": format!("Payment sent to {}@{}", username, address.domain)
        })),
    };

    Ok(Json(response))
}

/// GET /api/lnurl/addresses/{username}/availability
///
/// Check if a username is available for registration
pub async fn check_username_availability(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<serde_json::Value>, LightningAddressError> {
    let available = state
        .services
        .lightning_address
        .check_availability(&username, None)
        .await?;

    Ok(Json(json!({
        "username": username,
        "available": available
    })))
}

/// Lightning address management endpoints (for authenticated users)

#[derive(Debug, Deserialize)]
pub struct CreateAddressApiRequest {
    pub username: String,
    pub wallet_id: String, // UUID as string
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub min_sendable_sats: Option<u64>, // Convert to msat
    pub max_sendable_sats: Option<u64>, // Convert to msat
}

#[derive(Debug, Serialize)]
pub struct AddressResponse {
    pub id: String,
    pub username: String,
    pub domain: String,
    pub full_address: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub min_sendable_sats: u64,
    pub max_sendable_sats: u64,
    pub is_active: bool,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

impl From<::entity::lightning_addresses::Model> for AddressResponse {
    fn from(model: ::entity::lightning_addresses::Model) -> Self {
        Self {
            id: model.id.to_string(),
            username: model.username.clone(),
            domain: model.domain.clone(),
            full_address: format!("{}@{}", model.username, model.domain),
            display_name: model.display_name,
            avatar: model.avatar,
            description: model.description,
            min_sendable_sats: (model.min_sendable_msat / 1000) as u64,
            max_sendable_sats: (model.max_sendable_msat / 1000) as u64,
            is_active: model.is_active,
            created_at: model.created_at.to_rfc3339(),
            last_used_at: model.last_used_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

/// POST /api/lnurl/addresses
///
/// Create a new lightning address (authenticated endpoint)
pub async fn create_lightning_address(
    State(state): State<AppState>,
    Json(request): Json<CreateAddressApiRequest>,
) -> Result<Json<AddressResponse>, LightningAddressError> {
    use crate::services::lnurl::lightning_address::CreateAddressRequest;
    use uuid::Uuid;

    let wallet_id = Uuid::parse_str(&request.wallet_id)
        .map_err(|_| LightningAddressError::Validation("Invalid wallet ID".to_string()))?;

    let create_request = CreateAddressRequest {
        username: request.username,
        wallet_id,
        domain: None, // Use default domain
        display_name: request.display_name,
        avatar: request.avatar,
        description: request.description,
        min_sendable: request.min_sendable_sats.unwrap_or(1) as i64 * 1000, // Convert sats to msat
        max_sendable: request.max_sendable_sats.unwrap_or(100_000) as i64 * 1000, // Convert sats to msat
        metadata: None,
    };

    let address = state
        .services
        .lightning_address
        .create_address(create_request)
        .await?;

    Ok(Json(AddressResponse::from(address)))
}

/// GET /api/lnurl/addresses/wallet/{wallet_id}
///
/// Get all lightning addresses for a wallet (authenticated endpoint)
pub async fn get_wallet_addresses(
    State(state): State<AppState>,
    Path(wallet_id): Path<String>,
) -> Result<Json<Vec<AddressResponse>>, LightningAddressError> {
    use uuid::Uuid;

    let wallet_id = Uuid::parse_str(&wallet_id)
        .map_err(|_| LightningAddressError::Validation("Invalid wallet ID".to_string()))?;

    let addresses = state
        .services
        .lightning_address
        .get_wallet_addresses(wallet_id)
        .await?;

    let response: Vec<AddressResponse> = addresses.into_iter().map(AddressResponse::from).collect();

    Ok(Json(response))
}

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    repositories: Repositories,
    #[allow(dead_code)]
    services: Services,
}

/// Create LNURL API router
pub fn router(repositories: Repositories, services: Services) -> Router {
    Router::new()
        // .well-known endpoints (public, no auth required)
        .route("/.well-known/lnurlp/:username", get(well_known_lnurlp))
        // LNURL-pay callback (public, no auth required)
        .route("/pay/callback/:username", get(lnurl_pay_callback))
        // Username availability check (public)
        .route(
            "/addresses/:username/availability",
            get(check_username_availability),
        )
        // Lightning address management (authenticated endpoints - TODO: add auth middleware)
        .route("/addresses", post(create_lightning_address))
        .route("/addresses/wallet/:wallet_id", get(get_wallet_addresses))
        .with_state(AppState {
            repositories,
            services,
        })
}
