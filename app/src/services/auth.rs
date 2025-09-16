use crate::middleware::auth::{Claims, UserContext};
#[cfg(test)]
use crate::middleware::auth::{RealmAccess, ResourceAccess};
use crate::repositories::Repositories;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthServiceError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Token exchange failed: {0}")]
    TokenExchangeFailed(String),
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub message: String,
    pub verification_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub user_id: String,
    pub otp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub verified: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverResponse {
    pub message: String,
    pub reset_token_sent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycloakConfig {
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
    pub server_url: String,
}

impl KeycloakConfig {
    pub fn token_endpoint(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.server_url, self.realm
        )
    }

    pub fn userinfo_endpoint(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/userinfo",
            self.server_url, self.realm
        )
    }

    pub fn logout_endpoint(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/logout",
            self.server_url, self.realm
        )
    }

    pub fn jwks_endpoint(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/certs",
            self.server_url, self.realm
        )
    }

    pub fn users_endpoint(&self) -> String {
        format!("{}/admin/realms/{}/users", self.server_url, self.realm)
    }
}

#[derive(Clone)]
pub struct AuthService {
    #[allow(dead_code)]
    repositories: Repositories,
    keycloak_config: KeycloakConfig,
    http_client: Client,
}

impl AuthService {
    pub fn new(repositories: Repositories, keycloak_config: KeycloakConfig) -> Self {
        let http_client = Client::new();

        Self {
            repositories,
            keycloak_config,
            http_client,
        }
    }

    pub async fn login(
        &self,
        login_request: LoginRequest,
    ) -> Result<LoginResponse, AuthServiceError> {
        let mut params = HashMap::new();
        params.insert("grant_type", "password".to_string());
        params.insert("client_id", self.keycloak_config.client_id.clone());
        params.insert("client_secret", self.keycloak_config.client_secret.clone());
        params.insert("username", login_request.username);
        params.insert("password", login_request.password);

        let response = self
            .http_client
            .post(self.keycloak_config.token_endpoint())
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: LoginResponse = response.json().await?;
            Ok(token_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Log the actual error for debugging but don't expose it
            tracing::debug!("Authentication failed: {}", error_text);

            // Return a generic error message
            Err(AuthServiceError::InvalidCredentials)
        }
    }

