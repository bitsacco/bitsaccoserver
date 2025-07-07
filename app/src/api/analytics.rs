use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::{
    repositories::Repositories,
    services::{analytics::ReportPeriod, Services},
};

pub fn router(repositories: Repositories, services: Services) -> Router {
    Router::new()
        .route("/", get(comprehensive_report))
        .route("/shareholders", get(shareholder_summary))
        .route("/offers", get(offer_analytics))
        .route("/market", get(market_analytics))
        .route("/transactions", get(transaction_analytics))
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

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub description: Option<String>,
}

pub async fn comprehensive_report(
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> impl IntoResponse {
    let report_period = if query.start_date.is_some() || query.end_date.is_some() {
        Some(ReportPeriod {
            start_date: query.start_date,
            end_date: query.end_date,
            description: query.description.unwrap_or_else(|| {
                match (query.start_date, query.end_date) {
                    (Some(start), Some(end)) => format!(
                        "Period from {} to {}",
                        start.format("%Y-%m-%d"),
                        end.format("%Y-%m-%d")
                    ),
                    (Some(start), None) => {
                        format!("Period from {} onwards", start.format("%Y-%m-%d"))
                    }
                    (None, Some(end)) => format!("Period up to {}", end.format("%Y-%m-%d")),
                    (None, None) => "All-time data".to_string(),
                }
            }),
        })
    } else {
        None
    };

    match state
        .services
        .analytics
        .generate_comprehensive_report(report_period)
        .await
    {
        Ok(report) => Json(serde_json::json!({
            "comprehensive_report": report,
            "metadata": {
                "report_type": "comprehensive",
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "data_sources": ["shares", "share_offers", "audit_logs"]
            }
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to generate comprehensive report: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to generate comprehensive report",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn shareholder_summary(State(state): State<AppState>) -> impl IntoResponse {
    match state.services.analytics.get_shareholder_summary().await {
        Ok(summary) => Json(serde_json::json!({
            "shareholder_summary": summary,
            "metadata": {
                "report_type": "shareholder_summary",
                "generated_at": chrono::Utc::now().to_rfc3339()
            }
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to get shareholder summary: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get shareholder summary",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn offer_analytics(State(state): State<AppState>) -> impl IntoResponse {
    match state.services.analytics.get_offer_analytics().await {
        Ok(analytics) => Json(serde_json::json!({
            "offer_analytics": analytics,
            "metadata": {
                "report_type": "offer_analytics",
                "generated_at": chrono::Utc::now().to_rfc3339()
            }
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to get offer analytics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get offer analytics",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn market_analytics(State(state): State<AppState>) -> impl IntoResponse {
    match state.services.analytics.get_market_analytics().await {
        Ok(analytics) => Json(serde_json::json!({
            "market_analytics": analytics,
            "metadata": {
                "report_type": "market_analytics",
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "metrics_included": [
                    "total_market_value",
                    "total_shares_in_circulation",
                    "average_share_price",
                    "price_distribution",
                    "ownership_distribution",
                    "concentration_metrics"
                ]
            }
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to get market analytics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get market analytics",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn transaction_analytics(State(state): State<AppState>) -> impl IntoResponse {
    match state.services.analytics.get_transaction_analytics().await {
        Ok(analytics) => Json(serde_json::json!({
            "transaction_analytics": analytics,
            "metadata": {
                "report_type": "transaction_analytics",
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "note": "Transaction data is derived from share records. Future versions will include dedicated transaction tracking."
            }
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to get transaction analytics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get transaction analytics",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}
