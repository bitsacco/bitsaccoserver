use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use crate::middleware::auth::{extract_user_context, UserContext};
use crate::middleware::auth_compat::{AuthCompatLayer, AuthTokens};
use crate::repositories::Repositories;
use crate::services::auth::{AuthServiceError, LoginRequest, RefreshTokenRequest, RegisterRequest, VerifyRequest, RecoverRequest};
use crate::services::Services;

/// NestJS-compatible login response format
#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NestJSLoginResponse {
    pub user: UserContext,
    pub authenticated: bool,
    pub accessToken: String,
    pub refreshToken: String,
}

/// NestJS-compatible error response format
#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NestJSErrorResponse {
    pub statusCode: u16,
    pub message: String,
    pub error: String,
}

/// NestJS-compatible logout response
#[derive(Debug, Serialize, Deserialize)]
pub struct NestJSLogoutResponse {
    pub message: String,
}

/// NestJS-compatible authenticate response 
#[derive(Debug, Serialize, Deserialize)]
pub struct NestJSAuthenticateResponse {
    pub authenticated: bool,
    pub user: Option<UserContext>,
}

/// NestJS-compatible register response
#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NestJSRegisterResponse {
    pub userId: String,
    pub message: String,
    pub verificationRequired: bool,
}

/// NestJS-compatible verify response
#[derive(Debug, Serialize, Deserialize)]
pub struct NestJSVerifyResponse {
    pub verified: bool,
    pub message: String,
}

/// NestJS-compatible recover response
#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NestJSRecoverResponse {
    pub message: String,
    pub resetTokenSent: bool,
}

/// Router for NestJS-compatible auth endpoints (without /api prefix)
pub fn compat_router<S>(repositories: Repositories, services: Services) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/login", post(compat_login))
        .route("/logout", post(compat_logout))
        .route("/refresh", post(compat_refresh))
        .route("/authenticate", post(authenticate_session))
        .route("/register", post(register_user))
        .route("/verify", post(verify_user))
        .route("/recover", post(recover_account))
        .with_state((repositories, services))
}

