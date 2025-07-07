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
        let response = self
            .http_client
            .get(&self.keycloak_config.userinfo_endpoint())
            .bearer_auth(access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let user_info: serde_json::Value = response.json().await?;

            // Extract user information from Keycloak response
            let user_id = user_info
                .get("sub")
                .and_then(|s| s.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or_else(|| AuthServiceError::UserNotFound("Invalid user ID".to_string()))?;

            let email = user_info
                .get("email")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            let username = user_info
                .get("preferred_username")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            let given_name = user_info
                .get("given_name")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());

            let family_name = user_info
                .get("family_name")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());

            let roles = user_info
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

            let groups = user_info
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

            let resource_access = user_info
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
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AuthServiceError::UserNotFound(error_text))
        }
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
}

pub type AuthServiceResult<T> = Result<T, AuthServiceError>;
