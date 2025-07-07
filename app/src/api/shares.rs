use ::entity::shares::OwnerType;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    repositories::Repositories,
    services::{
        share_purchase::{SharePurchaseRequest, ShareTransferRequest},
        Services,
    },
};

pub fn router(repositories: Repositories, services: Services) -> Router {
    Router::new()
        .route("/purchase", post(purchase_shares))
        .route("/transfer", post(transfer_shares))
        .route("/owner/{owner_id}/{owner_type}", get(get_shares_by_owner))
        .route(
            "/summary/{owner_id}/{owner_type}",
            get(get_ownership_summary),
        )
        .route("/", get(list_shares))
        .route("/{id}", get(get_share))
        .with_state(AppState {
            repositories,
            services,
        })
}

#[derive(Clone)]
pub struct AppState {
    repositories: Repositories,
    services: Services,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseSharesRequest {
    pub share_offer_id: Uuid,
    pub owner_id: Uuid,
    pub owner_type: OwnerType,
    pub quantity: Decimal,
    pub purchased_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferSharesRequest {
    pub share_id: Uuid,
    pub new_owner_id: Uuid,
    pub new_owner_type: OwnerType,
    pub quantity_to_transfer: Option<Decimal>,
    pub transferred_by: Option<Uuid>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListSharesQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub owner_id: Option<Uuid>,
    pub owner_type: Option<String>,
    pub share_offer_id: Option<Uuid>,
}

pub async fn purchase_shares(
    State(state): State<AppState>,
    Json(request): Json<PurchaseSharesRequest>,
) -> impl IntoResponse {
    let purchase_request = SharePurchaseRequest {
        share_offer_id: request.share_offer_id,
        owner_id: request.owner_id,
        owner_type: request.owner_type,
        quantity: request.quantity,
        purchased_by: request.purchased_by,
    };

    match state
        .services
        .share_purchase
        .purchase_shares(purchase_request)
        .await
    {
        Ok(result) => Json(serde_json::json!({
            "message": "Shares purchased successfully",
            "share_record": result.share_record,
            "updated_offer": result.updated_offer,
            "transaction_summary": result.transaction_summary
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to purchase shares: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Failed to purchase shares",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn transfer_shares(
    State(state): State<AppState>,
    Json(request): Json<TransferSharesRequest>,
) -> impl IntoResponse {
    let transfer_request = ShareTransferRequest {
        share_id: request.share_id,
        new_owner_id: request.new_owner_id,
        new_owner_type: request.new_owner_type,
        quantity_to_transfer: request.quantity_to_transfer,
        transferred_by: request.transferred_by,
        reason: request.reason,
    };

    match state
        .services
        .share_purchase
        .transfer_shares(transfer_request)
        .await
    {
        Ok(result) => Json(serde_json::json!({
            "message": "Shares transferred successfully",
            "original_share": result.original_share,
            "new_share": result.new_share,
            "quantity_transferred": result.quantity_transferred
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to transfer shares: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Failed to transfer shares",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn get_shares_by_owner(
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
        .repositories
        .shares
        .find_by_owner(owner_id, owner_type)
        .await
    {
        Ok(shares) => Json(serde_json::json!({
            "shares": shares,
            "total": shares.len(),
            "owner_id": owner_id,
            "owner_type": owner_type_str
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to get shares by owner: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get shares by owner",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn get_ownership_summary(
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
        .share_purchase
        .get_ownership_summary(owner_id, owner_type)
        .await
    {
        Ok(summary) => Json(serde_json::json!({
            "summary": summary,
            "owner_id": owner_id,
            "owner_type": owner_type_str
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to get ownership summary: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get ownership summary",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn list_shares(
    State(state): State<AppState>,
    Query(query): Query<ListSharesQuery>,
) -> impl IntoResponse {
    let mut shares = match state.repositories.shares.find_all().await {
        Ok(shares) => shares,
        Err(e) => {
            tracing::error!("Failed to list shares: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to list shares",
                    "details": e.to_string()
                })),
            )
                .into_response();
        }
    };

    // Apply filters
    if let Some(owner_id) = query.owner_id {
        shares.retain(|s| s.owner_id == owner_id);
    }

    if let Some(owner_type_str) = query.owner_type {
        let owner_type = match owner_type_str.as_str() {
            "member" => Some(OwnerType::Member),
            "group" => Some(OwnerType::Group),
            _ => None,
        };
        if let Some(ot) = owner_type {
            shares.retain(|s| s.owner_type == ot);
        }
    }

    if let Some(share_offer_id) = query.share_offer_id {
        shares.retain(|s| s.share_offer_id == share_offer_id);
    }

    // Apply pagination
    let paginated_shares = if let (Some(limit), Some(offset)) = (query.limit, query.offset) {
        shares
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect::<Vec<_>>()
    } else {
        shares
    };

    Json(serde_json::json!({
        "shares": paginated_shares,
        "total": paginated_shares.len()
    }))
    .into_response()
}

pub async fn get_share(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.repositories.shares.find_by_id(id).await {
        Ok(Some(share)) => Json(share).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Share not found"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get share: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get share",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}
