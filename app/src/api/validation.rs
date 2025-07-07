use ::entity::shares::OwnerType;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    repositories::Repositories,
    services::{
        validation::{SharePurchaseValidationRequest, ShareTransferValidationRequest},
        Services,
    },
};

pub fn router(repositories: Repositories, services: Services) -> Router {
    Router::new()
        .route("/purchase", post(validate_purchase))
        .route("/transfer", post(validate_transfer))
        .route(
            "/owner/{owner_id}/{owner_type}",
            get(get_owner_validation_info),
        )
        .route(
            "/offer/{offer_id}/{quantity}",
            get(get_offer_validation_info),
        )
        .with_state(AppState {
            repositories,
            services,
        })
}

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    repositories: Repositories,
    services: Services,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidatePurchaseRequest {
    pub offer_id: Uuid,
    pub owner_id: Uuid,
    pub owner_type: OwnerType,
    pub quantity: rust_decimal::Decimal,
    pub requested_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTransferRequest {
    pub from_owner_id: Uuid,
    pub from_owner_type: OwnerType,
    pub to_owner_id: Uuid,
    pub to_owner_type: OwnerType,
    pub share_offer_id: Uuid,
    pub quantity: rust_decimal::Decimal,
    pub requested_by: Option<Uuid>,
}

pub async fn validate_purchase(
    State(state): State<AppState>,
    Json(request): Json<ValidatePurchaseRequest>,
) -> impl IntoResponse {
    let validation_request = SharePurchaseValidationRequest {
        offer_id: request.offer_id,
        owner_id: request.owner_id,
        owner_type: request.owner_type,
        quantity: request.quantity,
        requested_by: request.requested_by,
    };

    match state
        .services
        .validation
        .validate_share_purchase(&validation_request)
        .await
    {
        Ok(result) => Json(serde_json::json!({
            "validation_result": result,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "request_summary": {
                "offer_id": request.offer_id,
                "owner_id": request.owner_id,
                "owner_type": request.owner_type,
                "quantity": request.quantity
            }
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to validate purchase: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to validate purchase",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn validate_transfer(
    State(state): State<AppState>,
    Json(request): Json<ValidateTransferRequest>,
) -> impl IntoResponse {
    let validation_request = ShareTransferValidationRequest {
        from_owner_id: request.from_owner_id,
        from_owner_type: request.from_owner_type,
        to_owner_id: request.to_owner_id,
        to_owner_type: request.to_owner_type,
        share_offer_id: request.share_offer_id,
        quantity: request.quantity,
        requested_by: request.requested_by,
    };

    match state
        .services
        .validation
        .validate_share_transfer(&validation_request)
        .await
    {
        Ok(result) => Json(serde_json::json!({
            "validation_result": result,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "request_summary": {
                "from_owner_id": request.from_owner_id,
                "from_owner_type": request.from_owner_type,
                "to_owner_id": request.to_owner_id,
                "to_owner_type": request.to_owner_type,
                "share_offer_id": request.share_offer_id,
                "quantity": request.quantity
            }
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to validate transfer: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to validate transfer",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn get_owner_validation_info(
    State(state): State<AppState>,
    Path((owner_id, owner_type_str)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    let owner_type = match owner_type_str.as_str() {
        "member" => OwnerType::Member,
        "group" => OwnerType::Group,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid owner type",
                    "valid_types": ["member", "group"]
                })),
            )
                .into_response()
        }
    };

    match state
        .services
        .validation
        .get_owner_validation_info(owner_id, owner_type)
        .await
    {
        Ok(info) => Json(serde_json::json!({
            "owner_validation_info": info,
            "owner_id": owner_id,
            "owner_type": owner_type_str,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to get owner validation info: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get owner validation info",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn get_offer_validation_info(
    State(state): State<AppState>,
    Path((offer_id, quantity)): Path<(Uuid, rust_decimal::Decimal)>,
) -> impl IntoResponse {
    match state
        .services
        .validation
        .get_offer_validation_info(offer_id, quantity)
        .await
    {
        Ok(info) => Json(serde_json::json!({
            "offer_validation_info": info,
            "offer_id": offer_id,
            "requested_quantity": quantity,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to get offer validation info: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get offer validation info",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}