/// NestJS-compatible login endpoint with cookie support
pub async fn compat_login(
    State((_, services)): State<(Repositories, Services)>,
    mut cookie_jar: CookieJar,
    Json(login_request): Json<LoginRequest>,
) -> Result<(CookieJar, Json<NestJSLoginResponse>), (StatusCode, Json<NestJSErrorResponse>)> {
    match services.auth.login(login_request).await {
        Ok(token_response) => {
            // Get user info from the access token
            match services
                .auth
                .get_user_info(&token_response.access_token)
                .await
            {
                Ok(user_context) => {
                    // Create auth tokens for cookie setting
                    let auth_tokens = AuthTokens {
                        access_token: token_response.access_token.clone(),
                        refresh_token: token_response.refresh_token.clone(),
                        expires_in: token_response.expires_in,
                        refresh_expires_in: token_response.refresh_expires_in,
                    };

                    // Set cookies for NestJS compatibility
                    AuthCompatLayer::set_auth_cookies(&mut cookie_jar, &auth_tokens);

                    // Create NestJS-compatible response
                    let response = NestJSLoginResponse {
                        user: user_context,
                        authenticated: true,
                        accessToken: token_response.access_token,
                        refreshToken: token_response.refresh_token,
                    };

                    Ok((cookie_jar, Json(response)))
                }
                Err(e) => {
                    let error_response = NestJSErrorResponse {
                        statusCode: 500,
                        message: e.to_string(),
                        error: "Internal Server Error".to_string(),
                    };
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Err(e) => {
            let error_response = NestJSErrorResponse {
                statusCode: 401,
                message: e.to_string(),
                error: "Unauthorized".to_string(),
            };
            Err((StatusCode::UNAUTHORIZED, Json(error_response)))
        }
    }
}

/// NestJS-compatible logout endpoint with cookie clearing
pub async fn compat_logout(
    State((_, services)): State<(Repositories, Services)>,
    mut cookie_jar: CookieJar,
    Json(logout_request): Json<serde_json::Value>,
) -> Result<(CookieJar, Json<NestJSLogoutResponse>), (StatusCode, Json<NestJSErrorResponse>)> {
    // Extract refresh token from request body or cookies
    let refresh_token = logout_request
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .or_else(|| {
            // Try to get from RefreshToken cookie
            cookie_jar
                .get("RefreshToken")
                .map(|cookie| cookie.value())
        });

    if let Some(refresh_token) = refresh_token {
        // Attempt to logout on Keycloak side
        let _ = services.auth.logout(refresh_token.to_string()).await;
    }

    // Clear authentication cookies regardless of Keycloak logout result
    AuthCompatLayer::clear_auth_cookies(&mut cookie_jar);

    let response = NestJSLogoutResponse {
        message: "Logged out successfully".to_string(),
    };

    Ok((cookie_jar, Json(response)))
}

/// NestJS-compatible refresh endpoint with cookie support
pub async fn compat_refresh(
    State((_, services)): State<(Repositories, Services)>,
    mut cookie_jar: CookieJar,
    Json(refresh_request): Json<RefreshTokenRequest>,
) -> Result<(CookieJar, Json<NestJSLoginResponse>), (StatusCode, Json<NestJSErrorResponse>)> {
    match services.auth.refresh_token(refresh_request).await {
        Ok(token_response) => {
            // Get user info from the new access token
            match services
                .auth
                .get_user_info(&token_response.access_token)
                .await
            {
                Ok(user_context) => {
                    // Create auth tokens for cookie setting
                    let auth_tokens = AuthTokens {
                        access_token: token_response.access_token.clone(),
                        refresh_token: token_response.refresh_token.clone(),
                        expires_in: token_response.expires_in,
                        refresh_expires_in: token_response.refresh_expires_in,
                    };

                    // Update cookies with new tokens
                    AuthCompatLayer::set_auth_cookies(&mut cookie_jar, &auth_tokens);

                    // Create NestJS-compatible response
                    let response = NestJSLoginResponse {
                        user: user_context,
                        authenticated: true,
                        accessToken: token_response.access_token,
                        refreshToken: token_response.refresh_token,
                    };

                    Ok((cookie_jar, Json(response)))
                }
                Err(e) => {
                    let error_response = NestJSErrorResponse {
                        statusCode: 500,
                        message: e.to_string(),
                        error: "Internal Server Error".to_string(),
                    };
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Err(e) => {
            let error_response = NestJSErrorResponse {
                statusCode: 401,
                message: e.to_string(),
                error: "Unauthorized".to_string(),
            };
            Err((StatusCode::UNAUTHORIZED, Json(error_response)))
        }
    }
}

/// NestJS-compatible authenticate endpoint (cookie-based session validation)
pub async fn authenticate_session(
    request: axum::extract::Request,
) -> Result<Json<NestJSAuthenticateResponse>, (StatusCode, Json<NestJSErrorResponse>)> {
    match extract_user_context(&request) {
        Ok(user_context) => {
            let response = NestJSAuthenticateResponse {
                authenticated: true,
                user: Some(user_context.clone()),
            };
            Ok(Json(response))
        }
        Err(_) => {
            let response = NestJSAuthenticateResponse {
                authenticated: false,
                user: None,
            };
            // Return 200 with authenticated: false for NestJS compatibility
            // (NestJS doesn't return 401 for this endpoint)
            Ok(Json(response))
        }
    }
}

/// NestJS-compatible user registration endpoint
pub async fn register_user(
    State((_, services)): State<(Repositories, Services)>,
    Json(register_request): Json<RegisterRequest>,
) -> Result<Json<NestJSRegisterResponse>, (StatusCode, Json<NestJSErrorResponse>)> {
    match services.auth.register(register_request).await {
        Ok(register_response) => {
            let response = NestJSRegisterResponse {
                userId: register_response.user_id,
                message: register_response.message,
                verificationRequired: register_response.verification_required,
            };
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = NestJSErrorResponse {
                statusCode: 400,
                message: e.to_string(),
                error: "Bad Request".to_string(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// NestJS-compatible user verification endpoint
pub async fn verify_user(
    State((_, services)): State<(Repositories, Services)>,
    Json(verify_request): Json<VerifyRequest>,
) -> Result<Json<NestJSVerifyResponse>, (StatusCode, Json<NestJSErrorResponse>)> {
    match services.auth.verify(verify_request).await {
        Ok(verify_response) => {
            let response = NestJSVerifyResponse {
                verified: verify_response.verified,
                message: verify_response.message,
            };
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = NestJSErrorResponse {
                statusCode: 400,
                message: e.to_string(),
                error: "Bad Request".to_string(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// NestJS-compatible account recovery endpoint
pub async fn recover_account(
    State((_, services)): State<(Repositories, Services)>,
    Json(recover_request): Json<RecoverRequest>,
) -> Result<Json<NestJSRecoverResponse>, (StatusCode, Json<NestJSErrorResponse>)> {
    match services.auth.recover(recover_request).await {
        Ok(recover_response) => {
            let response = NestJSRecoverResponse {
                message: recover_response.message,
                resetTokenSent: recover_response.reset_token_sent,
            };
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = NestJSErrorResponse {
                statusCode: 500,
                message: e.to_string(),
                error: "Internal Server Error".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Helper function to convert AuthServiceError to NestJS error response
#[allow(dead_code)]
fn auth_error_to_nestjs_response(
    error: AuthServiceError,
) -> (StatusCode, Json<NestJSErrorResponse>) {
    let (status_code, error_type) = match error {
        AuthServiceError::AuthenticationFailed(_) => {
            (StatusCode::UNAUTHORIZED, "Unauthorized")
        }
        AuthServiceError::TokenExchangeFailed(_) => {
            (StatusCode::UNAUTHORIZED, "Unauthorized")
        }
        AuthServiceError::UserNotFound(_) => (StatusCode::NOT_FOUND, "Not Found"),
        AuthServiceError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Unauthorized"),
        AuthServiceError::HttpError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
        AuthServiceError::JsonError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
    };

    let error_response = NestJSErrorResponse {
        statusCode: status_code.as_u16(),
        message: error.to_string(),
        error: error_type.to_string(),
    };

    (status_code, Json(error_response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nestjs_response_format() {
        let user = UserContext {
            user_id: uuid::Uuid::new_v4(),
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            given_name: Some("Test".to_string()),
            family_name: Some("User".to_string()),
            roles: vec!["user".to_string()],
            groups: vec![],
            resource_access: std::collections::HashMap::new(),
        };

        let response = NestJSLoginResponse {
            user: user.clone(),
            authenticated: true,
            accessToken: "access-token".to_string(),
            refreshToken: "refresh-token".to_string(),
        };

        // Verify the response structure matches NestJS format
        assert!(response.authenticated);
        assert_eq!(response.accessToken, "access-token");
        assert_eq!(response.refreshToken, "refresh-token");
        assert_eq!(response.user.email, user.email);
        assert_eq!(response.user.username, user.username);
    }

    #[test]
    fn test_nestjs_error_format() {
        let error = NestJSErrorResponse {
            statusCode: 401,
            message: "Authentication failed".to_string(),
            error: "Unauthorized".to_string(),
        };

        assert_eq!(error.statusCode, 401);
        assert_eq!(error.error, "Unauthorized");
        assert_eq!(error.message, "Authentication failed");
    }

    #[test]
    fn test_authenticate_response_format() {
        let auth_response = NestJSAuthenticateResponse {
            authenticated: true,
            user: None,
        };

        assert!(auth_response.authenticated);
        assert!(auth_response.user.is_none());

        let unauth_response = NestJSAuthenticateResponse {
            authenticated: false,
            user: None,
        };

        assert!(!unauth_response.authenticated);
        assert!(unauth_response.user.is_none());
    }

    #[test]
    fn test_register_response_format() {
        let response = NestJSRegisterResponse {
            userId: "test-user-id".to_string(),
            message: "User registered successfully".to_string(),
            verificationRequired: true,
        };

        assert_eq!(response.userId, "test-user-id");
        assert_eq!(response.message, "User registered successfully");
        assert!(response.verificationRequired);
    }

    #[test]
    fn test_verify_response_format() {
        let success_response = NestJSVerifyResponse {
            verified: true,
            message: "Email verified successfully".to_string(),
        };

        assert!(success_response.verified);
        assert_eq!(success_response.message, "Email verified successfully");

        let error_response = NestJSVerifyResponse {
            verified: false,
            message: "Invalid OTP code".to_string(),
        };

        assert!(!error_response.verified);
        assert_eq!(error_response.message, "Invalid OTP code");
    }

    #[test]
    fn test_recover_response_format() {
        let response = NestJSRecoverResponse {
            message: "Password reset email sent".to_string(),
            resetTokenSent: true,
        };

        assert_eq!(response.message, "Password reset email sent");
        assert!(response.resetTokenSent);
    }
}