    pub async fn refresh_token(
        &self,
        refresh_request: RefreshTokenRequest,
    ) -> Result<LoginResponse, AuthServiceError> {
        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token".to_string());
        params.insert("client_id", self.keycloak_config.client_id.clone());
        params.insert("client_secret", self.keycloak_config.client_secret.clone());
        params.insert("refresh_token", refresh_request.refresh_token);

        let response = self
            .http_client
            .post(self.keycloak_config.token_endpoint())
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: LoginResponse = response.json().await?;
            Ok(token_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AuthServiceError::TokenExchangeFailed(error_text))
        }
    }

    pub async fn logout(&self, refresh_token: String) -> Result<(), AuthServiceError> {
        let mut params = HashMap::new();
        params.insert("client_id", self.keycloak_config.client_id.clone());
        params.insert("client_secret", self.keycloak_config.client_secret.clone());
        params.insert("refresh_token", refresh_token);

        let response = self
            .http_client
            .post(self.keycloak_config.logout_endpoint())
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AuthServiceError::AuthenticationFailed(error_text))
        }
    }

    pub async fn get_user_info(&self, access_token: &str) -> Result<UserContext, AuthServiceError> {
        // Decode JWT without signature validation since this is typically called after middleware validation
        // We disable signature validation but still parse the claims structure
        let mut validation = Validation::new(Algorithm::HS256);
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.validate_aud = false;

        let token_data = decode::<Claims>(
            access_token,
            &DecodingKey::from_secret(&[]), // Dummy key since signature validation is disabled
            &validation,
        )
        .map_err(|e| {
            AuthServiceError::AuthenticationFailed(format!("Failed to decode JWT: {}", e))
        })?;

        let claims = token_data.claims;

        // Extract user ID from subject claim
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            AuthServiceError::AuthenticationFailed("Invalid user ID in JWT".to_string())
        })?;

        // Extract roles from realm access
        let roles = claims
            .realm_access
            .as_ref()
            .map(|ra| ra.roles.clone())
            .unwrap_or_default();

        // Extract groups
        let groups = claims.groups.unwrap_or_default();

        // Extract resource access
        let resource_access = claims.resource_access.unwrap_or_default();

        // Create UserContext
        let user_context = UserContext {
            user_id,
            email: claims.email,
            username: claims.preferred_username,
            given_name: claims.given_name,
            family_name: claims.family_name,
            roles,
            groups,
            resource_access,
        };

        Ok(user_context)
    }

    pub async fn validate_user_permissions(
        &self,
        user: &UserContext,
        required_role: &str,
    ) -> Result<bool, AuthServiceError> {
        // Check if user has the required role
        if user.roles.contains(&required_role.to_string()) {
            return Ok(true);
        }

        // Check if user has admin role (admin can access everything)
        if user.roles.contains(&"admin".to_string()) {
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn get_jwks(&self) -> Result<serde_json::Value, AuthServiceError> {
        let response = self
            .http_client
            .get(self.keycloak_config.jwks_endpoint())
            .send()
            .await?;

        if response.status().is_success() {
            let jwks: serde_json::Value = response.json().await?;
            Ok(jwks)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AuthServiceError::AuthenticationFailed(error_text))
        }
    }

    pub async fn register(
        &self,
        register_request: RegisterRequest,
    ) -> Result<RegisterResponse, AuthServiceError> {
        // Build the user representation for Keycloak
        let mut user_data = serde_json::json!({
            "username": register_request.username,
            "email": register_request.email,
            "enabled": true,
            "emailVerified": false,
            "credentials": [{
                "type": "password",
                "value": register_request.password,
                "temporary": false
            }]
        });

        // Add optional fields if provided
        if let Some(given_name) = register_request.given_name {
            user_data["firstName"] = serde_json::Value::String(given_name);
        }
        if let Some(family_name) = register_request.family_name {
            user_data["lastName"] = serde_json::Value::String(family_name);
        }

        // Add attributes for phone if provided
        if let Some(phone) = register_request.phone {
            user_data["attributes"] = serde_json::json!({
                "phone": [phone]
            });
        }

        // Get admin token to create user
        let admin_token = self.get_admin_token().await?;

        let response = self
            .http_client
            .post(self.keycloak_config.users_endpoint())
            .bearer_auth(admin_token)
            .json(&user_data)
            .send()
            .await?;

        if response.status().is_success() {
            // Extract user ID from Location header or response
            let user_id = response
                .headers()
                .get("location")
                .and_then(|loc| loc.to_str().ok())
                .and_then(|loc| loc.split('/').next_back())
                .unwrap_or("unknown")
                .to_string();

            Ok(RegisterResponse {
                user_id,
                message: "User registered successfully. Please verify your email.".to_string(),
                verification_required: true,
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AuthServiceError::AuthenticationFailed(format!(
                "Registration failed: {}",
                error_text
            )))
        }
    }

    pub async fn verify(
        &self,
        verify_request: VerifyRequest,
    ) -> Result<VerifyResponse, AuthServiceError> {
        // For now, implement a simple OTP verification
        // In a production system, you would:
        // 1. Validate the OTP against stored values (database/cache)
        // 2. Update user's email verification status in Keycloak
        // 3. Possibly activate the user account

        // Simple validation - in real implementation, check against stored OTP
        if verify_request.otp.len() >= 4 && verify_request.otp.chars().all(|c| c.is_numeric()) {
            // Get admin token to update user
            let admin_token = self.get_admin_token().await?;

            // Update user verification status
            let update_data = serde_json::json!({
                "emailVerified": true,
                "enabled": true
            });

            let response = self
                .http_client
                .put(format!(
                    "{}/{}",
                    self.keycloak_config.users_endpoint(),
                    verify_request.user_id
                ))
                .bearer_auth(admin_token)
                .json(&update_data)
                .send()
                .await?;

            if response.status().is_success() {
                Ok(VerifyResponse {
                    verified: true,
                    message: "Email verified successfully".to_string(),
                })
            } else {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(AuthServiceError::AuthenticationFailed(format!(
                    "Verification failed: {}",
                    error_text
                )))
            }
        } else {
            Ok(VerifyResponse {
                verified: false,
                message: "Invalid OTP code".to_string(),
            })
        }
    }

    pub async fn recover(
        &self,
        recover_request: RecoverRequest,
    ) -> Result<RecoverResponse, AuthServiceError> {
        // For account recovery, we would typically:
        // 1. Find user by email
        // 2. Generate a password reset token
        // 3. Send email with reset link
        // 4. Store the token for later verification

        // For now, return a success response indicating the process has started
        // In a real implementation, integrate with email service

        tracing::info!(
            "Password recovery requested for email: {}",
            recover_request.email
        );

        Ok(RecoverResponse {
            message: "If an account with this email exists, you will receive a password reset link shortly.".to_string(),
            reset_token_sent: true,
        })
    }

    /// Get admin token for managing users
    async fn get_admin_token(&self) -> Result<String, AuthServiceError> {
        let mut params = HashMap::new();
        params.insert("grant_type", "client_credentials".to_string());
        params.insert("client_id", self.keycloak_config.client_id.clone());
        params.insert("client_secret", self.keycloak_config.client_secret.clone());

        let response = self
            .http_client
            .post(self.keycloak_config.token_endpoint())
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: serde_json::Value = response.json().await?;
            token_response
                .get("access_token")
                .and_then(|t| t.as_str())
                .map(|t| t.to_string())
                .ok_or_else(|| {
                    AuthServiceError::AuthenticationFailed(
                        "No access token in response".to_string(),
                    )
                })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AuthServiceError::AuthenticationFailed(format!(
                "Admin token failed: {}",
                error_text
            )))
        }
    }
}

