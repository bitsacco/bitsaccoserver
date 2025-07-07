pub mod audit_logs;
pub mod group_memberships;
pub mod groups;
pub mod members;
pub mod share_offers;
pub mod shares;

use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct Repositories {
    pub database: Arc<DatabaseConnection>,
    pub groups: groups::GroupRepository,
    pub members: members::MemberRepository,
    pub group_memberships: group_memberships::GroupMembershipRepository,
    pub share_offers: share_offers::ShareOfferRepository,
    pub shares: shares::ShareRepository,
    pub audit_logs: audit_logs::AuditLogRepository,
}

impl Repositories {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            database: db.clone(),
            groups: groups::GroupRepository::new(db.clone()),
            members: members::MemberRepository::new(db.clone()),
            group_memberships: group_memberships::GroupMembershipRepository::new(db.clone()),
            share_offers: share_offers::ShareOfferRepository::new(db.clone()),
            shares: shares::ShareRepository::new(db.clone()),
            audit_logs: audit_logs::AuditLogRepository::new(db),
        }
    }
}

// Common error type for repository operations
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("Entity not found")]
    NotFound,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Conflict error: {0}")]
    Conflict(String),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;
