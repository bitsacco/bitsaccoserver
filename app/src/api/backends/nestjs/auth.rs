use async_trait::async_trait;

use crate::api::{
    errors::ApiResult,
    traits::AuthApi,
    types::{
        auth::{
            AuthRequest, AuthResponse, LoginRequest, LogoutResponse, NestJsAuthResponse,
            RecoverRequest, RefreshTokenRequest, RegisterRequest, RevokeTokenRequest,
            RevokeTokenResponse, TokensResponse, VerifyRequest,
        },
        common::{Nostr, Phone, Profile, Role},
        user::User,
    },
};
use chrono::Utc;
use uuid::Uuid;

use super::client::NestJsClient;

#[derive(Clone)]
pub struct NestJsAuthApi {
    client: NestJsClient,
}

impl NestJsAuthApi {
    pub fn new(client: NestJsClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AuthApi for NestJsAuthApi {
    async fn login(&self, request: LoginRequest) -> ApiResult<AuthResponse> {
        let req = self.client.post("/auth/login");
        let nestjs_response: NestJsAuthResponse = self.client.send_json(req, &request).await?;

        // Convert NestJS response to standard AuthResponse
        let user_id = Uuid::parse_str(&nestjs_response.user.id).map_err(|e| {
            crate::api::errors::ApiError::Serialization {
                message: format!("Invalid UUID: {}", e),
            }
        })?;

        // Convert roles from numbers to enum
        let roles: Result<Vec<Role>, _> = nestjs_response
            .user
            .roles
            .into_iter()
            .map(|r| {
                match r {
                    0 => Ok(Role::Member),
                    1 => Ok(Role::Admin),
                    2 => Ok(Role::SuperAdmin),
                    3 => Ok(Role::SuperAdmin), // Role 3 might be another super admin variant
                    _ => Err(crate::api::errors::ApiError::Serialization {
                        message: format!("Unknown role: {}", r),
                    }),
                }
            })
            .collect();
        let roles = roles?;

        let user = User {
            id: user_id,
            phone: Some(Phone {
                number: nestjs_response.user.phone.number,
            }),
            nostr: Some(Nostr {
                npub: nestjs_response.user.nostr.npub,
            }),
            profile: Some(Profile {
                name: Some(nestjs_response.user.profile.name),
                avatar_url: Some(nestjs_response.user.profile.avatar_url),
            }),
            roles,
            verified: nestjs_response.user.phone.verified, // Use phone verification status
            created_at: Utc::now(), // NestJS doesn't provide these, use current time
            updated_at: Utc::now(),
        };

        Ok(AuthResponse {
            user,
            authenticated: nestjs_response.authenticated,
            access_token: Some(nestjs_response.access_token),
            refresh_token: Some(nestjs_response.refresh_token),
        })
    }

    async fn register(&self, request: RegisterRequest) -> ApiResult<AuthResponse> {
        let req = self.client.post("/auth/register");
        self.client.send_json(req, &request).await
    }

    async fn verify(&self, request: VerifyRequest) -> ApiResult<AuthResponse> {
        let req = self.client.post("/auth/verify");
        self.client.send_json(req, &request).await
    }

    async fn authenticate(&self, request: AuthRequest) -> ApiResult<AuthResponse> {
        let req = self
            .client
            .post("/auth/authenticate")
            .header("Authorization", format!("Bearer {}", request.access_token));
        self.client.send(req).await
    }

    async fn recover(&self, request: RecoverRequest) -> ApiResult<AuthResponse> {
        let req = self.client.post("/auth/recover");
        self.client.send_json(req, &request).await
    }

    async fn refresh_token(&self, request: RefreshTokenRequest) -> ApiResult<TokensResponse> {
        let req = self.client.post("/auth/refresh").header(
            "Cookie",
            format!(
                "RefreshToken={}; Secure; HttpOnly; SameSite=Strict",
                request.refresh_token
            ),
        );
        self.client.send(req).await
    }

    async fn revoke_token(&self, request: RevokeTokenRequest) -> ApiResult<RevokeTokenResponse> {
        let req = self.client.post("/auth/logout").header(
            "Cookie",
            format!(
                "RefreshToken={}; Secure; HttpOnly; SameSite=Strict",
                request.refresh_token
            ),
        );
        self.client.send(req).await
    }

    async fn logout(&self, request: RevokeTokenRequest) -> ApiResult<LogoutResponse> {
        let req = self.client.post("/auth/logout").header(
            "Cookie",
            format!(
                "RefreshToken={}; Secure; HttpOnly; SameSite=Strict",
                request.refresh_token
            ),
        );

        let response: RevokeTokenResponse = self.client.send(req).await?;

        Ok(LogoutResponse {
            success: response.success,
            message: response.message,
        })
    }
}
