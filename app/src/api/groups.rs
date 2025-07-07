use ::entity::{groups, prelude::*};
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    repositories::{Repositories, RepositoryError},
    services::Services,
};

// Helper function to convert RepositoryError to ApiError
fn repository_error_to_api_error(err: RepositoryError) -> ApiError {
    match err {
        RepositoryError::Database(db_err) => ApiError::Database(db_err),
        RepositoryError::NotFound => ApiError::NotFound,
        RepositoryError::Validation(msg) | RepositoryError::ValidationError(msg) => {
            ApiError::BadRequest(msg)
        }
        RepositoryError::Conflict(msg) => ApiError::BadRequest(msg),
    }
}

// Placeholder auth types until middleware is implemented
pub struct RequireAuth {
    pub user_id: Uuid,
}

impl RequireAuth {
    pub fn new() -> Self {
        Self {
            user_id: Uuid::new_v4(),
        }
    }
}

impl<S> axum::extract::FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(RequireAuth::new())
    }
}

use super::client::{ApiResponse, PaginatedResponse, PaginationQuery};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub group_type: groups::GroupType,
    pub parent_id: Option<Uuid>,
    pub sort_order: Option<i32>,
    pub settings: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub group_type: Option<groups::GroupType>,
    pub status: Option<groups::GroupStatus>,
    pub parent_id: Option<Uuid>,
    pub sort_order: Option<i32>,
    pub settings: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupFilters {
    pub group_type: Option<groups::GroupType>,
    pub status: Option<groups::GroupStatus>,
    pub parent_id: Option<Uuid>,
    pub search: Option<String>,
    #[serde(flatten)]
    pub pagination: PaginationQuery,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupResponse {
    #[serde(flatten)]
    pub group: groups::Model,
    pub member_count: Option<u64>,
    pub children_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupHierarchyResponse {
    #[serde(flatten)]
    pub group: groups::Model,
    pub children: Vec<GroupHierarchyResponse>,
    pub member_count: u64,
}

pub fn router(repositories: Repositories, services: Services) -> Router<()> {
    Router::new()
        .route("/", get(list_groups).post(create_group))
        .route(
            "/{id}",
            get(get_group).put(update_group).delete(delete_group),
        )
        .route("/{id}/members", get(get_group_members))
        .route("/{id}/shares", get(get_group_shares))
        .route("/{id}/children", get(get_group_children))
        .route("/hierarchy", get(get_group_hierarchy))
        .route("/search", get(search_groups))
        .with_state((repositories, services))
}

/// List groups with filtering and pagination
pub async fn list_groups(
    State((repositories, _services)): State<(Repositories, Services)>,
    Query(filters): Query<GroupFilters>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    let page = filters.pagination.page.unwrap_or(1);
    let limit = filters.pagination.limit.unwrap_or(25);
    let offset = (page - 1) * limit;

    let mut query = Groups::find();

    // Apply filters
    if let Some(ref group_type) = filters.group_type {
        query = query.filter(groups::Column::GroupType.eq(group_type.clone()));
    }

    if let Some(ref status) = filters.status {
        query = query.filter(groups::Column::Status.eq(status.clone()));
    }

    if let Some(parent_id) = filters.parent_id {
        query = query.filter(groups::Column::ParentId.eq(parent_id));
    }

    if let Some(search) = &filters.search {
        let search_term = format!("%{}%", search);
        query = query.filter(
            Condition::any()
                .add(groups::Column::Name.like(&search_term))
                .add(groups::Column::Description.like(&search_term)),
        );
    }

    // Apply pagination and sorting
    query = query
        .order_by_asc(groups::Column::Level)
        .order_by_asc(groups::Column::SortOrder)
        .order_by_asc(groups::Column::Name)
        .offset(offset as u64)
        .limit(limit as u64);

    let groups = query
        .all(&*repositories.database)
        .await
        .map_err(|e| ApiError::Database(e))?;

    // Get total count for pagination
    let mut count_query = Groups::find();
    if let Some(ref group_type) = filters.group_type {
        count_query = count_query.filter(groups::Column::GroupType.eq(group_type.clone()));
    }
    if let Some(ref status) = filters.status {
        count_query = count_query.filter(groups::Column::Status.eq(status.clone()));
    }
    if let Some(parent_id) = filters.parent_id {
        count_query = count_query.filter(groups::Column::ParentId.eq(parent_id));
    }
    if let Some(search) = &filters.search {
        let search_term = format!("%{}%", search);
        count_query = count_query.filter(
            Condition::any()
                .add(groups::Column::Name.like(&search_term))
                .add(groups::Column::Description.like(&search_term)),
        );
    }

    let total = count_query
        .count(&*repositories.database)
        .await
        .map_err(|e| ApiError::Database(e))?;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    // Convert to response format with additional metadata
    let mut group_responses = Vec::new();
    for group in groups {
        let member_count = repositories
            .group_memberships
            .count_by_group(group.id)
            .await
            .ok();

        let children_count = Groups::find()
            .filter(groups::Column::ParentId.eq(group.id))
            .count(&*repositories.database)
            .await
            .ok();

        group_responses.push(GroupResponse {
            group,
            member_count,
            children_count,
        });
    }

    let response = PaginatedResponse {
        data: group_responses,
        page,
        limit,
        total,
        total_pages,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Groups retrieved successfully".to_string()),
        errors: None,
    }))
}

