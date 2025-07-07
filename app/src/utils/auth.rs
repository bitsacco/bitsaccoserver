use crate::middleware::auth::UserContext;
use axum::http::StatusCode;

/// Extract user ID from user context, returning UNAUTHORIZED if missing
pub fn require_user_id(user: &UserContext) -> Result<uuid::Uuid, StatusCode> {
    Ok(user.user_id)
}

/// Check if user has admin role
pub fn is_admin(user: &UserContext) -> bool {
    user.roles.contains(&"admin".to_string())
}

/// Check if user has specific role
pub fn has_role(user: &UserContext, role: &str) -> bool {
    user.roles.contains(&role.to_string())
}

/// Check if user has any of the specified roles
pub fn has_any_role(user: &UserContext, roles: &[&str]) -> bool {
    roles
        .iter()
        .any(|role| user.roles.contains(&role.to_string()))
}

/// Check if user can access resource (admin or specific role)
pub fn can_access_resource(user: &UserContext, required_role: &str) -> bool {
    is_admin(user) || has_role(user, required_role)
}

/// Require specific role or return FORBIDDEN
pub fn require_role(user: &UserContext, role: &str) -> Result<(), StatusCode> {
    if has_role(user, role) || is_admin(user) {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// Require any of the specified roles or return FORBIDDEN
pub fn require_any_role(user: &UserContext, roles: &[&str]) -> Result<(), StatusCode> {
    if has_any_role(user, roles) || is_admin(user) {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}
