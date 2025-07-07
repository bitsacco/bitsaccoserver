use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a member entity in the system.
///
/// Members are individual users who can join groups, own shares, and participate
/// in various activities within the platform. Each member has a unique identifier
/// and member number for tracking and reference purposes.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "members")]
pub struct Model {
    /// Unique identifier for the member.
    ///
    /// This is a UUID that serves as the primary key and is automatically
    /// generated when a new member is created.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Unique member number for identification.
    ///
    /// A human-readable identifier that is unique across all members.
    /// This is typically used for display purposes and member lookup.
    /// Example: "MEM001", "MB-2024-001"
    #[sea_orm(unique)]
    pub member_number: String,

    /// Full name of the member.
    ///
    /// The complete name of the member, combining first and last names
    /// or any preferred display name format.
    /// Example: "John Doe", "Jane Smith"
    pub name: String,

    /// Email address of the member.
    ///
    /// Optional email address used for communication and notifications.
    /// If provided, should be a valid email format.
    /// Example: "john.doe@example.com"
    pub email: Option<String>,

    /// Phone number of the member.
    ///
    /// Optional phone number for contact purposes.
    /// Can include country codes and various formats.
    /// Example: "+1-555-0123", "0712345678"
    pub phone: Option<String>,

    /// Current status of the member.
    ///
    /// Indicates the member's current state in the system:
    /// - Active: Member is active and can participate in all activities
    /// - Inactive: Member account is temporarily disabled
    /// - Suspended: Member account is suspended due to violations
    /// - Pending: New member awaiting approval or verification
    /// - Archived: Former member whose account is archived
    pub status: MemberStatus,

    /// Keycloak user identifier.
    ///
    /// Optional identifier linking this member to a Keycloak user account
    /// for authentication and single sign-on purposes.
    /// This enables integration with external identity providers.
    pub keycloak_user_id: Option<String>,

    /// Additional profile information.
    ///
    /// JSON field containing flexible profile data that doesn't fit
    /// into structured fields. Can include preferences, settings,
    /// additional contact information, or custom attributes.
    /// Example: {"preferences": {"language": "en"}, "avatar_url": "..."}
    pub profile: Option<Json>,

    /// Timestamp when the member was created.
    ///
    /// Automatically set when the member record is first inserted.
    /// Stored in UTC timezone for consistency across different deployments.
    pub created_at: DateTimeWithTimeZone,

    /// Timestamp when the member was last updated.
    ///
    /// Automatically updated whenever the member record is modified.
    /// Stored in UTC timezone for consistency across different deployments.
    pub updated_at: DateTimeWithTimeZone,

    /// ID of the user who created this member.
    ///
    /// Optional reference to the user (typically an admin or the member themselves)
    /// who created this member record. Used for audit trail purposes.
    pub created_by: Option<Uuid>,

    /// ID of the user who last updated this member.
    ///
    /// Optional reference to the user who last modified this member record.
    /// Used for audit trail and tracking changes.
    pub updated_by: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::group_memberships::Entity")]
    GroupMemberships,
}

impl Related<super::group_memberships::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GroupMemberships.def()
    }
}

impl Related<super::groups::Entity> for Entity {
    fn to() -> RelationDef {
        super::group_memberships::Relation::Group.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::group_memberships::Relation::Member.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Check if the member is active and can perform operations
    pub fn is_active(&self) -> bool {
        self.status == MemberStatus::Active
    }

    /// Check if the member can be activated
    pub fn can_be_activated(&self) -> bool {
        matches!(self.status, MemberStatus::Inactive | MemberStatus::Pending)
    }

    /// Check if the member is in a state that allows read operations
    pub fn can_read(&self) -> bool {
        !matches!(
            self.status,
            MemberStatus::Archived | MemberStatus::Suspended
        )
    }
}

/// Enumeration representing the various states a member can be in.
///
/// This enum defines the lifecycle states of a member account and determines
/// what actions and operations the member can perform within the system.
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "member_status")]
pub enum MemberStatus {
    /// Member account is active and fully functional.
    ///
    /// Active members can:
    /// - Join and leave groups
    /// - Own and trade shares
    /// - Access all platform features
    /// - Participate in group activities
    #[sea_orm(string_value = "active")]
    Active,

    /// Member account is temporarily inactive.
    ///
    /// Inactive members:
    /// - Cannot perform transactions
    /// - Retain their data and relationships
    /// - Can be reactivated by admins
    /// - May have limited read-only access
    #[sea_orm(string_value = "inactive")]
    Inactive,

    /// Member account is suspended due to policy violations.
    ///
    /// Suspended members:
    /// - Cannot access most platform features
    /// - Retain their data for audit purposes
    /// - Require admin intervention to reactivate
    /// - May have appeal or review processes available
    #[sea_orm(string_value = "suspended")]
    Suspended,

    /// Member account is awaiting approval or verification.
    ///
    /// Pending members:
    /// - Have submitted registration but await approval
    /// - Cannot perform most actions until approved
    /// - May need to complete verification steps
    /// - Transition to Active once requirements are met
    #[sea_orm(string_value = "pending")]
    Pending,

    /// Member account has been archived.
    ///
    /// Archived members:
    /// - Are former members who have left the system
    /// - Have their data preserved for historical/audit purposes
    /// - Cannot access the platform
    /// - Maintain data integrity for past transactions
    #[sea_orm(string_value = "archived")]
    Archived,
}
