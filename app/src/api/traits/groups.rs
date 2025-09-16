use async_trait::async_trait;
use uuid::Uuid;

use crate::api::{
    errors::ApiResult,
    types::{PaginatedResponse, PaginationQuery, SearchQuery},
};

// Placeholder group type - will be expanded based on actual requirements
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[async_trait]
pub trait GroupsApi: Send + Sync {
    /// Get a group by ID
    async fn get_group(&self, group_id: Uuid) -> ApiResult<Group>;

    /// Get all groups with pagination
    async fn get_groups(&self, pagination: PaginationQuery) -> ApiResult<PaginatedResponse<Group>>;

    /// Search groups with filters
    async fn search_groups(
        &self,
        search: SearchQuery,
        pagination: PaginationQuery,
    ) -> ApiResult<PaginatedResponse<Group>>;

    /// Create a new group
    async fn create_group(&self, request: CreateGroupRequest) -> ApiResult<Group>;

    /// Update a group
    async fn update_group(&self, group_id: Uuid, request: UpdateGroupRequest) -> ApiResult<Group>;

    /// Delete a group
    async fn delete_group(&self, group_id: Uuid) -> ApiResult<()>;
}
