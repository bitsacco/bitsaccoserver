use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create ENUM types for wallet module
        manager
            .create_type(
                Type::create()
                    .as_enum(WalletStatus::Table)
                    .values([
                        WalletStatus::Active,
                        WalletStatus::Inactive,
                        WalletStatus::Suspended,
                        WalletStatus::Closed,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(TransactionType::Table)
                    .values([
                        TransactionType::Deposit,
                        TransactionType::Withdraw,
                        TransactionType::Transfer,
                        TransactionType::Payment,
                        TransactionType::Refund,
                        TransactionType::Fee,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(TransactionStatus::Table)
                    .values([
                        TransactionStatus::Pending,
                        TransactionStatus::Processing,
                        TransactionStatus::Completed,
                        TransactionStatus::Failed,
                        TransactionStatus::Cancelled,
                        TransactionStatus::Expired,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(FedimintOperationType::Table)
                    .values([
                        FedimintOperationType::Deposit,
                        FedimintOperationType::Withdraw,
                        FedimintOperationType::Lightning,
                        FedimintOperationType::Onchain,
                        FedimintOperationType::Mint,
                        FedimintOperationType::Backup,
                        FedimintOperationType::Recovery,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(ReserveType::Table)
                    .values([
                        ReserveType::Available,
                        ReserveType::Pending,
                        ReserveType::Locked,
                        ReserveType::Emergency,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create wallets table
        manager
            .create_table(
                Table::create()
                    .table(Wallets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Wallets::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Wallets::OwnerId).uuid().not_null())
                    .col(ColumnDef::new(Wallets::OwnerType).string().not_null()) // member, group
                    .col(ColumnDef::new(Wallets::Name).string().not_null())
                    .col(ColumnDef::new(Wallets::Description).text())
                    .col(
                        ColumnDef::new(Wallets::Status)
                            .enumeration(
                                WalletStatus::Table,
                                [
                                    WalletStatus::Active,
                                    WalletStatus::Inactive,
                                    WalletStatus::Suspended,
                                    WalletStatus::Closed,
                                ],
                            )
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        ColumnDef::new(Wallets::BalanceMsat)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Wallets::PendingInMsat)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Wallets::PendingOutMsat)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Wallets::FederationId).string())
                    .col(ColumnDef::new(Wallets::ClientConfig).json_binary())
                    .col(ColumnDef::new(Wallets::Metadata).json_binary())
                    .col(ColumnDef::new(Wallets::LastSyncAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Wallets::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Wallets::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Wallets::CreatedBy).uuid())
                    .col(ColumnDef::new(Wallets::UpdatedBy).uuid())
                    .to_owned(),
            )
            .await?;

        // Create wallet_transactions table
        manager
            .create_table(
                Table::create()
                    .table(WalletTransactions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WalletTransactions::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WalletTransactions::WalletId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WalletTransactions::TransactionType)
                            .enumeration(
                                TransactionType::Table,
                                [
                                    TransactionType::Deposit,
                                    TransactionType::Withdraw,
                                    TransactionType::Transfer,
                                    TransactionType::Payment,
                                    TransactionType::Refund,
                                    TransactionType::Fee,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WalletTransactions::Status)
                            .enumeration(
                                TransactionStatus::Table,
                                [
                                    TransactionStatus::Pending,
                                    TransactionStatus::Processing,
                                    TransactionStatus::Completed,
                                    TransactionStatus::Failed,
                                    TransactionStatus::Cancelled,
                                    TransactionStatus::Expired,
                                ],
                            )
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(WalletTransactions::AmountMsat)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WalletTransactions::FeeMsat)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(WalletTransactions::Description).text())
                    .col(ColumnDef::new(WalletTransactions::ExternalId).string()) // Lightning invoice, onchain txid, etc.
                    .col(ColumnDef::new(WalletTransactions::CounterpartyId).uuid()) // Other wallet involved
                    .col(ColumnDef::new(WalletTransactions::FedimintOperationId).uuid())
                    .col(ColumnDef::new(WalletTransactions::Metadata).json_binary())
                    .col(ColumnDef::new(WalletTransactions::ProcessedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(WalletTransactions::ExpiresAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(WalletTransactions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(WalletTransactions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(WalletTransactions::CreatedBy).uuid())
                    .col(ColumnDef::new(WalletTransactions::UpdatedBy).uuid())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_wallet_transactions_wallet")
                            .from(WalletTransactions::Table, WalletTransactions::WalletId)
                            .to(Wallets::Table, Wallets::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create wallet_reserves table for tracking different types of balances
        manager
            .create_table(
                Table::create()
                    .table(WalletReserves::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WalletReserves::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WalletReserves::WalletId).uuid().not_null())
                    .col(
                        ColumnDef::new(WalletReserves::ReserveType)
                            .enumeration(
                                ReserveType::Table,
                                [
                                    ReserveType::Available,
                                    ReserveType::Pending,
                                    ReserveType::Locked,
                                    ReserveType::Emergency,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WalletReserves::AmountMsat)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(WalletReserves::Reference).string()) // Transaction ID or operation reference
                    .col(ColumnDef::new(WalletReserves::ExpiresAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(WalletReserves::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(WalletReserves::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(WalletReserves::CreatedBy).uuid())
                    .col(ColumnDef::new(WalletReserves::UpdatedBy).uuid())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_wallet_reserves_wallet")
                            .from(WalletReserves::Table, WalletReserves::WalletId)
                            .to(Wallets::Table, Wallets::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create fedimint_operations table for tracking Fedimint-specific operations
        manager
            .create_table(
                Table::create()
                    .table(FedimintOperations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FedimintOperations::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("uuid_generate_v4()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(FedimintOperations::WalletId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FedimintOperations::OperationType)
                            .enumeration(
                                FedimintOperationType::Table,
                                [
                                    FedimintOperationType::Deposit,
                                    FedimintOperationType::Withdraw,
                                    FedimintOperationType::Lightning,
                                    FedimintOperationType::Onchain,
                                    FedimintOperationType::Mint,
                                    FedimintOperationType::Backup,
                                    FedimintOperationType::Recovery,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FedimintOperations::Status)
                            .enumeration(
                                TransactionStatus::Table,
                                [
                                    TransactionStatus::Pending,
                                    TransactionStatus::Processing,
                                    TransactionStatus::Completed,
                                    TransactionStatus::Failed,
                                    TransactionStatus::Cancelled,
                                    TransactionStatus::Expired,
                                ],
                            )
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(FedimintOperations::FedimintOperationId).string()) // Fedimint operation ID
                    .col(ColumnDef::new(FedimintOperations::AmountMsat).big_integer())
                    .col(
                        ColumnDef::new(FedimintOperations::FeeMsat)
                            .big_integer()
                            .default(0),
                    )
                    .col(ColumnDef::new(FedimintOperations::Request).json_binary()) // Original request data
                    .col(ColumnDef::new(FedimintOperations::Response).json_binary()) // Response data from Fedimint
                    .col(ColumnDef::new(FedimintOperations::ErrorDetails).text()) // Error information if failed
                    .col(
                        ColumnDef::new(FedimintOperations::RetryCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(FedimintOperations::LastRetryAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(FedimintOperations::ProcessedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(FedimintOperations::ExpiresAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(FedimintOperations::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(FedimintOperations::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(FedimintOperations::CreatedBy).uuid())
                    .col(ColumnDef::new(FedimintOperations::UpdatedBy).uuid())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fedimint_operations_wallet")
                            .from(FedimintOperations::Table, FedimintOperations::WalletId)
                            .to(Wallets::Table, Wallets::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order of creation (due to foreign keys)
        manager
            .drop_table(Table::drop().table(FedimintOperations::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(WalletReserves::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(WalletTransactions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Wallets::Table).to_owned())
            .await?;

        // Drop ENUM types
        manager
            .drop_type(Type::drop().name(ReserveType::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(FedimintOperationType::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(TransactionStatus::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(TransactionType::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(WalletStatus::Table).to_owned())
            .await?;

        Ok(())
    }
}

// Table identifiers
#[derive(Iden)]
enum Wallets {
    Table,
    Id,
    #[iden = "owner_id"]
    OwnerId,
    #[iden = "owner_type"]
    OwnerType,
    Name,
    Description,
    Status,
    #[iden = "balance_msat"]
    BalanceMsat,
    #[iden = "pending_in_msat"]
    PendingInMsat,
    #[iden = "pending_out_msat"]
    PendingOutMsat,
    #[iden = "federation_id"]
    FederationId,
    #[iden = "client_config"]
    ClientConfig,
    Metadata,
    #[iden = "last_sync_at"]
    LastSyncAt,
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
enum WalletTransactions {
    Table,
    Id,
    #[iden = "wallet_id"]
    WalletId,
    #[iden = "transaction_type"]
    TransactionType,
    Status,
    #[iden = "amount_msat"]
    AmountMsat,
    #[iden = "fee_msat"]
    FeeMsat,
    Description,
    #[iden = "external_id"]
    ExternalId,
    #[iden = "counterparty_id"]
    CounterpartyId,
    #[iden = "fedimint_operation_id"]
    FedimintOperationId,
    Metadata,
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

#[derive(Iden)]
enum WalletReserves {
    Table,
    Id,
    #[iden = "wallet_id"]
    WalletId,
    #[iden = "reserve_type"]
    ReserveType,
    #[iden = "amount_msat"]
    AmountMsat,
    Reference,
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
enum FedimintOperations {
    Table,
    Id,
    #[iden = "wallet_id"]
    WalletId,
    #[iden = "operation_type"]
    OperationType,
    Status,
    #[iden = "fedimint_operation_id"]
    FedimintOperationId,
    #[iden = "amount_msat"]
    AmountMsat,
    #[iden = "fee_msat"]
    FeeMsat,
    Request,
    Response,
    #[iden = "error_details"]
    ErrorDetails,
    #[iden = "retry_count"]
    RetryCount,
    #[iden = "last_retry_at"]
    LastRetryAt,
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
enum WalletStatus {
    Table,
    Active,
    Inactive,
    Suspended,
    Closed,
}

#[derive(DeriveIden)]
enum TransactionType {
    Table,
    Deposit,
    Withdraw,
    Transfer,
    Payment,
    Refund,
    Fee,
}

#[derive(DeriveIden)]
enum TransactionStatus {
    Table,
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    Expired,
}

#[derive(DeriveIden)]
enum FedimintOperationType {
    Table,
    Deposit,
    Withdraw,
    Lightning,
    Onchain,
    Mint,
    Backup,
    Recovery,
}

#[derive(DeriveIden)]
enum ReserveType {
    Table,
    Available,
    Pending,
    Locked,
    Emergency,
}
