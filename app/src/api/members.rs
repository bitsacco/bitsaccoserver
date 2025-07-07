use ::entity::{members, prelude::*};
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
pub struct CreateMemberRequest {
    pub member_number: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    // Note: address, id_number, date_of_birth, employment_info, emergency_contact not in current entity
    // pub address: Option<String>,
    // pub id_number: Option<String>,
    // pub date_of_birth: Option<chrono::NaiveDate>,
    // pub employment_info: Option<serde_json::Value>,
    // pub emergency_contact: Option<serde_json::Value>,
    // pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMemberRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<members::MemberStatus>,
    // Note: address, id_number, date_of_birth, employment_info, emergency_contact not in current entity
    // pub address: Option<String>,
    // pub id_number: Option<String>,
    // pub date_of_birth: Option<chrono::NaiveDate>,
    // pub employment_info: Option<serde_json::Value>,
    // pub emergency_contact: Option<serde_json::Value>,
    // pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberFilters {
    pub status: Option<members::MemberStatus>,
    pub search: Option<String>,
    pub group_id: Option<Uuid>,
    #[serde(flatten)]
    pub pagination: PaginationQuery,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberResponse {
    #[serde(flatten)]
    pub member: members::Model,
    pub groups: Option<Vec<::entity::groups::Model>>,
    pub shares_count: Option<u64>,
    pub total_shares_value: Option<rust_decimal::Decimal>,
}

pub fn router(repositories: Repositories, services: Services) -> Router<()> {
    Router::new()
        .route("/", get(list_members).post(create_member))
        .route(
            "/{id}",
            get(get_member).put(update_member).delete(delete_member),
        )
        .route("/{id}/groups", get(get_member_groups))
        .route("/{id}/shares", get(get_member_shares))
        .route("/{id}/transactions", get(get_member_transactions))
        .route("/search", get(search_members))
        .route(
            "/by-member-number/{member_number}",
            get(get_member_by_number),
        )
        .with_state((repositories, services))
}

/// List members with filtering and pagination
pub async fn list_members(
    State((repositories, _services)): State<(Repositories, Services)>,
    Query(filters): Query<MemberFilters>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    let page = filters.pagination.page.unwrap_or(1);
    let limit = filters.pagination.limit.unwrap_or(25);
    let offset = (page - 1) * limit;

    let mut query = Members::find();

    // Apply filters
    if let Some(ref status) = filters.status {
        query = query.filter(members::Column::Status.eq(status.clone()));
    }

    if let Some(search) = &filters.search {
        let search_term = format!("%{}%", search);
        query = query.filter(
            Condition::any()
                .add(members::Column::Name.like(&search_term))
                .add(members::Column::Email.like(&search_term))
                .add(members::Column::MemberNumber.like(&search_term))
                .add(members::Column::Phone.like(&search_term)),
        );
    }

    // Filter by group membership if specified
    if let Some(group_id) = filters.group_id {
        query = query
            .inner_join(::entity::group_memberships::Entity)
            .filter(::entity::group_memberships::Column::GroupId.eq(group_id));
    }

    // Apply pagination and sorting
    query = query
        .order_by_asc(members::Column::Name)
        .offset(offset as u64)
        .limit(limit as u64);

    let members = query
        .all(&*repositories.database)
        .await
        .map_err(|e| ApiError::Database(e))?;

    // Get total count for pagination
    let mut count_query = Members::find();
    if let Some(ref status) = filters.status {
        count_query = count_query.filter(members::Column::Status.eq(status.clone()));
    }
    if let Some(search) = &filters.search {
        let search_term = format!("%{}%", search);
        count_query = count_query.filter(
            Condition::any()
                .add(members::Column::Name.like(&search_term))
                .add(members::Column::Email.like(&search_term))
                .add(members::Column::MemberNumber.like(&search_term))
                .add(members::Column::Phone.like(&search_term)),
        );
    }
    if let Some(group_id) = filters.group_id {
        count_query = count_query
            .inner_join(::entity::group_memberships::Entity)
            .filter(::entity::group_memberships::Column::GroupId.eq(group_id));
    }

    let total = count_query
        .count(&*repositories.database)
        .await
        .map_err(|e| ApiError::Database(e))?;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    // Convert to response format with additional metadata
    let mut member_responses = Vec::new();
    for member in members {
        let shares_count = Shares::find()
            .filter(::entity::shares::Column::OwnerId.eq(member.id))
            .filter(::entity::shares::Column::OwnerType.eq(::entity::shares::OwnerType::Member))
            .count(&*repositories.database)
            .await
            .ok();

        member_responses.push(MemberResponse {
            member,
            groups: None, // Will be loaded separately if needed
            shares_count,
            total_shares_value: None, // Will be calculated if needed
        });
    }

    let response = PaginatedResponse {
        data: member_responses,
        page,
        limit,
        total,
        total_pages,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Members retrieved successfully".to_string()),
        errors: None,
    }))
}

