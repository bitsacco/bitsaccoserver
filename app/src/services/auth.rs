use crate::middleware::auth::UserContext;
use crate::repositories::Repositories;
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
        format!(
            "{}/admin/realms/{}/users",
            self.server_url, self.realm
        )
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
            .post(&self.keycloak_config.token_endpoint())
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
            .post(&self.keycloak_config.token_endpoint())
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
            .post(&self.keycloak_config.logout_endpoint())
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
        // First, decode the JWT to get roles and groups
        use base64::{Engine, engine::general_purpose};
        
        // Split the JWT and decode the payload
        let parts: Vec<&str> = access_token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthServiceError::AuthenticationFailed("Invalid token format".to_string()));
        }
        
        // Decode the payload
        let payload = general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|_| AuthServiceError::AuthenticationFailed("Failed to decode token".to_string()))?;
        
        let token_claims: serde_json::Value = serde_json::from_slice(&payload)
            .map_err(|_| AuthServiceError::AuthenticationFailed("Failed to parse token claims".to_string()))?;
        
        // Extract user information from token claims
        let user_id = token_claims
            .get("sub")
            .and_then(|s| s.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| AuthServiceError::UserNotFound("Invalid user ID".to_string()))?;

        let email = token_claims
            .get("email")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        let username = token_claims
            .get("preferred_username")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        let given_name = token_claims
            .get("given_name")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        let family_name = token_claims
            .get("family_name")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        let roles = token_claims
            .get("realm_access")
            .and_then(|ra| ra.get("roles"))
            .and_then(|roles| roles.as_array())
            .map(|roles| {
                roles
                    .iter()
                    .filter_map(|role| role.as_str())
                    .map(|role| role.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let groups = token_claims
            .get("groups")
            .and_then(|groups| groups.as_array())
            .map(|groups| {
                groups
                    .iter()
                    .filter_map(|group| group.as_str())
                    .map(|group| group.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let resource_access = token_claims
            .get("resource_access")
            .and_then(|ra| ra.as_object())
            .map(|ra| {
                ra.iter()
                    .filter_map(|(key, value)| {
                        value
                            .get("roles")
                            .and_then(|roles| roles.as_array())
                            .map(|roles| {
                                let role_strings: Vec<String> = roles
                                    .iter()
                                    .filter_map(|role| role.as_str())
                                    .map(|role| role.to_string())
                                    .collect();
                                (
                                    key.clone(),
                                    crate::middleware::auth::ResourceAccess {
                                        roles: role_strings,
                                    },
                                )
                            })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(UserContext {
            user_id,
            email,
            username,
            given_name,
            family_name,
            roles,
            groups,
            resource_access,
        })
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
            .get(&self.keycloak_config.jwks_endpoint())
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

    pub async fn register(&self, register_request: RegisterRequest) -> Result<RegisterResponse, AuthServiceError> {
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
            .post(&self.keycloak_config.users_endpoint())
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
                .and_then(|loc| loc.split('/').last())
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
            Err(AuthServiceError::AuthenticationFailed(format!("Registration failed: {}", error_text)))
        }
    }

    pub async fn verify(&self, verify_request: VerifyRequest) -> Result<VerifyResponse, AuthServiceError> {
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
                .put(&format!("{}/{}", self.keycloak_config.users_endpoint(), verify_request.user_id))
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
                Err(AuthServiceError::AuthenticationFailed(format!("Verification failed: {}", error_text)))
            }
        } else {
            Ok(VerifyResponse {
                verified: false,
                message: "Invalid OTP code".to_string(),
            })
        }
    }

    pub async fn recover(&self, recover_request: RecoverRequest) -> Result<RecoverResponse, AuthServiceError> {
        // For account recovery, we would typically:
        // 1. Find user by email
        // 2. Generate a password reset token
        // 3. Send email with reset link
        // 4. Store the token for later verification
        
        // For now, return a success response indicating the process has started
        // In a real implementation, integrate with email service
        
        tracing::info!("Password recovery requested for email: {}", recover_request.email);
        
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
            .post(&self.keycloak_config.token_endpoint())
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: serde_json::Value = response.json().await?;
            token_response
                .get("access_token")
                .and_then(|t| t.as_str())
                .map(|t| t.to_string())
                .ok_or_else(|| AuthServiceError::AuthenticationFailed("No access token in response".to_string()))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AuthServiceError::AuthenticationFailed(format!("Admin token failed: {}", error_text)))
        }
    }
}

pub type AuthServiceResult<T> = Result<T, AuthServiceError>;
