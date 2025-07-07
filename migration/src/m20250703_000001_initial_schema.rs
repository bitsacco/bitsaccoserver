use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create UUID extension if not exists
        manager
            .get_connection()
            .execute_unprepared("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";")
            .await?;

        // Create ENUM types
        manager
            .create_type(
                Type::create()
                    .as_enum(GroupType::Table)
                    .values([GroupType::Organization, GroupType::Chama])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(GroupStatus::Table)
                    .values([
                        GroupStatus::Active,
                        GroupStatus::Inactive,
                        GroupStatus::Suspended,
                        GroupStatus::Pending,
                        GroupStatus::Archived,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(MemberStatus::Table)
                    .values([
                        MemberStatus::Active,
                        MemberStatus::Inactive,
                        MemberStatus::Suspended,
                        MemberStatus::Pending,
                        MemberStatus::Archived,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(MembershipRole::Table)
                    .values([MembershipRole::Member, MembershipRole::Admin])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(OwnerType::Table)
                    .values([OwnerType::Member, OwnerType::Group])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(ShareOfferStatus::Table)
                    .values([
                        ShareOfferStatus::Draft,
                        ShareOfferStatus::Active,
                        ShareOfferStatus::Paused,
                        ShareOfferStatus::Completed,
                        ShareOfferStatus::Expired,
                        ShareOfferStatus::Cancelled,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create unified groups table
        manager
            .create_table(
                Table::create()
                    .table(Groups::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Groups::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Groups::Name).string().not_null())
                    .col(ColumnDef::new(Groups::Description).text())
                    .col(
                        ColumnDef::new(Groups::GroupType)
                            .enumeration(
                                GroupType::Table,
                                [GroupType::Organization, GroupType::Chama],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Groups::Status)
                            .enumeration(
                                GroupStatus::Table,
                                [
                                    GroupStatus::Active,
                                    GroupStatus::Inactive,
                                    GroupStatus::Suspended,
                                    GroupStatus::Pending,
                                    GroupStatus::Archived,
                                ],
                            )
                            .not_null()
                            .default("active"),
                    )
                    .col(ColumnDef::new(Groups::ParentId).uuid())
                    .col(ColumnDef::new(Groups::Path).string()) // Materialized path for hierarchy
                    .col(
                        ColumnDef::new(Groups::Level)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Groups::SortOrder).integer().default(0))
                    .col(ColumnDef::new(Groups::Settings).json_binary())
                    .col(ColumnDef::new(Groups::Metadata).json_binary())
                    .col(
                        ColumnDef::new(Groups::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Groups::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Groups::CreatedBy).uuid())
                    .col(ColumnDef::new(Groups::UpdatedBy).uuid())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_groups_parent")
                            .from(Groups::Table, Groups::ParentId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create members table
        manager
            .create_table(
                Table::create()
                    .table(Members::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Members::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Members::MemberNumber)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Members::Name).string().not_null())
                    .col(ColumnDef::new(Members::Email).string())
                    .col(ColumnDef::new(Members::Phone).string())
                    .col(
                        ColumnDef::new(Members::Status)
                            .enumeration(
                                MemberStatus::Table,
                                [
                                    MemberStatus::Active,
                                    MemberStatus::Inactive,
                                    MemberStatus::Suspended,
                                    MemberStatus::Pending,
                                    MemberStatus::Archived,
                                ],
                            )
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(Members::KeycloakUserId).string())
                    .col(ColumnDef::new(Members::Profile).json_binary())
                    .col(
                        ColumnDef::new(Members::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Members::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Members::CreatedBy).uuid())
                    .col(ColumnDef::new(Members::UpdatedBy).uuid())
                    .to_owned(),
            )
            .await?;

        // Create group_memberships table (many-to-many with roles)
        manager
            .create_table(
                Table::create()
                    .table(GroupMemberships::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GroupMemberships::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GroupMemberships::GroupId).uuid().not_null())
                    .col(ColumnDef::new(GroupMemberships::MemberId).uuid().not_null())
                    .col(
                        ColumnDef::new(GroupMemberships::Role)
                            .enumeration(
                                MembershipRole::Table,
                                [MembershipRole::Member, MembershipRole::Admin],
                            )
                            .not_null()
                            .default("member"),
                    )
                    .col(
                        ColumnDef::new(GroupMemberships::JoinedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(GroupMemberships::LeftAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(GroupMemberships::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(GroupMemberships::Permissions).json_binary())
                    .col(
                        ColumnDef::new(GroupMemberships::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(GroupMemberships::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(GroupMemberships::CreatedBy).uuid())
                    .col(ColumnDef::new(GroupMemberships::UpdatedBy).uuid())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_memberships_group")
                            .from(GroupMemberships::Table, GroupMemberships::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_memberships_member")
                            .from(GroupMemberships::Table, GroupMemberships::MemberId)
                            .to(Members::Table, Members::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create share_offers table
        manager
            .create_table(
                Table::create()
                    .table(ShareOffers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ShareOffers::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ShareOffers::Name).string().not_null())
                    .col(ColumnDef::new(ShareOffers::Description).text())
                    .col(
                        ColumnDef::new(ShareOffers::PricePerShare)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ShareOffers::TotalSharesAvailable)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ShareOffers::SharesSold)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ShareOffers::SharesRemaining)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ShareOffers::Status)
                            .enumeration(
                                ShareOfferStatus::Table,
                                [
                                    ShareOfferStatus::Draft,
                                    ShareOfferStatus::Active,
                                    ShareOfferStatus::Paused,
                                    ShareOfferStatus::Completed,
                                    ShareOfferStatus::Expired,
                                    ShareOfferStatus::Cancelled,
                                ],
                            )
                            .not_null()
                            .default("draft"),
                    )
                    .col(ColumnDef::new(ShareOffers::ValidFrom).timestamp_with_time_zone())
                    .col(ColumnDef::new(ShareOffers::ValidUntil).timestamp_with_time_zone())
                    .col(ColumnDef::new(ShareOffers::MinPurchaseQuantity).decimal_len(15, 2))
                    .col(ColumnDef::new(ShareOffers::MaxPurchaseQuantity).decimal_len(15, 2))
                    .col(ColumnDef::new(ShareOffers::Settings).json_binary())
                    .col(ColumnDef::new(ShareOffers::Metadata).json_binary())
                    .col(
                        ColumnDef::new(ShareOffers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ShareOffers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(ShareOffers::CreatedBy).uuid())
                    .col(ColumnDef::new(ShareOffers::UpdatedBy).uuid())
                    .to_owned(),
            )
            .await?;

        // Create shares table
        manager
            .create_table(
                Table::create()
                    .table(Shares::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Shares::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Shares::OwnerId).uuid().not_null())
                    .col(
                        ColumnDef::new(Shares::OwnerType)
                            .enumeration(OwnerType::Table, [OwnerType::Member, OwnerType::Group])
                            .not_null(),
                    )
                    .col(ColumnDef::new(Shares::ShareOfferId).uuid().not_null())
                    .col(
                        ColumnDef::new(Shares::ShareQuantity)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Shares::ShareValue)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Shares::TotalValue)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Shares::LastTransactionAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Shares::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Shares::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Shares::CreatedBy).uuid())
                    .col(ColumnDef::new(Shares::UpdatedBy).uuid())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_shares_offer")
                            .from(Shares::Table, Shares::ShareOfferId)
                            .to(ShareOffers::Table, ShareOffers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // Note: We cannot use standard foreign keys since owner_id can reference
                    // either members or groups table based on owner_type.
                    // Database constraints for referential integrity should be handled
                    // at the application level or through database triggers.
                    .to_owned(),
            )
            .await?;

        // Create audit_logs table
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuditLogs::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuditLogs::TableName).string().not_null())
                    .col(ColumnDef::new(AuditLogs::RecordId).uuid().not_null())
                    .col(ColumnDef::new(AuditLogs::Operation).string().not_null()) // INSERT, UPDATE, DELETE
                    .col(ColumnDef::new(AuditLogs::OldValues).json_binary())
                    .col(ColumnDef::new(AuditLogs::NewValues).json_binary())
                    .col(ColumnDef::new(AuditLogs::ChangedBy).uuid())
                    .col(
                        ColumnDef::new(AuditLogs::ChangedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(AuditLogs::IpAddress).string())
                    .col(ColumnDef::new(AuditLogs::UserAgent).text())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Shares::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(GroupMemberships::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Members::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Groups::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(MembershipRole::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(MemberStatus::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(GroupStatus::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(GroupType::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Groups {
    Table,
    Id,
    Name,
    Description,
    #[iden = "group_type"]
    GroupType,
    Status,
    #[iden = "parent_id"]
    ParentId,
    Path,
    Level,
    #[iden = "sort_order"]
    SortOrder,
    Settings,
    Metadata,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "updated_at"]
    UpdatedAt,
    #[iden = "created_by"]
    CreatedBy,
    #[iden = "updated_by"]
    UpdatedBy,
}

#[derive(DeriveIden)]
enum GroupType {
    Table,
    Organization,
    Chama,
}

#[derive(DeriveIden)]
enum GroupStatus {
    Table,
    Active,
    Inactive,
    Suspended,
    Pending,
    Archived,
}

#[derive(Iden)]
enum Members {
    Table,
    Id,
    #[iden = "member_number"]
    MemberNumber,
    Name,
    Email,
    Phone,
    Status,
    #[iden = "keycloak_user_id"]
    KeycloakUserId,
    Profile,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "updated_at"]
    UpdatedAt,
    #[iden = "created_by"]
    CreatedBy,
    #[iden = "updated_by"]
    UpdatedBy,
}

#[derive(DeriveIden)]
enum MemberStatus {
    Table,
    Active,
    Inactive,
    Suspended,
    Pending,
    Archived,
}

#[derive(Iden)]
enum GroupMemberships {
    Table,
    Id,
    #[iden = "group_id"]
    GroupId,
    #[iden = "member_id"]
    MemberId,
    Role,
    #[iden = "joined_at"]
    JoinedAt,
    #[iden = "left_at"]
    LeftAt,
    #[iden = "is_active"]
    IsActive,
    Permissions,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "updated_at"]
    UpdatedAt,
    #[iden = "created_by"]
    CreatedBy,
    #[iden = "updated_by"]
    UpdatedBy,
}

#[derive(DeriveIden)]
enum MembershipRole {
    Table,
    Member,
    Admin,
}

#[derive(Iden)]
enum ShareOffers {
    Table,
    Id,
    Name,
    Description,
    #[iden = "price_per_share"]
    PricePerShare,
    #[iden = "total_shares_available"]
    TotalSharesAvailable,
    #[iden = "shares_sold"]
    SharesSold,
    #[iden = "shares_remaining"]
    SharesRemaining,
    Status,
    #[iden = "valid_from"]
    ValidFrom,
    #[iden = "valid_until"]
    ValidUntil,
    #[iden = "min_purchase_quantity"]
    MinPurchaseQuantity,
    #[iden = "max_purchase_quantity"]
    MaxPurchaseQuantity,
    Settings,
    Metadata,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "updated_at"]
    UpdatedAt,
    #[iden = "created_by"]
    CreatedBy,
    #[iden = "updated_by"]
    UpdatedBy,
}

#[derive(DeriveIden)]
enum ShareOfferStatus {
    Table,
    Draft,
    Active,
    Paused,
    Completed,
    Expired,
    Cancelled,
}

#[derive(Iden)]
enum Shares {
    Table,
    Id,
    #[iden = "owner_id"]
    OwnerId,
    #[iden = "owner_type"]
    OwnerType,
    #[iden = "share_offer_id"]
    ShareOfferId,
    #[iden = "share_quantity"]
    ShareQuantity,
    #[iden = "share_value"]
    ShareValue,
    #[iden = "total_value"]
    TotalValue,
    #[iden = "last_transaction_at"]
    LastTransactionAt,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "updated_at"]
    UpdatedAt,
    #[iden = "created_by"]
    CreatedBy,
    #[iden = "updated_by"]
    UpdatedBy,
}

#[derive(DeriveIden)]
enum OwnerType {
    Table,
    Member,
    Group,
}

#[derive(Iden)]
enum AuditLogs {
    Table,
    Id,
    #[iden = "table_name"]
    TableName,
    #[iden = "record_id"]
    RecordId,
    Operation,
    #[iden = "old_values"]
    OldValues,
    #[iden = "new_values"]
    NewValues,
    #[iden = "changed_by"]
    ChangedBy,
    #[iden = "changed_at"]
    ChangedAt,
    #[iden = "ip_address"]
    IpAddress,
    #[iden = "user_agent"]
    UserAgent,
}
