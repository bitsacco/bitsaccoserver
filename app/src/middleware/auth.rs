use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::server::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub resource_access: std::collections::HashMap<String, ResourceAccess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub aud: serde_json::Value,
    pub email: String,
    pub preferred_username: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub resource_access: Option<std::collections::HashMap<String, ResourceAccess>>,
    pub groups: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Clone)]
pub struct JwtConfig {
    pub decoding_key: DecodingKey,
    pub validation: Validation,
    pub cache: Cache<String, UserContext>,
}

impl JwtConfig {
    pub fn new(
        public_key: &str,
        issuer: &str,
        audience: &str,
    ) -> Result<Self, jsonwebtoken::errors::Error> {
        // For development, if no key is provided, use HS256 with a dummy secret
        let (validation, decoding_key) = if public_key.is_empty() {
            let mut validation = Validation::new(Algorithm::HS256);
            validation.set_issuer(&[issuer]);
            validation.set_audience(&[audience]);
            validation.validate_exp = false; // Disable for development
            validation.validate_nbf = false;
            let decoding_key = DecodingKey::from_secret(b"development-secret");
            (validation, decoding_key)
        } else {
            let mut validation = Validation::new(Algorithm::RS256);
            validation.set_issuer(&[issuer]);
            validation.set_audience(&[audience]);
            validation.validate_exp = true;
            validation.validate_nbf = true;
            let decoding_key = DecodingKey::from_rsa_pem(public_key.as_bytes())?;
            (validation, decoding_key)
        };

        let cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(std::time::Duration::from_secs(300)) // 5 minutes
            .build();

        Ok(Self {
            decoding_key,
            validation,
            cache,
        })
    }
}

pub struct AuthMiddleware;

// Standalone middleware function with correct signature
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            &header[7..] // Remove "Bearer " prefix
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let user_context = match validate_token(&state, token).await {
        Ok(context) => context,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Insert user context into request extensions
    request.extensions_mut().insert(user_context);

    Ok(next.run(request).await)
}

async fn validate_token(state: &AppState, token: &str) -> Result<UserContext, AuthError> {
    let jwt_config = &state.jwt_config;

    // Check cache first
    if let Some(cached_context) = jwt_config.cache.get(token).await {
        return Ok(cached_context);
    }

    // Decode and validate JWT
    let claims = decode::<Claims>(token, &jwt_config.decoding_key, &jwt_config.validation)
        .map_err(|_| AuthError::InvalidToken)?;

    let user_id = Uuid::parse_str(&claims.claims.sub).map_err(|_| AuthError::InvalidUserId)?;

    let roles = claims
        .claims
        .realm_access
        .as_ref()
        .map(|ra| ra.roles.clone())
        .unwrap_or_default();

    let groups = claims.claims.groups.unwrap_or_default();

    let resource_access = claims.claims.resource_access.unwrap_or_default();

    let user_context = UserContext {
        user_id,
        email: claims.claims.email,
        username: claims.claims.preferred_username,
        given_name: claims.claims.given_name,
        family_name: claims.claims.family_name,
        roles,
        groups,
        resource_access,
    };

    // Cache the user context
    jwt_config
        .cache
        .insert(token.to_string(), user_context.clone())
        .await;

    Ok(user_context)
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Invalid user ID")]
    InvalidUserId,
    #[error("Missing required claim")]
    MissingClaim,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
}

// Extension trait for extracting user context from request
pub trait UserContextExt {
    fn user_context(&self) -> Option<&UserContext>;
}

impl UserContextExt for Request {
    fn user_context(&self) -> Option<&UserContext> {
        self.extensions().get::<UserContext>()
    }
}

// Helper function to extract user context from request extensions
pub fn extract_user_context(request: &Request) -> Result<&UserContext, StatusCode> {
    request
        .extensions()
        .get::<UserContext>()
        .ok_or(StatusCode::UNAUTHORIZED)
}

// Role-based access control helper
pub fn has_role(user: &UserContext, role: &str) -> bool {
    user.roles.contains(&role.to_string())
}

pub fn has_any_role(user: &UserContext, roles: &[&str]) -> bool {
    roles
        .iter()
        .any(|role| user.roles.contains(&role.to_string()))
}

pub fn has_resource_role(user: &UserContext, resource: &str, role: &str) -> bool {
    user.resource_access
        .get(resource)
        .map(|access| access.roles.contains(&role.to_string()))
        .unwrap_or(false)
}

// Macro for easy role checking
#[macro_export]
macro_rules! require_role {
    ($user:expr, $role:expr) => {
        if !crate::middleware::auth::has_role($user, $role) {
            return Err(StatusCode::FORBIDDEN);
        }
    };
}

#[macro_export]
macro_rules! require_any_role {
    ($user:expr, $($role:expr),+) => {
        if !crate::middleware::auth::has_any_role($user, &[$($role),+]) {
            return Err(StatusCode::FORBIDDEN);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_checking() {
        let user = UserContext {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            given_name: None,
            family_name: None,
            roles: vec!["admin".to_string(), "user".to_string()],
            groups: vec![],
            resource_access: std::collections::HashMap::new(),
        };

        assert!(has_role(&user, "admin"));
        assert!(has_role(&user, "user"));
        assert!(!has_role(&user, "manager"));
        assert!(has_any_role(&user, &["admin", "manager"]));
        assert!(!has_any_role(&user, &["manager", "supervisor"]));
    }
}