/// Create a new member
pub async fn create_member(
    State((repositories, _services)): State<(Repositories, Services)>,
    user: RequireAuth,
    Json(request): Json<CreateMemberRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if member number is unique
    if let Some(_existing) = repositories
        .members
        .find_by_member_number(&request.member_number)
        .await
        .map_err(repository_error_to_api_error)?
    {
        return Err(ApiError::BadRequest(
            "Member number already exists".to_string(),
        ));
    }

    // Check if email is unique (if provided)
    if let Some(ref email) = request.email {
        if let Some(_existing) = repositories
            .members
            .find_by_email(email)
            .await
            .map_err(repository_error_to_api_error)?
        {
            return Err(ApiError::BadRequest("Email already exists".to_string()));
        }
    }

    let member_id = Uuid::new_v4();
    let now = chrono::Utc::now().into();

    let member = members::ActiveModel {
        id: Set(member_id),
        member_number: Set(request.member_number),
        name: Set(request.name),
        email: Set(request.email),
        phone: Set(request.phone),
        status: Set(members::MemberStatus::Pending),
        keycloak_user_id: Set(None),
        profile: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        created_by: Set(Some(user.user_id)),
        updated_by: Set(Some(user.user_id)),
    };

    let created_member = repositories
        .members
        .create(member)
        .await
        .map_err(repository_error_to_api_error)?;

    let response = MemberResponse {
        member: created_member,
        groups: Some(Vec::new()),
        shares_count: Some(0),
        total_shares_value: None,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Member created successfully".to_string()),
        errors: None,
    }))
}

/// Get a specific member by ID
pub async fn get_member(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(id): Path<Uuid>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    let member = repositories
        .members
        .find_by_id(id)
        .await
        .map_err(repository_error_to_api_error)?
        .ok_or(ApiError::NotFound)?;

    // Get member's groups
    let groups = repositories
        .group_memberships
        .find_groups_by_member(id)
        .await
        .map_err(repository_error_to_api_error)?;

    // Get member's shares count and total value
    let shares = Shares::find()
        .filter(::entity::shares::Column::OwnerId.eq(id))
        .filter(::entity::shares::Column::OwnerType.eq(::entity::shares::OwnerType::Member))
        .all(&*repositories.database)
        .await
        .map_err(|e| ApiError::Database(e))?;

    let shares_count = shares.len() as u64;
    let total_shares_value = shares
        .iter()
        .map(|s| s.total_value)
        .fold(rust_decimal::Decimal::ZERO, |acc, val| acc + val);

    let response = MemberResponse {
        member,
        groups: Some(groups),
        shares_count: Some(shares_count),
        total_shares_value: Some(total_shares_value),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Member retrieved successfully".to_string()),
        errors: None,
    }))
}

/// Get member by member number
pub async fn get_member_by_number(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(member_number): Path<String>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    let member = repositories
        .members
        .find_by_member_number(&member_number)
        .await
        .map_err(repository_error_to_api_error)?
        .ok_or(ApiError::NotFound)?;

    let response = MemberResponse {
        member,
        groups: None,
        shares_count: None,
        total_shares_value: None,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Member retrieved successfully".to_string()),
        errors: None,
    }))
}

/// Update a member
pub async fn update_member(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(id): Path<Uuid>,
    user: RequireAuth,
    Json(request): Json<UpdateMemberRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if member exists
    let existing_member = repositories
        .members
        .find_by_id(id)
        .await
        .map_err(repository_error_to_api_error)?
        .ok_or(ApiError::NotFound)?;

    // Check email uniqueness if changed
    if let Some(ref new_email) = request.email {
        if existing_member.email.as_ref() != Some(new_email) {
            if let Some(_existing) = repositories
                .members
                .find_by_email(new_email)
                .await
                .map_err(repository_error_to_api_error)?
            {
                return Err(ApiError::BadRequest("Email already exists".to_string()));
            }
        }
    }

    let mut active_model: members::ActiveModel = existing_member.into();

    // Update fields if provided
    if let Some(name) = request.name {
        active_model.name = Set(name);
    }
    if let Some(email) = request.email {
        active_model.email = Set(Some(email));
    }
    if let Some(phone) = request.phone {
        active_model.phone = Set(Some(phone));
    }
    if let Some(status) = request.status {
        active_model.status = Set(status);
    }
    // Note: Removed non-existent fields (address, id_number, date_of_birth, employment_info, emergency_contact, metadata)
    // These would be handled through the profile JSON field if needed

    active_model.updated_at = Set(chrono::Utc::now().into());
    active_model.updated_by = Set(Some(user.user_id));

    let updated_member = active_model
        .update(&*repositories.database)
        .await
        .map_err(|e| ApiError::Database(e))?;

    let response = MemberResponse {
        member: updated_member,
        groups: None,
        shares_count: None,
        total_shares_value: None,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Member updated successfully".to_string()),
        errors: None,
    }))
}

