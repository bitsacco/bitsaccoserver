use ::entity::share_offers::{self, ShareOfferStatus};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{repositories::Repositories, services::Services};

pub fn router(repositories: Repositories, services: Services) -> Router {
    Router::new()
        .route("/", get(list_offers).post(create_offer))
        .route(
            "/{id}",
            get(get_offer).patch(update_offer).delete(delete_offer),
        )
        .route("/{id}/activate", post(activate_offer))
        .route("/{id}/pause", post(pause_offer))
        .route("/{id}/cancel", post(cancel_offer))
        .route("/active", get(list_active_offers))
        .route("/status/{status}", get(list_offers_by_status))
        .with_state(AppState {
            repositories,
            services,
        })
}

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    repositories: Repositories,
    #[allow(dead_code)]
    services: Services,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOfferRequest {
    pub name: String,
    pub description: Option<String>,
    pub price_per_share: rust_decimal::Decimal,
    pub total_shares_available: rust_decimal::Decimal,
    pub valid_from: Option<chrono::DateTime<chrono::Utc>>,
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
    pub min_purchase_quantity: Option<rust_decimal::Decimal>,
    pub max_purchase_quantity: Option<rust_decimal::Decimal>,
    pub settings: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOfferRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price_per_share: Option<rust_decimal::Decimal>,
    pub total_shares_available: Option<rust_decimal::Decimal>,
    pub valid_from: Option<chrono::DateTime<chrono::Utc>>,
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
    pub min_purchase_quantity: Option<rust_decimal::Decimal>,
    pub max_purchase_quantity: Option<rust_decimal::Decimal>,
    pub settings: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfferActionRequest {
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListOffersQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub status: Option<String>,
}

pub async fn create_offer(
    State(state): State<AppState>,
    Json(request): Json<CreateOfferRequest>,
) -> impl IntoResponse {
    let offer = share_offers::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(request.name),
        description: Set(request.description),
        price_per_share: Set(request.price_per_share),
        total_shares_available: Set(request.total_shares_available),
        shares_sold: Set(rust_decimal::Decimal::ZERO),
        shares_remaining: Set(request.total_shares_available),
        status: Set(ShareOfferStatus::Draft),
        valid_from: Set(request.valid_from.map(|dt| dt.into())),
        valid_until: Set(request.valid_until.map(|dt| dt.into())),
        min_purchase_quantity: Set(request.min_purchase_quantity),
        max_purchase_quantity: Set(request.max_purchase_quantity),
        settings: Set(request.settings.map(|v| v.into())),
        metadata: Set(request.metadata.map(|v| v.into())),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        created_by: Set(request.created_by),
        updated_by: Set(request.created_by),
    };

    match state.repositories.share_offers.create(offer).await {
        Ok(created_offer) => (StatusCode::CREATED, Json(created_offer)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create share offer: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to create share offer",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn get_offer(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.repositories.share_offers.find_by_id(id).await {
        Ok(Some(offer)) => Json(offer).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Share offer not found"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get share offer: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get share offer",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn list_offers(
    State(state): State<AppState>,
    Query(query): Query<ListOffersQuery>,
) -> impl IntoResponse {
    match state.repositories.share_offers.find_all().await {
        Ok(offers) => {
            let filtered_offers = if let Some(status_str) = query.status {
                let status_filter = match status_str.as_str() {
                    "draft" => Some(ShareOfferStatus::Draft),
                    "active" => Some(ShareOfferStatus::Active),
                    "paused" => Some(ShareOfferStatus::Paused),
                    "completed" => Some(ShareOfferStatus::Completed),
                    "expired" => Some(ShareOfferStatus::Expired),
                    "cancelled" => Some(ShareOfferStatus::Cancelled),
                    _ => None,
                };

                if let Some(status) = status_filter {
                    offers.into_iter().filter(|o| o.status == status).collect()
                } else {
                    offers
                }
            } else {
                offers
            };

            let paginated_offers = if let (Some(limit), Some(offset)) = (query.limit, query.offset)
            {
                filtered_offers
                    .into_iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .collect::<Vec<_>>()
            } else {
                filtered_offers
            };

            Json(serde_json::json!({
                "offers": paginated_offers,
                "total": paginated_offers.len()
            }))
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list share offers: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to list share offers",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn list_active_offers(State(state): State<AppState>) -> impl IntoResponse {
    match state.repositories.share_offers.find_active_offers().await {
        Ok(offers) => Json(serde_json::json!({
            "offers": offers,
            "total": offers.len()
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to list active share offers: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to list active share offers",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn list_offers_by_status(
    State(state): State<AppState>,
    Path(status_str): Path<String>,
) -> impl IntoResponse {
    let status = match status_str.as_str() {
        "draft" => ShareOfferStatus::Draft,
        "active" => ShareOfferStatus::Active,
        "paused" => ShareOfferStatus::Paused,
        "completed" => ShareOfferStatus::Completed,
        "expired" => ShareOfferStatus::Expired,
        "cancelled" => ShareOfferStatus::Cancelled,
        _ => return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid status",
                "valid_statuses": ["draft", "active", "paused", "completed", "expired", "cancelled"]
            })),
        )
            .into_response(),
    };

    match state.repositories.share_offers.find_by_status(status).await {
        Ok(offers) => Json(serde_json::json!({
            "offers": offers,
            "total": offers.len(),
            "status": status_str
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to list share offers by status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to list share offers by status",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn update_offer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateOfferRequest>,
) -> impl IntoResponse {
    let mut update_model = share_offers::ActiveModel {
        id: Set(id),
        updated_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };

    if let Some(name) = request.name {
        update_model.name = Set(name);
    }
    if let Some(description) = request.description {
        update_model.description = Set(Some(description));
    }
    if let Some(price) = request.price_per_share {
        update_model.price_per_share = Set(price);
    }
    if let Some(total) = request.total_shares_available {
        update_model.total_shares_available = Set(total);
    }
    if let Some(valid_from) = request.valid_from {
        update_model.valid_from = Set(Some(valid_from.into()));
    }
    if let Some(valid_until) = request.valid_until {
        update_model.valid_until = Set(Some(valid_until.into()));
    }
    if let Some(min_qty) = request.min_purchase_quantity {
        update_model.min_purchase_quantity = Set(Some(min_qty));
    }
    if let Some(max_qty) = request.max_purchase_quantity {
        update_model.max_purchase_quantity = Set(Some(max_qty));
    }
    if let Some(settings) = request.settings {
        update_model.settings = Set(Some(settings.into()));
    }
    if let Some(metadata) = request.metadata {
        update_model.metadata = Set(Some(metadata.into()));
    }
    if let Some(updated_by) = request.updated_by {
        update_model.updated_by = Set(Some(updated_by));
    }

    match state
        .repositories
        .share_offers
        .update(id, update_model)
        .await
    {
        Ok(updated_offer) => Json(updated_offer).into_response(),
        Err(e) => {
            tracing::error!("Failed to update share offer: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to update share offer",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn activate_offer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<OfferActionRequest>,
) -> impl IntoResponse {
    match state
        .repositories
        .share_offers
        .activate_offer(id, request.updated_by)
        .await
    {
        Ok(activated_offer) => Json(serde_json::json!({
            "message": "Share offer activated successfully",
            "offer": activated_offer
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to activate share offer: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Failed to activate share offer",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn pause_offer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<OfferActionRequest>,
) -> impl IntoResponse {
    match state
        .repositories
        .share_offers
        .pause_offer(id, request.updated_by)
        .await
    {
        Ok(paused_offer) => Json(serde_json::json!({
            "message": "Share offer paused successfully",
            "offer": paused_offer
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to pause share offer: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Failed to pause share offer",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn cancel_offer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<OfferActionRequest>,
) -> impl IntoResponse {
    match state
        .repositories
        .share_offers
        .cancel_offer(id, request.updated_by)
        .await
    {
        Ok(cancelled_offer) => Json(serde_json::json!({
            "message": "Share offer cancelled successfully",
            "offer": cancelled_offer
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to cancel share offer: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Failed to cancel share offer",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn delete_offer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repositories.share_offers.delete(id).await {
        Ok(()) => (
            StatusCode::NO_CONTENT,
            Json(serde_json::json!({
                "message": "Share offer deleted successfully"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete share offer: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Failed to delete share offer",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}