/// Create a new group
pub async fn create_group(
    State((repositories, _services)): State<(Repositories, Services)>,
    user: RequireAuth,
    Json(request): Json<CreateGroupRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate parent exists if specified
    if let Some(parent_id) = request.parent_id {
        let parent = repositories
            .groups
            .find_by_id(parent_id)
            .await
            .map_err(repository_error_to_api_error)?;

        if parent.is_none() {
            return Err(ApiError::BadRequest("Parent group not found".to_string()));
        }

        let parent = parent.unwrap();
        if !parent.can_have_children() {
            return Err(ApiError::BadRequest(
                "Parent group cannot have children".to_string(),
            ));
        }
    }

    // Calculate level and path based on parent
    let (level, path) = if let Some(parent_id) = request.parent_id {
        let parent = repositories
            .groups
            .find_by_id(parent_id)
            .await
            .map_err(repository_error_to_api_error)?
            .unwrap();

        let level = parent.level + 1;
        let path = if let Some(parent_path) = parent.path {
            format!("{}/{}", parent_path, parent_id)
        } else {
            format!("/{}", parent_id)
        };
        (level, Some(path))
    } else {
        (0, None)
    };

    let group_id = Uuid::new_v4();
    let now = chrono::Utc::now().into();

    let group = groups::ActiveModel {
        id: Set(group_id),
        name: Set(request.name),
        description: Set(request.description),
        group_type: Set(request.group_type),
        status: Set(groups::GroupStatus::Pending),
        parent_id: Set(request.parent_id),
        path: Set(path),
        level: Set(level),
        sort_order: Set(request.sort_order),
        settings: Set(request.settings),
        metadata: Set(request.metadata),
        created_at: Set(now),
        updated_at: Set(now),
        created_by: Set(Some(user.user_id)),
        updated_by: Set(Some(user.user_id)),
    };

    let created_group = repositories
        .groups
        .create(group)
        .await
        .map_err(repository_error_to_api_error)?;

    let response = GroupResponse {
        group: created_group,
        member_count: Some(0),
        children_count: Some(0),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Group created successfully".to_string()),
        errors: None,
    }))
}

/// Get a specific group by ID
pub async fn get_group(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(id): Path<Uuid>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    let group = repositories
        .groups
        .find_by_id(id)
        .await
        .map_err(repository_error_to_api_error)?
        .ok_or(ApiError::NotFound)?;

    let member_count = repositories.group_memberships.count_by_group(id).await.ok();

    let children_count = Groups::find()
        .filter(groups::Column::ParentId.eq(id))
        .count(&*repositories.database)
        .await
        .ok();

    let response = GroupResponse {
        group,
        member_count,
        children_count,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Group retrieved successfully".to_string()),
        errors: None,
    }))
}

