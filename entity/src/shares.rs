use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents share ownership in the system.
///
/// This entity tracks share holdings by either individual members or groups.
/// Shares are issued globally and not tied to specific groups. The ownership can be
/// by individual members or by groups themselves, with the owner_type field indicating
/// which type of entity owns the shares. The total value is automatically calculated
/// from share_quantity × share_value.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "shares")]
pub struct Model {
    /// Unique identifier for this share record.
    ///
    /// This is a UUID that serves as the primary key and is automatically
    /// generated when a new share record is created.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Reference to the entity that owns these shares.
    ///
    /// UUID linking to either the members or groups table, depending on owner_type.
    /// This generic ownership allows both individual members and groups to own shares.
    pub owner_id: Uuid,

    /// Type of entity that owns these shares.
    ///
    /// Indicates whether the owner_id refers to a member or a group:
    /// - Member: Individual member owns these shares
    /// - Group: Group entity owns these shares
    pub owner_type: OwnerType,

    /// Reference to the share offer from which these shares were purchased.
    ///
    /// Foreign key linking to the share_offers table. This identifies
    /// which specific share offering these shares were acquired from,
    /// ensuring traceability and consistent pricing within the same offer.
    pub share_offer_id: Uuid,

    /// Quantity of shares purchased in this transaction.
    ///
    /// Decimal value representing the number of shares acquired by the owner
    /// in this specific purchase transaction from the associated share offer.
    /// Supports fractional shares for flexible contribution systems.
    /// Example: 100.00, 50.5, 1000.25
    pub share_quantity: Decimal,

    /// Current value per individual share.
    ///
    /// Decimal value representing the current monetary value of a single share.
    /// This value can change over time based on different share issuances.
    /// however, this value is consistent across all share records within the same issuance.
    /// Example: 1000.00 (for 1000 KES per share)
    pub share_value: Decimal,

    /// Total value of this share purchase.
    ///
    /// Automatically calculated field: share_quantity × share_value.
    /// This value is maintained by database triggers and represents
    /// the total monetary value of this specific share purchase.
    pub total_value: Decimal,

    /// Timestamp of the last transaction affecting these shares.
    ///
    /// Optional field recording when shares were last bought, sold,
    /// or had their value updated. Used for tracking transaction history
    /// and calculating time-based metrics.
    pub last_transaction_at: Option<DateTimeWithTimeZone>,

    /// Timestamp when this share record was created.
    ///
    /// Automatically set when the share record is first inserted.
    /// Represents when the owner first acquired these shares.
    pub created_at: DateTimeWithTimeZone,

    /// Timestamp when this share record was last updated.
    ///
    /// Automatically updated whenever the share record is modified.
    /// Tracks changes to share quantity, value, or other attributes.
    pub updated_at: DateTimeWithTimeZone,

    /// ID of the user who created this share record.
    ///
    /// Optional reference to the user (admin, member, or system) who
    /// created this share allocation. Used for audit trail purposes.
    pub created_by: Option<Uuid>,

    /// ID of the user who last updated this share record.
    ///
    /// Optional reference to the user who last modified this share record.
    /// Used for tracking who authorized changes to share holdings.
    pub updated_by: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::share_offers::Entity",
        from = "Column::ShareOfferId",
        to = "super::share_offers::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    ShareOffer,
}

impl Related<super::share_offers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ShareOffer.def()
    }
}

// Note: Since owner_id can reference either members or groups based on owner_type,
// we cannot use standard SeaORM relationships for ownership. Instead, we need to handle the
// ownership relationships programmatically based on the owner_type field.

impl ActiveModelBehavior for ActiveModel {}

/// Enumeration defining the types of entities that can own shares.
///
/// This enum determines whether the owner_id field references a member or a group,
/// enabling both individual members and groups to own shares in the system.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "owner_type")]
pub enum OwnerType {
    /// Individual member owns these shares.
    ///
    /// When owner_type is Member, the owner_id field references the members table.
    /// This represents personal share ownership by individual members.
    #[sea_orm(string_value = "member")]
    Member,

    /// Group entity owns these shares.
    ///
    /// When owner_type is Group, the owner_id field references the groups table.
    /// This represents institutional share ownership by groups or organizations.
    #[sea_orm(string_value = "group")]
    Group,
}
