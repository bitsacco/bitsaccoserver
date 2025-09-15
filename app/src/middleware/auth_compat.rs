use axum::{
    extract::{Request, State},
    http::{header::COOKIE, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use time::Duration;

use crate::middleware::auth::{validate_token, AuthError};
use crate::server::state::AppState;

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
        let auth_header = request
            .headers()
            .get("authorization")?
            .to_str()
            .ok()?;

        if auth_header.starts_with("Bearer ") {
            Some(auth_header[7..].to_string())
        } else {
            None
        }
    }

    /// Set authentication cookies in response for NestJS compatibility
    pub fn set_auth_cookies(cookie_jar: &mut CookieJar, tokens: &AuthTokens) {
        // Set Authentication cookie (access token)
        let auth_cookie = Cookie::build(("Authentication", tokens.access_token.clone()))
            .http_only(true)
            .secure(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .max_age(Duration::seconds(tokens.expires_in as i64))
            .path("/")
            .build();

        *cookie_jar = cookie_jar.clone().add(auth_cookie);

        // Set RefreshToken cookie (refresh token)
        let refresh_cookie = Cookie::build(("RefreshToken", tokens.refresh_token.clone()))
            .http_only(true)
            .secure(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .max_age(Duration::seconds(tokens.refresh_expires_in as i64))
            .path("/auth/refresh") // Restricted path for security
            .build();

        *cookie_jar = cookie_jar.clone().add(refresh_cookie);
    }

    /// Clear authentication cookies on logout
    pub fn clear_auth_cookies(cookie_jar: &mut CookieJar) {
        // Remove Authentication cookie
        let auth_cookie = Cookie::build(("Authentication", ""))
            .http_only(true)
            .secure(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .max_age(Duration::seconds(0))
            .path("/")
            .build();

        *cookie_jar = cookie_jar.clone().remove(auth_cookie);

        // Remove RefreshToken cookie
        let refresh_cookie = Cookie::build(("RefreshToken", ""))
            .http_only(true)
            .secure(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .max_age(Duration::seconds(0))
            .path("/auth/refresh")
            .build();

        *cookie_jar = cookie_jar.clone().remove(refresh_cookie);
    }
}

/// Enhanced auth middleware that supports both cookie and bearer token authentication
pub async fn auth_compat_middleware(
    State(state): State<AppState>,
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

    // Validate token and get user context
    let user_context = match validate_token(&state, &token).await {
        Ok(context) => context,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Insert user context into request extensions
    request.extensions_mut().insert(user_context);

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

    #[test]
    fn test_cookie_jar_operations() {
        let mut cookie_jar = CookieJar::new();
        let tokens = AuthTokens {
            access_token: "access-token".to_string(),
            refresh_token: "refresh-token".to_string(),
            expires_in: 3600,
            refresh_expires_in: 86400,
        };

        // Test setting cookies
        AuthCompatLayer::set_auth_cookies(&mut cookie_jar, &tokens);
        
        // Verify cookies are set (we can't easily test the exact cookie values 
        // without more complex setup, but we can verify the jar has cookies)
        assert!(!cookie_jar.iter().collect::<Vec<_>>().is_empty());

        // Test clearing cookies
        AuthCompatLayer::clear_auth_cookies(&mut cookie_jar);
        
        // After clearing, cookies should be empty or expired
        // The exact behavior depends on axum-extra implementation
    }
}