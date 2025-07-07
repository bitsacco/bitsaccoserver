use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a group entity in the system.
///
/// Groups are organizational structures that can contain members and manage shares.
/// They support hierarchical relationships, allowing for nested group structures
/// such as organizations containing multiple chamas or departments.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "groups")]
pub struct Model {
    /// Unique identifier for the group.
    ///
    /// This is a UUID that serves as the primary key and is automatically
    /// generated when a new group is created.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Name of the group.
    ///
    /// A human-readable name that identifies the group.
    /// Should be descriptive and unique within the same parent level.
    /// Example: "Acme Corporation", "Savings Chama Group A"
    pub name: String,

    /// Description of the group.
    ///
    /// Optional detailed description explaining the group's purpose,
    /// activities, or any relevant information for members and administrators.
    /// Example: "Monthly savings group for young professionals"
    pub description: Option<String>,

    /// Type of the group.
    ///
    /// Defines the category and purpose of the group:
    /// - Organization: Large entity that can contain multiple chamas
    /// - Chama: Investment/savings group with shared financial goals
    pub group_type: GroupType,

    /// Current operational status of the group.
    ///
    /// Indicates the group's current state and operational capability:
    /// - Active: Group is operational and accepting members
    /// - Inactive: Group is temporarily paused
    /// - Suspended: Group has been suspended due to issues
    /// - Pending: New group awaiting approval
    /// - Archived: Former group preserved for historical records
    pub status: GroupStatus,

    /// Parent group identifier for hierarchical structure.
    ///
    /// Optional reference to a parent group, enabling nested organizational
    /// structures. If None, this is a root-level group.
    /// Example: A chama might belong to a larger organization
    pub parent_id: Option<Uuid>,

    /// Materialized path for efficient hierarchy queries.
    ///
    /// Optional string representing the path from root to this group,
    /// typically using slash-separated IDs for fast hierarchy operations.
    /// Example: "/org1/chama3"
    pub path: Option<String>,

    /// Depth level in the hierarchy.
    ///
    /// Indicates how deep this group is in the organizational hierarchy.
    /// Root groups have level 0, their children have level 1, etc.
    /// Used for efficient querying and display organization.
    pub level: i32,

    /// Sort order within the same level.
    ///
    /// Optional integer used to control the display order of groups
    /// at the same hierarchical level. Lower numbers appear first.
    /// Useful for organizing groups in a specific sequence.
    pub sort_order: Option<i32>,

    /// Group-specific configuration settings.
    ///
    /// JSON field containing configurable settings specific to this group,
    /// such as meeting schedules, contribution amounts, rules, etc.
    /// Example: {"meeting_day": "Monday", "min_contribution": 1000}
    pub settings: Option<Json>,

    /// Additional metadata about the group.
    ///
    /// JSON field for storing additional information that doesn't fit
    /// into structured fields. Can include contact information, location,
    /// custom attributes, or integration data.
    /// Example: {"location": "Nairobi", "meeting_venue": "Community Center"}
    pub metadata: Option<Json>,

    /// Timestamp when the group was created.
    ///
    /// Automatically set when the group record is first inserted.
    /// Stored in UTC timezone for consistency across different deployments.
    pub created_at: DateTimeWithTimeZone,

    /// Timestamp when the group was last updated.
    ///
    /// Automatically updated whenever the group record is modified.
    /// Stored in UTC timezone for consistency across different deployments.
    pub updated_at: DateTimeWithTimeZone,

    /// ID of the user who created this group.
    ///
    /// Optional reference to the user who created this group.
    /// Used for audit trail and accountability purposes.
    pub created_by: Option<Uuid>,

    /// ID of the user who last updated this group.
    ///
    /// Optional reference to the user who last modified this group record.
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

impl Related<super::members::Entity> for Entity {
    fn to() -> RelationDef {
        super::group_memberships::Relation::Member.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::group_memberships::Relation::Group.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Check if the group is active and can perform operations
    pub fn is_active(&self) -> bool {
        self.status == GroupStatus::Active
    }

    /// Check if the group can be activated
    pub fn can_be_activated(&self) -> bool {
        matches!(self.status, GroupStatus::Inactive | GroupStatus::Pending)
    }

    /// Check if the group is in a state that allows read operations
    pub fn can_read(&self) -> bool {
        !matches!(self.status, GroupStatus::Archived | GroupStatus::Suspended)
    }

    /// Check if the group is a root-level group (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Check if the group can have child groups
    pub fn can_have_children(&self) -> bool {
        matches!(self.group_type, GroupType::Organization) && self.is_active()
    }
}

// Self-referential relationships for parent-child hierarchy
impl Entity {
    pub fn find_children() -> Select<Entity> {
        Self::find()
    }

    pub fn find_by_parent(parent_id: Uuid) -> Select<Entity> {
        Self::find().filter(Column::ParentId.eq(parent_id))
    }
}

/// Enumeration defining the different types of groups in the system.
///
/// This enum categorizes groups based on their organizational purpose and structure,
/// determining their capabilities and relationship patterns.
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "group_type")]
pub enum GroupType {
    /// Large organizational entity that can contain multiple sub-groups.
    ///
    /// Organizations are typically:
    /// - Companies, institutions, or large entities
    /// - Parent containers for multiple chamas or departments
    /// - Have administrative oversight capabilities
    /// - Can set policies for their sub-groups
    /// Example: "Acme Financial Services", "Kenya Women's Network"
    #[sea_orm(string_value = "organization")]
    Organization,

    /// Investment or savings group with shared financial goals.
    ///
    /// Chamas are typically:
    /// - Small to medium-sized collaborative groups
    /// - Focused on collective savings and investment
    /// - Have rotating contribution and payout systems
    /// - Operate under shared agreements and rules
    /// Example: "Monthly Savings Circle", "Investment Club Kenya"
    #[sea_orm(string_value = "chama")]
    Chama,
}

/// Enumeration representing the operational status of a group.
///
/// This enum defines the lifecycle states of a group and determines
/// what operations and activities the group can perform.
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "group_status")]
pub enum GroupStatus {
    /// Group is fully operational and accepting new members.
    ///
    /// Active groups can:
    /// - Accept new member applications
    /// - Conduct financial transactions
    /// - Hold meetings and activities
    /// - Manage shares and contributions
    #[sea_orm(string_value = "active")]
    Active,

    /// Group is temporarily inactive.
    ///
    /// Inactive groups:
    /// - Cannot accept new members
    /// - Have limited transaction capabilities
    /// - Retain all historical data
    /// - Can be reactivated by administrators
    #[sea_orm(string_value = "inactive")]
    Inactive,

    /// Group has been suspended due to violations or issues.
    ///
    /// Suspended groups:
    /// - Cannot perform most operations
    /// - Are under administrative review
    /// - Retain data for audit purposes
    /// - Require resolution of issues for reactivation
    #[sea_orm(string_value = "suspended")]
    Suspended,

    /// New group awaiting approval or setup completion.
    ///
    /// Pending groups:
    /// - Are in the process of being established
    /// - May need additional verification or documentation
    /// - Cannot perform full operations until approved
    /// - Transition to Active once requirements are met
    #[sea_orm(string_value = "pending")]
    Pending,

    /// Former group that has been archived for historical purposes.
    ///
    /// Archived groups:
    /// - Are no longer operational
    /// - Preserve historical data and transactions
    /// - Cannot be modified or reactivated
    /// - Maintain data integrity for audit and reporting
    #[sea_orm(string_value = "archived")]
    Archived,
}
