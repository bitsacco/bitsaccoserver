use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a share offering in the system.
///
/// Share offers define batches of shares that are made available for purchase.
/// Each offer has a specific price, quantity, and validity period. All shares
/// purchased from the same offer will have the same characteristics and price.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "share_offers")]
pub struct Model {
    /// Unique identifier for this share offer.
    ///
    /// This is a UUID that serves as the primary key and is automatically
    /// generated when a new share offer is created.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Name or title of the share offer.
    ///
    /// A descriptive name that helps identify this particular share offering.
    /// Example: "Q1 2024 Share Issuance", "Expansion Fund Shares", "Founding Member Shares"
    pub name: String,

    /// Detailed description of the share offer.
    ///
    /// Optional field providing additional information about the purpose,
    /// terms, or conditions of this share offering.
    /// Example: "Special offer for early members with reduced pricing"
    pub description: Option<String>,

    /// Price per share in this offering.
    ///
    /// The fixed price at which each share in this offer is sold.
    /// All shares purchased from this offer will have this price.
    /// Example: 1000.00 (for 1000 KES per share)
    pub price_per_share: Decimal,

    /// Total number of shares available in this offer.
    ///
    /// The maximum quantity of shares that can be purchased from this offering.
    /// This represents the total pool of shares available for sale.
    pub total_shares_available: Decimal,

    /// Number of shares already purchased from this offer.
    ///
    /// Tracks how many shares have been sold from this offering.
    /// This value is automatically updated when shares are purchased.
    /// Must not exceed total_shares_available.
    pub shares_sold: Decimal,

    /// Number of shares remaining available for purchase.
    ///
    /// Automatically calculated field: total_shares_available - shares_sold.
    /// This value is maintained by database triggers and represents
    /// how many shares can still be purchased from this offer.
    pub shares_remaining: Decimal,

    /// Current status of the share offer.
    ///
    /// Indicates the offer's current state and availability:
    /// - Draft: Offer is being prepared and not yet available
    /// - Active: Offer is available for purchase
    /// - Paused: Offer is temporarily unavailable
    /// - Completed: All shares have been sold
    /// - Expired: Offer validity period has ended
    /// - Cancelled: Offer has been cancelled
    pub status: ShareOfferStatus,

    /// Date and time when the offer becomes available.
    ///
    /// Optional field defining when this offer starts accepting purchases.
    /// If null, the offer is available immediately upon activation.
    pub valid_from: Option<DateTimeWithTimeZone>,

    /// Date and time when the offer expires.
    ///
    /// Optional field defining when this offer stops accepting purchases.
    /// If null, the offer remains valid until manually closed or completed.
    pub valid_until: Option<DateTimeWithTimeZone>,

    /// Minimum number of shares that can be purchased in a single transaction.
    ///
    /// Optional constraint on the minimum purchase quantity.
    /// Helps prevent very small purchases that might not be economical.
    /// Example: 10.00 (minimum 10 shares per purchase)
    pub min_purchase_quantity: Option<Decimal>,

    /// Maximum number of shares that can be purchased in a single transaction.
    ///
    /// Optional constraint on the maximum purchase quantity.
    /// Helps ensure fair distribution among potential buyers.
    /// Example: 1000.00 (maximum 1000 shares per purchase)
    pub max_purchase_quantity: Option<Decimal>,

    /// Configuration settings specific to this offer.
    ///
    /// JSON field containing offer-specific settings such as:
    /// - Payment terms and conditions
    /// - Eligibility criteria
    /// - Purchase restrictions
    /// - Notification preferences
    /// Example: {"payment_methods": ["bank_transfer", "mobile_money"], "member_only": true}
    pub settings: Option<Json>,

    /// Additional metadata about the offer.
    ///
    /// JSON field for storing additional information such as:
    /// - Marketing campaign data
    /// - External system references
    /// - Analytics tracking information
    /// - Custom attributes
    /// Example: {"campaign_id": "CAMP001", "priority": "high"}
    pub metadata: Option<Json>,

    /// Timestamp when this offer was created.
    ///
    /// Automatically set when the offer record is first inserted.
    /// Stored in UTC timezone for consistency across different deployments.
    pub created_at: DateTimeWithTimeZone,

    /// Timestamp when this offer was last updated.
    ///
    /// Automatically updated whenever the offer record is modified.
    /// Tracks changes to status, quantities, or other attributes.
    pub updated_at: DateTimeWithTimeZone,

    /// ID of the user who created this offer.
    ///
    /// Optional reference to the user (typically an admin) who created
    /// this share offering. Used for audit trail purposes.
    pub created_by: Option<Uuid>,

    /// ID of the user who last updated this offer.
    ///
    /// Optional reference to the user who last modified this offer.
    /// Used for tracking who made changes to the offering.
    pub updated_by: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::shares::Entity")]
    Shares,
}

impl Related<super::shares::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Shares.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Enumeration representing the various states a share offer can be in.
///
/// This enum defines the lifecycle states of a share offer and determines
/// what operations can be performed on the offer and whether shares can be purchased.
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "share_offer_status")]
pub enum ShareOfferStatus {
    /// Offer is being prepared and not yet available for purchase.
    ///
    /// Draft offers:
    /// - Cannot be purchased from
    /// - Are visible only to administrators
    /// - Can be modified freely
    /// - Must be activated to become available
    #[sea_orm(string_value = "draft")]
    Draft,

    /// Offer is available for purchase by eligible buyers.
    ///
    /// Active offers:
    /// - Can be purchased from (subject to validity dates and quantity limits)
    /// - Are visible to potential buyers
    /// - Have restrictions on modifications
    /// - Automatically transition to Completed when fully sold
    #[sea_orm(string_value = "active")]
    Active,

    /// Offer is temporarily unavailable for purchase.
    ///
    /// Paused offers:
    /// - Cannot be purchased from temporarily
    /// - Remain visible but show as unavailable
    /// - Can be reactivated by administrators
    /// - Retain all purchase history and remaining quantity
    #[sea_orm(string_value = "paused")]
    Paused,

    /// All available shares have been sold.
    ///
    /// Completed offers:
    /// - Cannot accept new purchases
    /// - Show as fully subscribed
    /// - Preserve all historical data
    /// - Cannot be reactivated (shares_remaining = 0)
    #[sea_orm(string_value = "completed")]
    Completed,

    /// Offer validity period has ended.
    ///
    /// Expired offers:
    /// - Cannot accept new purchases due to time constraints
    /// - May still have shares remaining
    /// - Show expiration information
    /// - Can potentially be extended with new validity dates
    #[sea_orm(string_value = "expired")]
    Expired,

    /// Offer has been cancelled and is no longer available.
    ///
    /// Cancelled offers:
    /// - Cannot accept new purchases
    /// - May need to handle refunds for existing purchases
    /// - Preserve historical data for audit purposes
    /// - Cannot be reactivated without creating a new offer
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
}