/// Update a group
pub async fn update_group(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(id): Path<Uuid>,
    user: RequireAuth,
    Json(request): Json<UpdateGroupRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if group exists
    let existing_group = repositories
        .groups
        .find_by_id(id)
        .await
        .map_err(repository_error_to_api_error)?
        .ok_or(ApiError::NotFound)?;

    // Validate parent change if specified
    if let Some(Some(new_parent_id)) = request.parent_id.as_ref().map(Some) {
        if *new_parent_id != existing_group.parent_id.unwrap_or_default() {
            let parent = repositories
                .groups
                .find_by_id(*new_parent_id)
                .await
                .map_err(repository_error_to_api_error)?
                .ok_or(ApiError::BadRequest("Parent group not found".to_string()))?;

            if !parent.can_have_children() {
                return Err(ApiError::BadRequest(
                    "Parent group cannot have children".to_string(),
                ));
            }

            // TODO: Check for circular references
        }
    }

    let mut active_model: groups::ActiveModel = existing_group.into();

    // Update fields if provided
    if let Some(name) = request.name {
        active_model.name = Set(name);
    }
    if let Some(description) = request.description {
        active_model.description = Set(Some(description));
    }
    if let Some(group_type) = request.group_type {
        active_model.group_type = Set(group_type);
    }
    if let Some(status) = request.status {
        active_model.status = Set(status);
    }
    if let Some(parent_id) = request.parent_id {
        active_model.parent_id = Set(Some(parent_id));
    }
    if let Some(sort_order) = request.sort_order {
        active_model.sort_order = Set(Some(sort_order));
    }
    if let Some(settings) = request.settings {
        active_model.settings = Set(Some(settings));
    }
    if let Some(metadata) = request.metadata {
        active_model.metadata = Set(Some(metadata));
    }

    active_model.updated_at = Set(chrono::Utc::now().into());
    active_model.updated_by = Set(Some(user.user_id));

    let updated_group = active_model
        .update(&*repositories.database)
        .await
        .map_err(|e| ApiError::Database(e))?;

    let response = GroupResponse {
        group: updated_group,
        member_count: None,
        children_count: None,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Group updated successfully".to_string()),
        errors: None,
    }))
}

/// Delete a group
pub async fn delete_group(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(id): Path<Uuid>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Check if group has children
    let children = Groups::find()
        .filter(groups::Column::ParentId.eq(id))
        .count(&*repositories.database)
        .await
        .map_err(|e| ApiError::Database(e))?;

    if children > 0 {
        return Err(ApiError::BadRequest(
            "Cannot delete group with child groups".to_string(),
        ));
    }

    // Check if group has members
    let member_count = repositories
        .group_memberships
        .count_by_group(id)
        .await
        .map_err(repository_error_to_api_error)?;

    if member_count > 0 {
        return Err(ApiError::BadRequest(
            "Cannot delete group with members".to_string(),
        ));
    }

    repositories.groups.delete(id).await.map_err(|e| match e {
        RepositoryError::NotFound => ApiError::NotFound,
        _ => ApiError::Database(DbErr::Custom("Failed to delete group".to_string())),
    })?;

    Ok(Json(ApiResponse::<()> {
        success: true,
        data: None,
        message: Some("Group deleted successfully".to_string()),
        errors: None,
    }))
}

/// Get group members
pub async fn get_group_members(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Verify group exists
    repositories
        .groups
        .find_by_id(id)
        .await
        .map_err(repository_error_to_api_error)?
        .ok_or(ApiError::NotFound)?;

    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(25);

    let members = repositories
        .group_memberships
        .find_members_by_group_paginated(id, page, limit)
        .await
        .map_err(repository_error_to_api_error)?;

    let total = repositories
        .group_memberships
        .count_by_group(id)
        .await
        .map_err(repository_error_to_api_error)?;

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let response = PaginatedResponse {
        data: members,
        page,
        limit,
        total,
        total_pages,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Group members retrieved successfully".to_string()),
        errors: None,
    }))
}