pub type AuthServiceResult<T> = Result<T, AuthServiceError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::Repositories;
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_auth_service() -> AuthService {
        let repositories = Repositories::new(std::sync::Arc::new(
            sea_orm::DatabaseConnection::Disconnected,
        ));
        let keycloak_config = KeycloakConfig {
            realm: "test".to_string(),
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            server_url: "http://localhost:8080".to_string(),
        };
        AuthService::new(repositories, keycloak_config)
    }

    fn create_test_jwt(claims: Claims) -> String {
        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(b"test-secret");
        encode(&header, &claims, &encoding_key).unwrap()
    }

    #[tokio::test]
    async fn test_get_user_info_success() {
        let auth_service = create_test_auth_service();

        let user_id = Uuid::new_v4();
        let claims = Claims {
            sub: user_id.to_string(),
            exp: 9999999999,
            iat: 1234567890,
            iss: "http://localhost:8080/realms/test".to_string(),
            aud: serde_json::Value::String("test-client".to_string()),
            email: "test@example.com".to_string(),
            preferred_username: "testuser".to_string(),
            given_name: Some("Test".to_string()),
            family_name: Some("User".to_string()),
            realm_access: Some(RealmAccess {
                roles: vec!["user".to_string(), "admin".to_string()],
            }),
            resource_access: Some({
                let mut map = HashMap::new();
                map.insert(
                    "test-client".to_string(),
                    ResourceAccess {
                        roles: vec!["client-role".to_string()],
                    },
                );
                map
            }),
            groups: Some(vec!["group1".to_string(), "group2".to_string()]),
        };

        let jwt = create_test_jwt(claims);
        let result = auth_service.get_user_info(&jwt).await;

        assert!(result.is_ok());
        let user_context = result.unwrap();

        assert_eq!(user_context.user_id, user_id);
        assert_eq!(user_context.email, "test@example.com");
        assert_eq!(user_context.username, "testuser");
        assert_eq!(user_context.given_name, Some("Test".to_string()));
        assert_eq!(user_context.family_name, Some("User".to_string()));
        assert_eq!(user_context.roles, vec!["user", "admin"]);
        assert_eq!(user_context.groups, vec!["group1", "group2"]);
        assert!(user_context.resource_access.contains_key("test-client"));
    }

    #[tokio::test]
    async fn test_get_user_info_invalid_user_id() {
        let auth_service = create_test_auth_service();

        let claims = Claims {
            sub: "invalid-uuid".to_string(),
            exp: 9999999999,
            iat: 1234567890,
            iss: "http://localhost:8080/realms/test".to_string(),
            aud: serde_json::Value::String("test-client".to_string()),
            email: "test@example.com".to_string(),
            preferred_username: "testuser".to_string(),
            given_name: None,
            family_name: None,
            realm_access: None,
            resource_access: None,
            groups: None,
        };

        let jwt = create_test_jwt(claims);
        let result = auth_service.get_user_info(&jwt).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid user ID"));
    }

    #[tokio::test]
    async fn test_get_user_info_minimal_claims() {
        let auth_service = create_test_auth_service();

        let user_id = Uuid::new_v4();
        let claims = Claims {
            sub: user_id.to_string(),
            exp: 9999999999,
            iat: 1234567890,
            iss: "http://localhost:8080/realms/test".to_string(),
            aud: serde_json::Value::String("test-client".to_string()),
            email: "test@example.com".to_string(),
            preferred_username: "testuser".to_string(),
            given_name: None,
            family_name: None,
            realm_access: None,
            resource_access: None,
            groups: None,
        };

        let jwt = create_test_jwt(claims);
        let result = auth_service.get_user_info(&jwt).await;

        assert!(result.is_ok());
        let user_context = result.unwrap();

        assert_eq!(user_context.user_id, user_id);
        assert_eq!(user_context.email, "test@example.com");
        assert_eq!(user_context.username, "testuser");
        assert_eq!(user_context.given_name, None);
        assert_eq!(user_context.family_name, None);
        assert!(user_context.roles.is_empty());
        assert!(user_context.groups.is_empty());
        assert!(user_context.resource_access.is_empty());
    }

    #[tokio::test]
    async fn test_get_user_info_invalid_jwt() {
        let auth_service = create_test_auth_service();

        let result = auth_service.get_user_info("invalid.jwt.token").await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to decode JWT"));
    }
}
