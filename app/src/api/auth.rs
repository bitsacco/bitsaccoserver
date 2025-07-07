use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::middleware::auth::{extract_user_context, UserContext};
use crate::repositories::Repositories;
use crate::services::auth::{AuthServiceError, LoginRequest, RefreshTokenRequest};
use crate::services::Services;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
    pub token_type: String,
    pub user: UserContext,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeResponse {
    pub user: UserContext,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthErrorResponse {
    pub error: String,
    pub message: String,
}

pub fn router(repositories: Repositories, services: Services) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/validate", get(validate_token))
        .with_state((repositories, services))
}

pub async fn login(
    State((_, services)): State<(Repositories, Services)>,
    Json(login_request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    match services.auth.login(login_request).await {
        Ok(token_response) => {
            // Get user info from the access token
            match services
                .auth
                .get_user_info(&token_response.access_token)
                .await
            {
                Ok(user_context) => {
                    let response = LoginResponse {
                        access_token: token_response.access_token,
                        refresh_token: token_response.refresh_token,
                        expires_in: token_response.expires_in,
                        refresh_expires_in: token_response.refresh_expires_in,
                        token_type: token_response.token_type,
                        user: user_context,
                    };
                    Ok(Json(response))
                }
                Err(e) => {
                    let error_response = AuthErrorResponse {
                        error: "user_info_failed".to_string(),
                        message: e.to_string(),
                    };
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Err(e) => {
            let error_response = AuthErrorResponse {
                error: "authentication_failed".to_string(),
                message: e.to_string(),
            };
            Err((StatusCode::UNAUTHORIZED, Json(error_response)))
        }
    }
}

pub async fn refresh_token(
    State((_, services)): State<(Repositories, Services)>,
    Json(refresh_request): Json<RefreshTokenRequest>,
) -> Result<Json<crate::services::auth::LoginResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    match services.auth.refresh_token(refresh_request).await {
        Ok(token_response) => Ok(Json(token_response)),
        Err(e) => {
            let error_response = AuthErrorResponse {
                error: "token_refresh_failed".to_string(),
                message: e.to_string(),
            };
            Err((StatusCode::UNAUTHORIZED, Json(error_response)))
        }
    }
}

pub async fn logout(
    State((_, services)): State<(Repositories, Services)>,
    Json(logout_request): Json<LogoutRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<AuthErrorResponse>)> {
    match services.auth.logout(logout_request.refresh_token).await {
        Ok(_) => {
            let response = serde_json::json!({
                "message": "Logged out successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = AuthErrorResponse {
                error: "logout_failed".to_string(),
                message: e.to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn me(
    request: axum::extract::Request,
) -> Result<Json<MeResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    match extract_user_context(&request) {
        Ok(user_context) => {
            let response = MeResponse {
                user: user_context.clone(),
            };
            Ok(Json(response))
        }
        Err(status_code) => {
            let error_response = AuthErrorResponse {
                error: "user_context_missing".to_string(),
                message: "User context not found in request".to_string(),
            };
            Err((status_code, Json(error_response)))
        }
    }
}

pub async fn validate_token(
    request: axum::extract::Request,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<AuthErrorResponse>)> {
    match extract_user_context(&request) {
        Ok(user_context) => {
            let response = serde_json::json!({
                "valid": true,
                "user_id": user_context.user_id,
                "email": user_context.email,
                "username": user_context.username,
                "roles": user_context.roles,
                "groups": user_context.groups
            });
            Ok(Json(response))
        }
        Err(status_code) => {
            let error_response = AuthErrorResponse {
                error: "token_invalid".to_string(),
                message: "Token validation failed".to_string(),
            };
            Err((status_code, Json(error_response)))
        }
    }
}

// Helper function to convert AuthServiceError to HTTP response
#[allow(dead_code)]
fn auth_error_to_response(error: AuthServiceError) -> (StatusCode, Json<AuthErrorResponse>) {
    let (status_code, error_type) = match error {
        AuthServiceError::AuthenticationFailed(_) => {
            (StatusCode::UNAUTHORIZED, "authentication_failed")
        }
        AuthServiceError::TokenExchangeFailed(_) => {
            (StatusCode::UNAUTHORIZED, "token_exchange_failed")
        }
        AuthServiceError::UserNotFound(_) => (StatusCode::NOT_FOUND, "user_not_found"),
        AuthServiceError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
        AuthServiceError::HttpError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "http_error"),
        AuthServiceError::JsonError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "json_error"),
    };

    let error_response = AuthErrorResponse {
        error: error_type.to_string(),
        message: error.to_string(),
    };

    (status_code, Json(error_response))
}
