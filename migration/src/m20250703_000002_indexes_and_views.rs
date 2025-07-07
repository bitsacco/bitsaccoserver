use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create indexes for performance optimization

        // Groups table indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_groups_parent_id")
                    .table(Groups::Table)
                    .col(Groups::ParentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_groups_type")
                    .table(Groups::Table)
                    .col(Groups::GroupType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_groups_status")
                    .table(Groups::Table)
                    .col(Groups::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_groups_path")
                    .table(Groups::Table)
                    .col(Groups::Path)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_groups_level")
                    .table(Groups::Table)
                    .col(Groups::Level)
                    .to_owned(),
            )
            .await?;

        // Members table indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_members_member_number")
                    .table(Members::Table)
                    .col(Members::MemberNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_members_email")
                    .table(Members::Table)
                    .col(Members::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_members_status")
                    .table(Members::Table)
                    .col(Members::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_members_keycloak_user_id")
                    .table(Members::Table)
                    .col(Members::KeycloakUserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_members_name")
                    .table(Members::Table)
                    .col(Members::Name)
                    .to_owned(),
            )
            .await?;

        // Group memberships table indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_group_memberships_group_id")
                    .table(GroupMemberships::Table)
                    .col(GroupMemberships::GroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_group_memberships_member_id")
                    .table(GroupMemberships::Table)
                    .col(GroupMemberships::MemberId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_group_memberships_role")
                    .table(GroupMemberships::Table)
                    .col(GroupMemberships::Role)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_group_memberships_active")
                    .table(GroupMemberships::Table)
                    .col(GroupMemberships::IsActive)
                    .to_owned(),
            )
            .await?;

        // Unique constraint for active group memberships
        manager
            .create_index(
                Index::create()
                    .name("idx_group_memberships_unique_active")
                    .table(GroupMemberships::Table)
                    .col(GroupMemberships::GroupId)
                    .col(GroupMemberships::MemberId)
                    .col(GroupMemberships::Role)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Share offers table indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_share_offers_status")
                    .table(ShareOffers::Table)
                    .col(ShareOffers::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_share_offers_valid_dates")
                    .table(ShareOffers::Table)
                    .col(ShareOffers::ValidFrom)
                    .col(ShareOffers::ValidUntil)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_share_offers_price")
                    .table(ShareOffers::Table)
                    .col(ShareOffers::PricePerShare)
                    .to_owned(),
            )
            .await?;

        // Shares table indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_shares_owner_id")
                    .table(Shares::Table)
                    .col(Shares::OwnerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_shares_owner_type")
                    .table(Shares::Table)
                    .col(Shares::OwnerType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_shares_owner_composite")
                    .table(Shares::Table)
                    .col(Shares::OwnerId)
                    .col(Shares::OwnerType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_shares_total_value")
                    .table(Shares::Table)
                    .col(Shares::TotalValue)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_shares_offer_id")
                    .table(Shares::Table)
                    .col(Shares::ShareOfferId)
                    .to_owned(),
            )
            .await?;

        // Audit logs table indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_table_record")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::TableName)
                    .col(AuditLogs::RecordId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_changed_by")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::ChangedBy)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_changed_at")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::ChangedAt)
                    .to_owned(),
            )
            .await?;

        // Create materialized views for caching

        // Group hierarchy view with aggregated data
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE MATERIALIZED VIEW group_hierarchy_cache AS
                WITH RECURSIVE group_tree AS (
                    -- Base case: root groups (no parent)
                    SELECT 
                        id,
                        name,
                        description,
                        group_type,
                        status,
                        parent_id,
                        path,
                        level,
                        sort_order,
                        created_at,
                        ARRAY[id] as ancestors,
                        0 as depth_from_root
                    FROM groups 
                    WHERE parent_id IS NULL
                    
                    UNION ALL
                    
                    -- Recursive case: child groups
                    SELECT 
                        g.id,
                        g.name,
                        g.description,
                        g.group_type,
                        g.status,
                        g.parent_id,
                        g.path,
                        g.level,
                        g.sort_order,
                        g.created_at,
                        gt.ancestors || g.id,
                        gt.depth_from_root + 1
                    FROM groups g
                    JOIN group_tree gt ON g.parent_id = gt.id
                )
                SELECT 
                    gt.*,
                    COUNT(gm.id) as member_count,
                    COUNT(CASE WHEN gm.is_active = true THEN 1 END) as active_member_count,
                    COUNT(CASE WHEN gm.role = 'admin' THEN 1 END) as admin_count,
                    COALESCE(SUM(CASE WHEN s.owner_type = 'group' THEN s.total_value ELSE 0 END), 0) as total_share_value
                FROM group_tree gt
                LEFT JOIN group_memberships gm ON gt.id = gm.group_id
                LEFT JOIN shares s ON gt.id = s.owner_id AND s.owner_type = 'group'
                GROUP BY 
                    gt.id, gt.name, gt.description, gt.group_type, gt.status,
                    gt.parent_id, gt.path, gt.level, gt.sort_order, gt.created_at,
                    gt.ancestors, gt.depth_from_root
                ORDER BY gt.level, gt.sort_order, gt.name;
                "#,
            )
            .await?;

        // Member summary view with aggregated share data
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE MATERIALIZED VIEW member_summary_cache AS
                SELECT 
                    m.id,
                    m.member_number,
                    m.name,
                    m.email,
                    m.phone,
                    m.status,
                    m.created_at,
                    COUNT(gm.id) as group_count,
                    COUNT(CASE WHEN gm.is_active = true THEN 1 END) as active_group_count,
                    COUNT(CASE WHEN gm.role = 'admin' THEN 1 END) as admin_role_count,
                    COALESCE(SUM(s.total_value), 0) as total_share_value,
                    COALESCE(SUM(s.share_quantity), 0) as total_share_quantity,
                    MAX(s.last_transaction_at) as last_share_transaction
                FROM members m
                LEFT JOIN group_memberships gm ON m.id = gm.member_id
                LEFT JOIN shares s ON m.id = s.owner_id AND s.owner_type = 'member'
                GROUP BY 
                    m.id, m.member_number, m.name, 
                    m.email, m.phone, m.status, m.created_at
                ORDER BY m.member_number;
                "#,
            )
            .await?;

        // Group financial summary view
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE MATERIALIZED VIEW group_financial_cache AS
                SELECT 
                    g.id,
                    g.name,
                    g.group_type,
                    g.status,
                    COUNT(s.id) as share_holders,
                    COALESCE(SUM(s.share_quantity), 0) as total_shares,
                    COALESCE(SUM(s.total_value), 0) as total_share_value,
                    COALESCE(AVG(s.share_value), 0) as average_share_value,
                    MAX(s.last_transaction_at) as last_transaction,
                    COUNT(gm.id) as total_members,
                    COUNT(CASE WHEN gm.is_active = true THEN 1 END) as active_members
                FROM groups g
                LEFT JOIN shares s ON g.id = s.owner_id AND s.owner_type = 'group'
                LEFT JOIN group_memberships gm ON g.id = gm.group_id
                GROUP BY g.id, g.name, g.group_type, g.status
                ORDER BY g.name;
                "#,
            )
            .await?;

        // Create indexes on materialized views
        manager
            .get_connection()
            .execute_unprepared("CREATE UNIQUE INDEX ON group_hierarchy_cache (id);")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("CREATE INDEX ON group_hierarchy_cache (parent_id);")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("CREATE INDEX ON group_hierarchy_cache (group_type);")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("CREATE UNIQUE INDEX ON member_summary_cache (id);")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("CREATE INDEX ON member_summary_cache (member_number);")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("CREATE UNIQUE INDEX ON group_financial_cache (id);")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop materialized views
        manager
            .get_connection()
            .execute_unprepared("DROP MATERIALIZED VIEW IF EXISTS group_financial_cache;")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP MATERIALIZED VIEW IF EXISTS member_summary_cache;")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP MATERIALIZED VIEW IF EXISTS group_hierarchy_cache;")
            .await?;

        // Drop indexes (they will be dropped automatically with the tables, but for completeness)
        let indexes = vec![
            "idx_audit_logs_changed_at",
            "idx_audit_logs_changed_by",
            "idx_audit_logs_table_record",
            "idx_shares_offer_id",
            "idx_shares_total_value",
            "idx_shares_owner_composite",
            "idx_shares_owner_type",
            "idx_shares_owner_id",
            "idx_share_offers_price",
            "idx_share_offers_valid_dates",
            "idx_share_offers_status",
            "idx_group_memberships_unique_active",
            "idx_group_memberships_active",
            "idx_group_memberships_role",
            "idx_group_memberships_member_id",
            "idx_group_memberships_group_id",
            "idx_members_name",
            "idx_members_keycloak_user_id",
            "idx_members_status",
            "idx_members_email",
            "idx_members_member_number",
            "idx_groups_level",
            "idx_groups_path",
            "idx_groups_status",
            "idx_groups_type",
            "idx_groups_parent_id",
        ];

        for index_name in indexes {
            manager
                .drop_index(Index::drop().name(index_name).to_owned())
                .await
                .ok(); // Ignore errors if index doesn't exist
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Groups {
    Table,
    GroupType,
    Status,
    ParentId,
    Path,
    Level,
}

#[derive(DeriveIden)]
enum Members {
    Table,
    MemberNumber,
    Name,
    Email,
    Status,
    KeycloakUserId,
}

#[derive(DeriveIden)]
enum GroupMemberships {
    Table,
    GroupId,
    MemberId,
    Role,
    IsActive,
}

#[derive(DeriveIden)]
enum ShareOffers {
    Table,
    Status,
    ValidFrom,
    ValidUntil,
    PricePerShare,
}

#[derive(DeriveIden)]
enum Shares {
    Table,
    OwnerId,
    OwnerType,
    ShareOfferId,
    TotalValue,
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    TableName,
    RecordId,
    ChangedBy,
    ChangedAt,
}
