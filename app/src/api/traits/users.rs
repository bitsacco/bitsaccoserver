use async_trait::async_trait;
use uuid::Uuid;

use crate::api::{
    errors::ApiResult,
    types::{
        FindUserRequest, PaginatedResponse, PaginationQuery, SearchQuery, UpdateUserRequest, User,
    },
};

#[async_trait]
pub trait UsersApi: Send + Sync {
    /// Get a user by ID
    async fn get_user(&self, user_id: Uuid) -> ApiResult<User>;

    /// Find a user by criteria
    async fn find_user(&self, request: FindUserRequest) -> ApiResult<Option<User>>;

    /// Get all users with pagination
    async fn get_users(&self, pagination: PaginationQuery) -> ApiResult<PaginatedResponse<User>>;

    /// Search users with filters
    async fn search_users(
        &self,
        search: SearchQuery,
        pagination: PaginationQuery,
    ) -> ApiResult<PaginatedResponse<User>>;

    /// Update a user
    async fn update_user(&self, request: UpdateUserRequest) -> ApiResult<User>;

    /// Delete a user
    async fn delete_user(&self, user_id: Uuid) -> ApiResult<()>;
}
