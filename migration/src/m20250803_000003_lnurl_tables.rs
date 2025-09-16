use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create ENUM types for LNURL module
        manager
            .create_type(
                Type::create()
                    .as_enum(LnurlTransactionType::Table)
                    .values([
                        LnurlTransactionType::Pay,
                        LnurlTransactionType::Withdraw,
                        LnurlTransactionType::Channel,
                        LnurlTransactionType::Auth,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(LnurlTransactionStatus::Table)
                    .values([
                        LnurlTransactionStatus::Pending,
                        LnurlTransactionStatus::Processing,
                        LnurlTransactionStatus::Completed,
                        LnurlTransactionStatus::Failed,
                        LnurlTransactionStatus::Expired,
                        LnurlTransactionStatus::Cancelled,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(ExternalTargetType::Table)
                    .values([
                        ExternalTargetType::LightningAddress,
                        ExternalTargetType::LnurlPay,
                        ExternalTargetType::LnurlWithdraw,
                        ExternalTargetType::Invoice,
                        ExternalTargetType::NodeId,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create lightning_addresses table
        manager
            .create_table(
                Table::create()
                    .table(LightningAddresses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LightningAddresses::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(LightningAddresses::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(LightningAddresses::WalletId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LightningAddresses::Domain)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(LightningAddresses::DisplayName).string())
                    .col(ColumnDef::new(LightningAddresses::Avatar).string()) // URL to avatar image
                    .col(ColumnDef::new(LightningAddresses::Description).text())
                    .col(
                        ColumnDef::new(LightningAddresses::MinSendableMsat)
                            .big_integer()
                            .not_null()
                            .default(1000), // 1 sat minimum
                    )
                    .col(
                        ColumnDef::new(LightningAddresses::MaxSendableMsat)
                            .big_integer()
                            .not_null()
                            .default(100000000), // 100k sats maximum
                    )
                    .col(
                        ColumnDef::new(LightningAddresses::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(LightningAddresses::Metadata).json_binary()) // LNURL metadata
                    .col(ColumnDef::new(LightningAddresses::LastUsedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(LightningAddresses::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(LightningAddresses::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(LightningAddresses::CreatedBy).uuid())
                    .col(ColumnDef::new(LightningAddresses::UpdatedBy).uuid())
                    .to_owned(),
            )
            .await?;

        // Create external_targets table for tracking external payment destinations
        manager
            .create_table(
                Table::create()
                    .table(ExternalTargets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExternalTargets::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ExternalTargets::TargetType)
                            .enumeration(
                                ExternalTargetType::Table,
                                [
                                    ExternalTargetType::LightningAddress,
                                    ExternalTargetType::LnurlPay,
                                    ExternalTargetType::LnurlWithdraw,
                                    ExternalTargetType::Invoice,
                                    ExternalTargetType::NodeId,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExternalTargets::Identifier)
                            .string()
                            .not_null(),
                    ) // username@domain, lnurl, invoice, etc.
                    .col(ColumnDef::new(ExternalTargets::DisplayName).string())
                    .col(ColumnDef::new(ExternalTargets::Description).text())
                    .col(ColumnDef::new(ExternalTargets::LnurlEndpoint).string()) // Resolved LNURL endpoint
                    .col(ColumnDef::new(ExternalTargets::MinSendableMsat).big_integer())
                    .col(ColumnDef::new(ExternalTargets::MaxSendableMsat).big_integer())
                    .col(ColumnDef::new(ExternalTargets::Metadata).json_binary()) // LNURL metadata, invoice details, etc.
                    .col(
                        ColumnDef::new(ExternalTargets::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(ExternalTargets::LastVerifiedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ExternalTargets::ExpiresAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(ExternalTargets::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ExternalTargets::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(ExternalTargets::CreatedBy).uuid())
                    .col(ColumnDef::new(ExternalTargets::UpdatedBy).uuid())
                    .to_owned(),
            )
            .await?;

        // Create lnurl_transactions table for LNURL-specific transaction tracking
        manager
            .create_table(
                Table::create()
                    .table(LnurlTransactions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LnurlTransactions::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LnurlTransactions::WalletTransactionId).uuid()) // Link to wallet_transactions
                    .col(ColumnDef::new(LnurlTransactions::LightningAddressId).uuid()) // For incoming payments to lightning addresses
                    .col(ColumnDef::new(LnurlTransactions::ExternalTargetId).uuid()) // For outgoing payments to external targets
                    .col(
                        ColumnDef::new(LnurlTransactions::TransactionType)
                            .enumeration(
                                LnurlTransactionType::Table,
                                [
                                    LnurlTransactionType::Pay,
                                    LnurlTransactionType::Withdraw,
                                    LnurlTransactionType::Channel,
                                    LnurlTransactionType::Auth,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LnurlTransactions::Status)
                            .enumeration(
                                LnurlTransactionStatus::Table,
                                [
                                    LnurlTransactionStatus::Pending,
                                    LnurlTransactionStatus::Processing,
                                    LnurlTransactionStatus::Completed,
                                    LnurlTransactionStatus::Failed,
                                    LnurlTransactionStatus::Expired,
                                    LnurlTransactionStatus::Cancelled,
                                ],
                            )
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(LnurlTransactions::LnurlString).string()) // Original LNURL
                    .col(ColumnDef::new(LnurlTransactions::K1).string()) // K1 parameter for withdraw/auth
                    .col(ColumnDef::new(LnurlTransactions::Invoice).text()) // Generated or received invoice
                    .col(ColumnDef::new(LnurlTransactions::PaymentHash).string()) // Payment hash
                    .col(ColumnDef::new(LnurlTransactions::Preimage).string()) // Payment preimage (when available)
                    .col(ColumnDef::new(LnurlTransactions::AmountMsat).big_integer()) // Amount for the LNURL operation
                    .col(ColumnDef::new(LnurlTransactions::Comment).text()) // Comment from sender/receiver
                    .col(ColumnDef::new(LnurlTransactions::SuccessAction).json_binary()) // Success action for LNURL-pay
                    .col(ColumnDef::new(LnurlTransactions::CallbackUrl).string()) // LNURL callback URL
                    .col(ColumnDef::new(LnurlTransactions::Tag).string()) // LNURL tag (payRequest, withdrawRequest, etc.)
                    .col(ColumnDef::new(LnurlTransactions::Metadata).json_binary()) // LNURL metadata and additional data
                    .col(ColumnDef::new(LnurlTransactions::ErrorDetails).text()) // Error information if failed
                    .col(ColumnDef::new(LnurlTransactions::ProcessedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(LnurlTransactions::ExpiresAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(LnurlTransactions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(LnurlTransactions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(LnurlTransactions::CreatedBy).uuid())
                    .col(ColumnDef::new(LnurlTransactions::UpdatedBy).uuid())
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_lightning_addresses_username_domain")
                    .table(LightningAddresses::Table)
                    .col(LightningAddresses::Username)
                    .col(LightningAddresses::Domain)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_external_targets_type_identifier")
                    .table(ExternalTargets::Table)
                    .col(ExternalTargets::TargetType)
                    .col(ExternalTargets::Identifier)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_lnurl_transactions_payment_hash")
                    .table(LnurlTransactions::Table)
                    .col(LnurlTransactions::PaymentHash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_lnurl_transactions_k1")
                    .table(LnurlTransactions::Table)
                    .col(LnurlTransactions::K1)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_lnurl_transactions_type_status")
                    .table(LnurlTransactions::Table)
                    .col(LnurlTransactions::TransactionType)
                    .col(LnurlTransactions::Status)
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraints after all tables are created
        manager
            .alter_table(
                Table::alter()
                    .table(LightningAddresses::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_lightning_addresses_wallet")
                            .from_col(LightningAddresses::WalletId)
                            .to_tbl(Alias::new("wallets"))
                            .to_col(Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(LnurlTransactions::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_lnurl_transactions_wallet_transaction")
                            .from_col(LnurlTransactions::WalletTransactionId)
                            .to_tbl(Alias::new("wallet_transactions"))
                            .to_col(Alias::new("id"))
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(LnurlTransactions::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_lnurl_transactions_lightning_address")
                            .from_col(LnurlTransactions::LightningAddressId)
                            .to_tbl(LightningAddresses::Table)
                            .to_col(LightningAddresses::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(LnurlTransactions::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_lnurl_transactions_external_target")
                            .from_col(LnurlTransactions::ExternalTargetId)
                            .to_tbl(ExternalTargets::Table)
                            .to_col(ExternalTargets::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order of creation (due to foreign keys)
        manager
            .drop_table(Table::drop().table(LnurlTransactions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ExternalTargets::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(LightningAddresses::Table).to_owned())
            .await?;

        // Drop ENUM types
        manager
            .drop_type(Type::drop().name(ExternalTargetType::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(LnurlTransactionStatus::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(LnurlTransactionType::Table).to_owned())
            .await?;

        Ok(())
    }
}

// Table identifiers
#[derive(Iden)]
enum LightningAddresses {
    Table,
    Id,
    Username,
    #[iden = "wallet_id"]
    WalletId,
    Domain,
    #[iden = "display_name"]
    DisplayName,
    Avatar,
    Description,
    #[iden = "min_sendable_msat"]
    MinSendableMsat,
    #[iden = "max_sendable_msat"]
    MaxSendableMsat,
    #[iden = "is_active"]
    IsActive,
    Metadata,
    #[iden = "last_used_at"]
    LastUsedAt,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "updated_at"]
    UpdatedAt,
    #[iden = "created_by"]
    CreatedBy,
    #[iden = "updated_by"]
    UpdatedBy,
}

#[derive(Iden)]
enum ExternalTargets {
    Table,
    Id,
    #[iden = "target_type"]
    TargetType,
    Identifier,
    #[iden = "display_name"]
    DisplayName,
    Description,
    #[iden = "lnurl_endpoint"]
    LnurlEndpoint,
    #[iden = "min_sendable_msat"]
    MinSendableMsat,
    #[iden = "max_sendable_msat"]
    MaxSendableMsat,
    Metadata,
    #[iden = "is_active"]
    IsActive,
    #[iden = "last_verified_at"]
    LastVerifiedAt,
    #[iden = "expires_at"]
    ExpiresAt,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "updated_at"]
    UpdatedAt,
    #[iden = "created_by"]
    CreatedBy,
    #[iden = "updated_by"]
    UpdatedBy,
}

#[derive(Iden)]
enum LnurlTransactions {
    Table,
    Id,
    #[iden = "wallet_transaction_id"]
    WalletTransactionId,
    #[iden = "lightning_address_id"]
    LightningAddressId,
    #[iden = "external_target_id"]
    ExternalTargetId,
    #[iden = "transaction_type"]
    TransactionType,
    Status,
    #[iden = "lnurl_string"]
    LnurlString,
    K1,
    Invoice,
    #[iden = "payment_hash"]
    PaymentHash,
    Preimage,
    #[iden = "amount_msat"]
    AmountMsat,
    Comment,
    #[iden = "success_action"]
    SuccessAction,
    #[iden = "callback_url"]
    CallbackUrl,
    Tag,
    Metadata,
    #[iden = "error_details"]
    ErrorDetails,
    #[iden = "processed_at"]
    ProcessedAt,
    #[iden = "expires_at"]
    ExpiresAt,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "updated_at"]
    UpdatedAt,
    #[iden = "created_by"]
    CreatedBy,
    #[iden = "updated_by"]
    UpdatedBy,
}

// ENUM identifiers
#[derive(DeriveIden)]
enum LnurlTransactionType {
    Table,
    Pay,
    Withdraw,
    Channel,
    Auth,
}

#[derive(DeriveIden)]
enum LnurlTransactionStatus {
    Table,
    Pending,
    Processing,
    Completed,
    Failed,
    Expired,
    Cancelled,
}

#[derive(DeriveIden)]
enum ExternalTargetType {
    Table,
    LightningAddress,
    LnurlPay,
    LnurlWithdraw,
    Invoice,
    NodeId,
}
