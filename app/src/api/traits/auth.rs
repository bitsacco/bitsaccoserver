use async_trait::async_trait;

use crate::api::{
    errors::ApiResult,
    types::{
        AuthRequest, AuthResponse, LoginRequest, LogoutResponse, RecoverRequest,
        RefreshTokenRequest, RegisterRequest, RevokeTokenRequest, RevokeTokenResponse,
        TokensResponse, VerifyRequest,
    },
};

#[async_trait]
pub trait AuthApi: Send + Sync {
    /// Authenticate a user with username and password
    async fn login(&self, request: LoginRequest) -> ApiResult<AuthResponse>;

    /// Register a new user
    async fn register(&self, request: RegisterRequest) -> ApiResult<AuthResponse>;

    /// Verify a user with OTP
    async fn verify(&self, request: VerifyRequest) -> ApiResult<AuthResponse>;

    /// Authenticate using an access token
    async fn authenticate(&self, request: AuthRequest) -> ApiResult<AuthResponse>;

    /// Recover user account
    async fn recover(&self, request: RecoverRequest) -> ApiResult<AuthResponse>;

    /// Refresh access token using refresh token
    async fn refresh_token(&self, request: RefreshTokenRequest) -> ApiResult<TokensResponse>;

    /// Revoke a refresh token
    async fn revoke_token(&self, request: RevokeTokenRequest) -> ApiResult<RevokeTokenResponse>;

    /// Logout and revoke tokens
    async fn logout(&self, request: RevokeTokenRequest) -> ApiResult<LogoutResponse>;
}