/// Get group shares
pub async fn get_group_shares(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Verify group exists
    repositories
        .groups
        .find_by_id(id)
        .await
        .map_err(repository_error_to_api_error)?
        .ok_or(ApiError::NotFound)?;

    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(25);
    let offset = (page - 1) * limit;

    let shares = Shares::find()
        .filter(::entity::shares::Column::OwnerId.eq(id))
        .filter(::entity::shares::Column::OwnerType.eq(::entity::shares::OwnerType::Group))
        .offset(offset as u64)
        .limit(limit as u64)
        .all(&*repositories.database)
        .await
        .map_err(|e| ApiError::Database(e))?;

    let total = Shares::find()
        .filter(::entity::shares::Column::OwnerId.eq(id))
        .filter(::entity::shares::Column::OwnerType.eq(::entity::shares::OwnerType::Group))
        .count(&*repositories.database)
        .await
        .map_err(|e| ApiError::Database(e))?;

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let response = PaginatedResponse {
        data: shares,
        page,
        limit,
        total,
        total_pages,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Group shares retrieved successfully".to_string()),
        errors: None,
    }))
}

/// Get group children
pub async fn get_group_children(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(id): Path<Uuid>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Verify group exists
    repositories
        .groups
        .find_by_id(id)
        .await
        .map_err(repository_error_to_api_error)?
        .ok_or(ApiError::NotFound)?;

    let children = repositories
        .groups
        .find_children(id)
        .await
        .map_err(repository_error_to_api_error)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(children),
        message: Some("Group children retrieved successfully".to_string()),
        errors: None,
    }))
}

/// Get full group hierarchy
pub async fn get_group_hierarchy(
    State((repositories, _services)): State<(Repositories, Services)>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    let all_groups = repositories
        .groups
        .find_hierarchy_from_root()
        .await
        .map_err(repository_error_to_api_error)?;

    // Build hierarchy structure
    let hierarchy = build_hierarchy(&repositories, all_groups).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(hierarchy),
        message: Some("Group hierarchy retrieved successfully".to_string()),
        errors: None,
    }))
}

/// Search groups
pub async fn search_groups(
    State((repositories, _services)): State<(Repositories, Services)>,
    Query(filters): Query<GroupFilters>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    let search_term = filters.search.as_deref().unwrap_or("");
    if search_term.is_empty() {
        return Err(ApiError::BadRequest("Search query is required".to_string()));
    }

    let groups = repositories
        .groups
        .search(search_term)
        .await
        .map_err(repository_error_to_api_error)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(groups),
        message: Some("Groups search completed successfully".to_string()),
        errors: None,
    }))
}

// Helper function to build hierarchy
async fn build_hierarchy(
    repositories: &Repositories,
    groups: Vec<groups::Model>,
) -> Result<Vec<GroupHierarchyResponse>, ApiError> {
    let mut hierarchy = Vec::new();
    let mut group_map = std::collections::HashMap::new();

    // First, get member counts for all groups
    for group in &groups {
        let member_count = repositories
            .group_memberships
            .count_by_group(group.id)
            .await
            .unwrap_or(0);

        group_map.insert(group.id, (group.clone(), member_count, Vec::new()));
    }

    // Build parent-child relationships
    for group in &groups {
        if let Some(parent_id) = group.parent_id {
            // Clone the group id to avoid borrowing issues
            let group_id = group.id;
            if let Some((child_group, member_count, child_children)) = group_map.remove(&group_id) {
                if let Some((_, _, ref mut children)) = group_map.get_mut(&parent_id) {
                    children.push(GroupHierarchyResponse {
                        group: child_group,
                        children: child_children,
                        member_count,
                    });
                }
            }
        }
    }

    // Collect root groups
    for (_, (group, member_count, children)) in group_map {
        if group.parent_id.is_none() {
            hierarchy.push(GroupHierarchyResponse {
                group,
                children,
                member_count,
            });
        }
    }

    Ok(hierarchy)
}

// Error handling
#[derive(Debug)]
pub enum ApiError {
    Database(DbErr),
    NotFound,
    BadRequest(String),
    Unauthorized,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            ApiError::Database(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            ApiError::NotFound => (
                axum::http::StatusCode::NOT_FOUND,
                "Resource not found".to_string(),
            ),
            ApiError::BadRequest(msg) => (axum::http::StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Unauthorized => (
                axum::http::StatusCode::UNAUTHORIZED,
                "Unauthorized".to_string(),
            ),
        };

        let response = ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(message.clone()),
            errors: Some(vec![message]),
        };

        (status, Json(response)).into_response()
    }
}
