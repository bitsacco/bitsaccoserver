// Graceful degradation implementation for Rust backend
// This provides friendly error handling when Rust backend is configured but not implemented

use crate::api::{
    config::ApiConfig,
    errors::{ApiError, ApiResult},
    traits::{AuthApi, GroupsApi, UsersApi, WalletsApi},
    types::{
        AuthRequest, AuthResponse, FindUserRequest, LoginRequest, LogoutResponse, RecoverRequest,
        RefreshTokenRequest, RegisterRequest, RevokeTokenRequest, RevokeTokenResponse,
        TokensResponse, UpdateUserRequest, User, VerifyRequest,
    },
};
use async_trait::async_trait;

pub struct RustBackend {
    #[allow(dead_code)]
    base_url: String,
    friendly_error_message: String,
}

impl RustBackend {
    pub fn new(config: &ApiConfig) -> ApiResult<Self> {
        // Instead of panicking, create a backend that provides helpful error messages
        Ok(Self {
            base_url: config.base_url.clone(),
            friendly_error_message: format!(
                "Rust backend is not currently implemented. The application is configured to use \
                 the Rust backend (API_BACKEND=rust), but only the NestJS backend is supported at this time. \
                 \n\nTo use the application:\n\
                 1. Set API_BACKEND=nestjs in your environment\n\
                 2. Or remove the API_BACKEND environment variable (defaults to NestJS)\n\
                 3. Ensure your NestJS backend is running at: {}\n\n\
                 For technical support, please contact your system administrator.",
                std::env::var("NESTJS_API_URL").unwrap_or_else(|_| "http://localhost:4000".to_string())
            ),
        })
    }

    /// Provides a user-friendly error message explaining the Rust backend limitation
    pub fn get_friendly_error(&self) -> ApiError {
        ApiError::Server {
            message: self.friendly_error_message.clone(),
        }
    }

    /// Check if this backend instance represents a graceful degradation scenario
    pub fn is_graceful_degradation(&self) -> bool {
        true // All Rust backend instances are currently graceful degradations
    }
}

// Implement AuthApi trait with graceful degradation
#[async_trait]
impl AuthApi for RustBackend {
    async fn login(&self, _request: LoginRequest) -> ApiResult<AuthResponse> {
        Err(self.get_friendly_error())
    }

    async fn register(&self, _request: RegisterRequest) -> ApiResult<AuthResponse> {
        Err(self.get_friendly_error())
    }

    async fn verify(&self, _request: VerifyRequest) -> ApiResult<AuthResponse> {
        Err(self.get_friendly_error())
    }

    async fn authenticate(&self, _request: AuthRequest) -> ApiResult<AuthResponse> {
        Err(self.get_friendly_error())
    }

    async fn recover(&self, _request: RecoverRequest) -> ApiResult<AuthResponse> {
        Err(self.get_friendly_error())
    }

    async fn refresh_token(&self, _request: RefreshTokenRequest) -> ApiResult<TokensResponse> {
        Err(self.get_friendly_error())
    }

    async fn revoke_token(&self, _request: RevokeTokenRequest) -> ApiResult<RevokeTokenResponse> {
        Err(self.get_friendly_error())
    }

    async fn logout(&self, _request: RevokeTokenRequest) -> ApiResult<LogoutResponse> {
        Err(self.get_friendly_error())
    }
}

// Implement UsersApi trait with graceful degradation
#[async_trait]
impl UsersApi for RustBackend {
    async fn get_user(&self, _user_id: uuid::Uuid) -> ApiResult<User> {
        Err(self.get_friendly_error())
    }

    async fn find_user(&self, _request: FindUserRequest) -> ApiResult<Option<User>> {
        Err(self.get_friendly_error())
    }

    async fn get_users(
        &self,
        _pagination: crate::api::types::PaginationQuery,
    ) -> ApiResult<crate::api::types::PaginatedResponse<User>> {
        Err(self.get_friendly_error())
    }

    async fn search_users(
        &self,
        _search: crate::api::types::SearchQuery,
        _pagination: crate::api::types::PaginationQuery,
    ) -> ApiResult<crate::api::types::PaginatedResponse<User>> {
        Err(self.get_friendly_error())
    }

    async fn update_user(&self, _request: UpdateUserRequest) -> ApiResult<User> {
        Err(self.get_friendly_error())
    }

    async fn delete_user(&self, _user_id: uuid::Uuid) -> ApiResult<()> {
        Err(self.get_friendly_error())
    }
}

// Implement GroupsApi trait with graceful degradation
#[async_trait]
impl GroupsApi for RustBackend {
    async fn get_group(
        &self,
        _group_id: uuid::Uuid,
    ) -> ApiResult<crate::api::traits::groups::Group> {
        Err(self.get_friendly_error())
    }

    async fn get_groups(
        &self,
        _pagination: crate::api::types::PaginationQuery,
    ) -> ApiResult<crate::api::types::PaginatedResponse<crate::api::traits::groups::Group>> {
        Err(self.get_friendly_error())
    }

    async fn search_groups(
        &self,
        _search: crate::api::types::SearchQuery,
        _pagination: crate::api::types::PaginationQuery,
    ) -> ApiResult<crate::api::types::PaginatedResponse<crate::api::traits::groups::Group>> {
        Err(self.get_friendly_error())
    }

    async fn create_group(
        &self,
        _request: crate::api::traits::groups::CreateGroupRequest,
    ) -> ApiResult<crate::api::traits::groups::Group> {
        Err(self.get_friendly_error())
    }

    async fn update_group(
        &self,
        _group_id: uuid::Uuid,
        _request: crate::api::traits::groups::UpdateGroupRequest,
    ) -> ApiResult<crate::api::traits::groups::Group> {
        Err(self.get_friendly_error())
    }

    async fn delete_group(&self, _group_id: uuid::Uuid) -> ApiResult<()> {
        Err(self.get_friendly_error())
    }
}

// Implement WalletsApi trait with graceful degradation
#[async_trait]
impl WalletsApi for RustBackend {
    async fn get_wallet(
        &self,
        _wallet_id: uuid::Uuid,
    ) -> ApiResult<crate::api::traits::wallets::Wallet> {
        Err(self.get_friendly_error())
    }

    async fn get_user_wallets(
        &self,
        _user_id: uuid::Uuid,
    ) -> ApiResult<Vec<crate::api::traits::wallets::Wallet>> {
        Err(self.get_friendly_error())
    }

    async fn get_wallets(
        &self,
        _pagination: crate::api::types::PaginationQuery,
    ) -> ApiResult<crate::api::types::PaginatedResponse<crate::api::traits::wallets::Wallet>> {
        Err(self.get_friendly_error())
    }

    async fn create_wallet(
        &self,
        _request: crate::api::traits::wallets::CreateWalletRequest,
    ) -> ApiResult<crate::api::traits::wallets::Wallet> {
        Err(self.get_friendly_error())
    }

    async fn delete_wallet(&self, _wallet_id: uuid::Uuid) -> ApiResult<()> {
        Err(self.get_friendly_error())
    }

    async fn get_wallet_transactions(
        &self,
        _wallet_id: uuid::Uuid,
        _pagination: crate::api::types::PaginationQuery,
    ) -> ApiResult<
        crate::api::types::PaginatedResponse<crate::api::traits::wallets::WalletTransaction>,
    > {
        Err(self.get_friendly_error())
    }

    async fn get_wallet_balance(&self, _wallet_id: uuid::Uuid) -> ApiResult<u64> {
        Err(self.get_friendly_error())
    }
}
