use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a membership relationship between a member and a group.
///
/// This entity manages the many-to-many relationship between members and groups,
/// including role assignments, membership status, and historical tracking of
/// membership changes over time.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "group_memberships")]
pub struct Model {
    /// Unique identifier for this membership record.
    ///
    /// This is a UUID that serves as the primary key and is automatically
    /// generated when a new membership is created.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Reference to the group in this membership.
    ///
    /// Foreign key linking to the groups table. Indicates which group
    /// this membership record relates to.
    pub group_id: Uuid,

    /// Reference to the member in this membership.
    ///
    /// Foreign key linking to the members table. Indicates which member
    /// this membership record relates to.
    pub member_id: Uuid,

    /// Role of the member within the group.
    ///
    /// Defines the member's permissions and responsibilities:
    /// - Member: Standard member with basic group privileges
    /// - Admin: Administrative member with management capabilities
    pub role: MembershipRole,

    /// Timestamp when the member joined the group.
    ///
    /// Records the exact date and time when the membership was established.
    /// Used for tenure calculations and historical tracking.
    pub joined_at: DateTimeWithTimeZone,

    /// Timestamp when the member left the group.
    ///
    /// Optional field that records when a member left or was removed
    /// from the group. None indicates the member is still active.
    /// Used in conjunction with is_active for membership status.
    pub left_at: Option<DateTimeWithTimeZone>,

    /// Whether the membership is currently active.
    ///
    /// Boolean flag indicating if this membership is currently active:
    /// - true: Member is active and can participate in group activities
    /// - false: Member has left, been removed, or temporarily deactivated
    pub is_active: bool,

    /// Custom permissions for this specific membership.
    ///
    /// JSON field containing role-specific or custom permissions that
    /// override or extend the default role permissions. Allows for
    /// fine-grained access control on a per-membership basis.
    /// Example: {"can_approve_transactions": true, "can_invite_members": false}
    pub permissions: Option<Json>,

    /// Timestamp when this membership record was created.
    ///
    /// Automatically set when the membership record is first inserted.
    /// Stored in UTC timezone for consistency across different deployments.
    pub created_at: DateTimeWithTimeZone,

    /// Timestamp when this membership record was last updated.
    ///
    /// Automatically updated whenever the membership record is modified.
    /// Tracks changes to role, status, permissions, etc.
    pub updated_at: DateTimeWithTimeZone,

    /// ID of the user who created this membership.
    ///
    /// Optional reference to the user (admin, member, or system) who
    /// created this membership record. Used for audit trail purposes.
    pub created_by: Option<Uuid>,

    /// ID of the user who last updated this membership.
    ///
    /// Optional reference to the user who last modified this membership.
    /// Used for tracking who made changes to roles or status.
    pub updated_by: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::groups::Entity",
        from = "Column::GroupId",
        to = "super::groups::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Group,
    #[sea_orm(
        belongs_to = "super::members::Entity",
        from = "Column::MemberId",
        to = "super::members::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Member,
}

impl Related<super::groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Group.def()
    }
}

impl Related<super::members::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Member.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Enumeration defining roles a member can have within a group.
///
/// This enum determines the level of access and administrative capabilities
/// a member has within a specific group. Roles can be upgraded or downgraded
/// based on group needs and member performance.
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "membership_role")]
pub enum MembershipRole {
    /// Standard member with basic group privileges.
    ///
    /// Regular members can:
    /// - Participate in group activities
    /// - Make contributions and receive payouts
    /// - View group information and history
    /// - Attend meetings and vote on matters
    /// - Cannot modify group settings or manage other members
    #[sea_orm(string_value = "member")]
    Member,

    /// Administrative member with management capabilities.
    ///
    /// Admin members can:
    /// - Perform all member actions
    /// - Manage group settings and configuration
    /// - Add, remove, and modify member roles
    /// - Approve transactions and handle disputes
    /// - Access administrative reports and analytics
    /// - Make decisions on behalf of the group
    #[sea_orm(string_value = "admin")]
    Admin,
}
