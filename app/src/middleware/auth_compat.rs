use axum::{
    extract::{Request, State},
    http::{header::COOKIE, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::contexts::auth::{decode_jwt_token, UserInfo};
use crate::middleware::auth::AuthError;
use crate::server::state::AppState;

/// Authentication state that can be injected into request extensions
#[derive(Debug, Clone)]
pub enum AuthState {
    /// User is authenticated with valid token
    Authenticated(UserInfo),
    /// User is not authenticated (no token found)
    Unauthenticated,
    /// Authentication failed due to invalid token
    InvalidToken(String),
}

/// Credentials extracted from either cookies or Authorization header
#[derive(Debug, Clone)]
pub enum Credentials {
    /// Authentication cookie containing JWT token
    Cookie(String),
    /// Bearer token from Authorization header
    Bearer(String),
}

/// NestJS-compatible authentication tokens structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
}

/// Compatibility layer for authentication that supports both cookies and bearer tokens
pub struct AuthCompatLayer;

impl AuthCompatLayer {
    /// Extract credentials from request, checking cookies first for NestJS compatibility
    pub fn extract_credentials(request: &Request) -> Result<Credentials, AuthError> {
        // First check for cookies (legacy NestJS clients)
        if let Some(cookie_token) = Self::extract_cookie_token(request) {
            return Ok(Credentials::Cookie(cookie_token));
        }

        // Then check Authorization header (new clients)
        if let Some(bearer_token) = Self::extract_bearer_token(request) {
            return Ok(Credentials::Bearer(bearer_token));
        }

        Err(AuthError::NoCredentials)
    }

    /// Extract JWT token from Authentication cookie
    fn extract_cookie_token(request: &Request) -> Option<String> {
        let cookies_header = request.headers().get(COOKIE)?.to_str().ok()?;

        // Parse cookies manually to find Authentication cookie
        for cookie_pair in cookies_header.split(';') {
            let cookie_pair = cookie_pair.trim();
            if let Some((name, value)) = cookie_pair.split_once('=') {
                if name.trim() == "Authentication" {
                    return Some(value.trim().to_string());
                }
            }
        }

        None
    }

    /// Extract JWT token from Authorization header
    fn extract_bearer_token(request: &Request) -> Option<String> {
        let auth_header = request.headers().get("authorization")?.to_str().ok()?;

        if auth_header.starts_with("Bearer ") {
            Some(auth_header[7..].to_string())
        } else {
            None
        }
    }
}

/// Enhanced auth middleware that supports both cookie and bearer token authentication
pub async fn auth_compat_middleware(
    State(_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract credentials using compatibility layer
    let credentials = match AuthCompatLayer::extract_credentials(&request) {
        Ok(creds) => creds,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Extract token from credentials
    let token = match credentials {
        Credentials::Cookie(token) | Credentials::Bearer(token) => token,
    };

    // For now, we'll rely on the client-side JWT decoding
    // In a production environment, you'd want proper server-side validation
    tracing::warn!("🔥 auth_compat_middleware: Using client-side JWT validation - NOT RECOMMENDED FOR PRODUCTION");

    // Decode JWT on server side for basic validation
    match decode_jwt_token(&token) {
        Ok(user_info) => {
            // Insert user context into request extensions
            request.extensions_mut().insert(user_info);
        }
        Err(e) => {
            tracing::error!("🔥 auth_compat_middleware: JWT decode failed: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    Ok(next.run(request).await)
}

/// Extended auth error for compatibility layer
#[derive(Debug, thiserror::Error)]
pub enum AuthCompatError {
    #[error("No credentials found")]
    NoCredentials,
    #[error("Invalid cookie format")]
    InvalidCookie,
    #[error("Token validation failed: {0}")]
    ValidationFailed(#[from] AuthError),
}

/// Enhanced SSR authentication middleware that integrates with Leptos context system
/// This middleware checks authentication before SSR rendering and provides user context
pub async fn ssr_auth_middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    let requires_auth = matches!(
        path,
        "/dashboard" | "/settings" | "/members" | "/groups" | "/shares"
    );

    tracing::info!(
        "🔥 SSR Auth: Processing request for path: {} (requires_auth: {})",
        path,
        requires_auth
    );

    if !requires_auth {
        tracing::info!(
            "🔥 SSR Auth: Path {} does not require authentication, proceeding",
            path
        );
        return Ok(next.run(request).await);
    }

    tracing::info!(
        "🔥 SSR Auth: Checking authentication for protected route: {}",
        path
    );

    // Extract auth token from cookies
    let auth_token = extract_auth_token_from_request(&request);

    match auth_token {
        Some(token) => {
            tracing::info!(
                "🔥 SSR Auth: Found auth token (length: {}), verifying...",
                token.len()
            );

            // Decode and validate the JWT token
            match decode_jwt_token(&token) {
                Ok(user_info) => {
                    tracing::info!(
                        "🔥 SSR Auth: Token valid for user: {} with roles: {:?}",
                        user_info.id,
                        user_info.roles
                    );

                    // Check if user has required permissions for the route
                    if has_required_permissions(&user_info, path) {
                        tracing::info!(
                            "🔥 SSR Auth: User has required permissions, injecting context"
                        );

                        // CRITICAL: Inject user info into request extensions for Leptos SSR access
                        request.extensions_mut().insert(user_info.clone());
                        request
                            .extensions_mut()
                            .insert(AuthState::Authenticated(user_info.clone()));

                        // Store user context in request for SSR components
                        request
                            .extensions_mut()
                            .insert(SSRAuthContext::Authenticated(user_info));

                        tracing::info!(
                            "🔥 SSR Auth: Context injected successfully, proceeding to route"
                        );
                        return Ok(next.run(request).await);
                    } else {
                        tracing::warn!("🔥 SSR Auth: User lacks required permissions for {}", path);
                        return Ok(create_access_denied_response("Insufficient permissions"));
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "🔥 SSR Auth: JWT validation failed for token (length: {}): {}",
                        token.len(),
                        e
                    );
                    tracing::error!(
                        "🔥 SSR Auth: Token preview: {}...",
                        &token[..std::cmp::min(50, token.len())]
                    );
                    request
                        .extensions_mut()
                        .insert(AuthState::InvalidToken(e.clone()));
                    request
                        .extensions_mut()
                        .insert(SSRAuthContext::InvalidToken(e));
                    Ok(create_redirect_response("/login"))
                }
            }
        }
        None => {
            tracing::info!("🔥 SSR Auth: No auth token found, redirecting to login");
            request.extensions_mut().insert(AuthState::Unauthenticated);
            request
                .extensions_mut()
                .insert(SSRAuthContext::Unauthenticated);
            return Ok(create_redirect_response("/login"));
        }
    }
}

/// SSR-specific auth context that can be extracted during server-side rendering
#[derive(Debug, Clone)]
pub enum SSRAuthContext {
    /// User is authenticated with valid token
    Authenticated(UserInfo),
    /// User is not authenticated (no token found)
    Unauthenticated,
    /// Authentication failed due to invalid token
    InvalidToken(String),
}

impl SSRAuthContext {
    /// Check if the context represents an authenticated user
    pub fn is_authenticated(&self) -> bool {
        matches!(self, SSRAuthContext::Authenticated(_))
    }

    /// Get the authenticated user info, if any
    pub fn user(&self) -> Option<&UserInfo> {
        match self {
            SSRAuthContext::Authenticated(user) => Some(user),
            _ => None,
        }
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        match self {
            SSRAuthContext::Authenticated(user) => roles
                .iter()
                .any(|role| user.roles.contains(&role.to_string())),
            _ => false,
        }
    }
}

/// Extract auth token from request cookies
fn extract_auth_token_from_request(request: &Request) -> Option<String> {
    let cookies_header = request.headers().get(COOKIE)?;

    tracing::info!("🔥 SSR Auth: Raw cookie header: {:?}", cookies_header);

    let cookies_str = cookies_header.to_str().ok()?;
    tracing::info!("🔥 SSR Auth: Parsing cookies: {}", cookies_str);

    // Look for auth_token cookie (client-side accessible) or auth token cookie (HttpOnly)
    for cookie_pair in cookies_str.split(';') {
        let cookie_pair = cookie_pair.trim();
        tracing::debug!("🔥 SSR Auth: Processing cookie pair: '{}'", cookie_pair);

        if let Some((name, value)) = cookie_pair.split_once('=') {
            let name = name.trim();
            let value = value.trim();

            tracing::debug!("🔥 SSR Auth: Cookie - name: '{}', value: '{}'", name, value);

            if name == "auth_token" || name == "Authentication" {
                tracing::info!(
                    "🔥 SSR Auth: Found {} cookie with value length: {}",
                    name,
                    value.len()
                );
                return Some(value.to_string());
            }
        }
    }

    tracing::warn!(
        "🔥 SSR Auth: No auth token found in cookies. Available cookies: {}",
        cookies_str
    );
    None
}

/// Check if user has required permissions for the given path
fn has_required_permissions(user_info: &UserInfo, path: &str) -> bool {
    // For now, any authenticated user can access any protected route
    // You can add more granular permission checks here based on roles
    match path {
        "/dashboard" | "/settings" | "/members" | "/groups" | "/shares" => {
            // Check if user has admin or superadmin role
            user_info
                .roles
                .iter()
                .any(|role| matches!(role.as_str(), "admin" | "superadmin" | "member"))
        }
        _ => true, // Non-protected routes
    }
}

/// Create a redirect response to the login page
fn create_redirect_response(location: &str) -> Response {
    use axum::http::{header, HeaderValue};

    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, HeaderValue::from_str(location).unwrap())
        .body(axum::body::Body::empty())
        .unwrap()
}

/// Create an access denied response for insufficient permissions
fn create_access_denied_response(reason: &str) -> Response {
    use axum::http::HeaderValue;

    let body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Access Denied</title>
    <style>
        body {{ font-family: system-ui, sans-serif; margin: 0; padding: 2rem; background: #f5f5f5; }}
        .container {{ max-width: 500px; margin: 2rem auto; padding: 2rem; background: white; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); text-align: center; }}
        h1 {{ color: #dc2626; margin-bottom: 1rem; }}
        p {{ color: #6b7280; margin-bottom: 2rem; }}
        a {{ display: inline-block; padding: 0.75rem 1.5rem; background: #3b82f6; color: white; text-decoration: none; border-radius: 6px; }}
        a:hover {{ background: #2563eb; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Access Denied</h1>
        <p>{}</p>
        <a href="/login">Return to Login</a>
    </div>
</body>
</html>"#,
        reason
    );

    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("content-type", HeaderValue::from_static("text/html"))
        .body(axum::body::Body::from(body))
        .unwrap()
}

/// Extract authentication state from request extensions
/// This is used by Leptos components to access auth state during SSR
pub fn extract_auth_state_from_request(request: &Request) -> AuthState {
    request
        .extensions()
        .get::<AuthState>()
        .cloned()
        .unwrap_or(AuthState::Unauthenticated)
}

/// Helper to extract authenticated user from request, if any
pub fn extract_authenticated_user_from_request(request: &Request) -> Option<UserInfo> {
    match extract_auth_state_from_request(request) {
        AuthState::Authenticated(user) => Some(user),
        _ => None,
    }
}

/// Extension of the base AuthError to include cookie-specific errors
impl From<AuthCompatError> for AuthError {
    fn from(error: AuthCompatError) -> Self {
        match error {
            AuthCompatError::NoCredentials => AuthError::InvalidToken,
            AuthCompatError::InvalidCookie => AuthError::InvalidToken,
            AuthCompatError::ValidationFailed(err) => err,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{header, HeaderMap, HeaderValue};

    fn create_test_request_with_headers(headers: HeaderMap) -> Request {
        let mut request = Request::builder()
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();

        *request.headers_mut() = headers;
        request
    }

    #[test]
    fn test_extract_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer test-jwt-token"),
        );

        let request = create_test_request_with_headers(headers);
        let token = AuthCompatLayer::extract_bearer_token(&request);

        assert_eq!(token, Some("test-jwt-token".to_string()));
    }

    #[test]
    fn test_extract_cookie_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("Authentication=test-jwt-token; RefreshToken=refresh-token"),
        );

        let request = create_test_request_with_headers(headers);
        let token = AuthCompatLayer::extract_cookie_token(&request);

        assert_eq!(token, Some("test-jwt-token".to_string()));
    }

    #[test]
    fn test_extract_credentials_prioritizes_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("Authentication=cookie-token"),
        );
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer bearer-token"),
        );

        let request = create_test_request_with_headers(headers);
        let credentials = AuthCompatLayer::extract_credentials(&request).unwrap();

        match credentials {
            Credentials::Cookie(token) => assert_eq!(token, "cookie-token"),
            Credentials::Bearer(_) => panic!("Should have extracted cookie token"),
        }
    }

    #[test]
    fn test_extract_credentials_falls_back_to_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer bearer-token"),
        );

        let request = create_test_request_with_headers(headers);
        let credentials = AuthCompatLayer::extract_credentials(&request).unwrap();

        match credentials {
            Credentials::Bearer(token) => assert_eq!(token, "bearer-token"),
            Credentials::Cookie(_) => panic!("Should have extracted bearer token"),
        }
    }

    #[test]
    fn test_extract_credentials_no_auth() {
        let headers = HeaderMap::new();
        let request = create_test_request_with_headers(headers);
        let result = AuthCompatLayer::extract_credentials(&request);

        assert!(result.is_err());
    }
}
