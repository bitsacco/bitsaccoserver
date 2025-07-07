use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents an audit log entry for tracking data changes.
///
/// This entity provides a comprehensive audit trail for all data modifications
/// in the system, capturing what changed, when, who made the change, and
/// contextual information about the modification.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "audit_logs")]
pub struct Model {
    /// Unique identifier for this audit log entry.
    ///
    /// This is a UUID that serves as the primary key and is automatically
    /// generated when a new audit log entry is created.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Name of the database table that was modified.
    ///
    /// String identifying which table in the database experienced the change.
    /// Used for categorizing and filtering audit logs by entity type.
    /// Example: "members", "groups", "shares", "group_memberships"
    pub table_name: String,

    /// Identifier of the specific record that was modified.
    ///
    /// UUID of the record that was inserted, updated, or deleted.
    /// Links the audit log entry to the specific entity instance that changed.
    pub record_id: Uuid,

    /// Type of database operation that occurred.
    ///
    /// String indicating the kind of change that was made:
    /// - "INSERT": New record was created
    /// - "UPDATE": Existing record was modified
    /// - "DELETE": Record was removed
    /// Automatically captured by database triggers.
    pub operation: String,

    /// Previous values before the change.
    ///
    /// JSON representation of the record's state before modification.
    /// Null for INSERT operations since there were no previous values.
    /// Contains the complete previous record for UPDATE and DELETE operations.
    pub old_values: Option<Json>,

    /// New values after the change.
    ///
    /// JSON representation of the record's state after modification.
    /// Contains the complete new record for INSERT and UPDATE operations.
    /// Null for DELETE operations since the record no longer exists.
    pub new_values: Option<Json>,

    /// ID of the user who made the change.
    ///
    /// Optional reference to the user who performed the operation.
    /// May be null for system-generated changes or when user context
    /// is not available. Used for accountability and access tracking.
    pub changed_by: Option<Uuid>,

    /// Timestamp when the change occurred.
    ///
    /// Exact date and time when the database operation was performed.
    /// Automatically captured by database triggers and stored in UTC
    /// for consistent temporal ordering across different deployments.
    pub changed_at: DateTimeWithTimeZone,

    /// IP address of the client that initiated the change.
    ///
    /// Optional field capturing the network address of the client
    /// that made the request leading to this change. Useful for
    /// security analysis and identifying suspicious activity patterns.
    pub ip_address: Option<String>,

    /// User agent string of the client application.
    ///
    /// Optional field capturing the browser or application identifier
    /// that made the request. Helps in understanding the context and
    /// source of changes, particularly useful for web-based modifications.
    pub user_agent: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