/// Delete a member
pub async fn delete_member(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(id): Path<Uuid>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Check if member has group memberships
    let memberships = repositories
        .group_memberships
        .find_by_member(id)
        .await
        .map_err(repository_error_to_api_error)?;

    if !memberships.is_empty() {
        return Err(ApiError::BadRequest(
            "Cannot delete member with active group memberships".to_string(),
        ));
    }

    // Check if member has shares
    let shares_count = Shares::find()
        .filter(::entity::shares::Column::OwnerId.eq(id))
        .filter(::entity::shares::Column::OwnerType.eq(::entity::shares::OwnerType::Member))
        .count(&*repositories.database)
        .await
        .map_err(|e| ApiError::Database(e))?;

    if shares_count > 0 {
        return Err(ApiError::BadRequest(
            "Cannot delete member with shares".to_string(),
        ));
    }

    repositories.members.delete(id).await.map_err(|e| match e {
        RepositoryError::NotFound => ApiError::NotFound,
        _ => ApiError::Database(DbErr::Custom("Failed to delete member".to_string())),
    })?;

    Ok(Json(ApiResponse::<()> {
        success: true,
        data: None,
        message: Some("Member deleted successfully".to_string()),
        errors: None,
    }))
}

/// Get member groups
pub async fn get_member_groups(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(id): Path<Uuid>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Verify member exists
    repositories
        .members
        .find_by_id(id)
        .await
        .map_err(repository_error_to_api_error)?
        .ok_or(ApiError::NotFound)?;

    let groups = repositories
        .group_memberships
        .find_groups_by_member(id)
        .await
        .map_err(repository_error_to_api_error)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(groups),
        message: Some("Member groups retrieved successfully".to_string()),
        errors: None,
    }))
}

/// Get member shares
pub async fn get_member_shares(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Verify member exists
    repositories
        .members
        .find_by_id(id)
        .await
        .map_err(repository_error_to_api_error)?
        .ok_or(ApiError::NotFound)?;

    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(25);
    let offset = (page - 1) * limit;

    let shares = Shares::find()
        .filter(::entity::shares::Column::OwnerId.eq(id))
        .filter(::entity::shares::Column::OwnerType.eq(::entity::shares::OwnerType::Member))
        .offset(offset as u64)
        .limit(limit as u64)
        .all(&*repositories.database)
        .await
        .map_err(|e| ApiError::Database(e))?;

    let total = Shares::find()
        .filter(::entity::shares::Column::OwnerId.eq(id))
        .filter(::entity::shares::Column::OwnerType.eq(::entity::shares::OwnerType::Member))
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
        message: Some("Member shares retrieved successfully".to_string()),
        errors: None,
    }))
}

/// Get member transactions
pub async fn get_member_transactions(
    State((repositories, _services)): State<(Repositories, Services)>,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Verify member exists
    repositories
        .members
        .find_by_id(id)
        .await
        .map_err(repository_error_to_api_error)?
        .ok_or(ApiError::NotFound)?;

    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(25);
    let _offset = (page - 1) * limit;

    // For now, return empty transactions since share_transactions entity doesn't exist
    let transactions: Vec<serde_json::Value> = vec![];
    let total: u64 = 0;

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let response = PaginatedResponse {
        data: transactions,
        page,
        limit,
        total,
        total_pages,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Member transactions retrieved successfully".to_string()),
        errors: None,
    }))
}

/// Search members
pub async fn search_members(
    State((repositories, _services)): State<(Repositories, Services)>,
    Query(filters): Query<MemberFilters>,
    _user: RequireAuth,
) -> Result<impl IntoResponse, ApiError> {
    let search_term = filters.search.as_deref().unwrap_or("");
    if search_term.is_empty() {
        return Err(ApiError::BadRequest("Search query is required".to_string()));
    }

    let members = repositories
        .members
        .search(search_term)
        .await
        .map_err(repository_error_to_api_error)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(members),
        message: Some("Members search completed successfully".to_string()),
        errors: None,
    }))
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
