use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create indexes for wallets table
        manager
            .create_index(
                Index::create()
                    .name("idx_wallets_owner")
                    .table(Wallets::Table)
                    .col(Wallets::OwnerId)
                    .col(Wallets::OwnerType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallets_status")
                    .table(Wallets::Table)
                    .col(Wallets::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallets_federation_id")
                    .table(Wallets::Table)
                    .col(Wallets::FederationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallets_last_sync_at")
                    .table(Wallets::Table)
                    .col(Wallets::LastSyncAt)
                    .to_owned(),
            )
            .await?;

        // Create indexes for wallet_transactions table
        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_transactions_wallet_id")
                    .table(WalletTransactions::Table)
                    .col(WalletTransactions::WalletId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_transactions_type_status")
                    .table(WalletTransactions::Table)
                    .col(WalletTransactions::TransactionType)
                    .col(WalletTransactions::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_transactions_external_id")
                    .table(WalletTransactions::Table)
                    .col(WalletTransactions::ExternalId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_transactions_counterparty")
                    .table(WalletTransactions::Table)
                    .col(WalletTransactions::CounterpartyId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_transactions_fedimint_op")
                    .table(WalletTransactions::Table)
                    .col(WalletTransactions::FedimintOperationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_transactions_created_at")
                    .table(WalletTransactions::Table)
                    .col(WalletTransactions::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_transactions_processed_at")
                    .table(WalletTransactions::Table)
                    .col(WalletTransactions::ProcessedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_transactions_expires_at")
                    .table(WalletTransactions::Table)
                    .col(WalletTransactions::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        // Create composite index for wallet transaction history queries
        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_transactions_wallet_created_desc")
                    .table(WalletTransactions::Table)
                    .col(WalletTransactions::WalletId)
                    .col((WalletTransactions::CreatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        // Create indexes for wallet_reserves table
        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_reserves_wallet_id")
                    .table(WalletReserves::Table)
                    .col(WalletReserves::WalletId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_reserves_type")
                    .table(WalletReserves::Table)
                    .col(WalletReserves::ReserveType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_reserves_reference")
                    .table(WalletReserves::Table)
                    .col(WalletReserves::Reference)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_reserves_expires_at")
                    .table(WalletReserves::Table)
                    .col(WalletReserves::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        // Create composite index for wallet balance calculations
        manager
            .create_index(
                Index::create()
                    .name("idx_wallet_reserves_wallet_type")
                    .table(WalletReserves::Table)
                    .col(WalletReserves::WalletId)
                    .col(WalletReserves::ReserveType)
                    .to_owned(),
            )
            .await?;

        // Create indexes for fedimint_operations table
        manager
            .create_index(
                Index::create()
                    .name("idx_fedimint_operations_wallet_id")
                    .table(FedimintOperations::Table)
                    .col(FedimintOperations::WalletId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_fedimint_operations_type_status")
                    .table(FedimintOperations::Table)
                    .col(FedimintOperations::OperationType)
                    .col(FedimintOperations::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_fedimint_operations_fedimint_id")
                    .table(FedimintOperations::Table)
                    .col(FedimintOperations::FedimintOperationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_fedimint_operations_status")
                    .table(FedimintOperations::Table)
                    .col(FedimintOperations::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_fedimint_operations_retry")
                    .table(FedimintOperations::Table)
                    .col(FedimintOperations::RetryCount)
                    .col(FedimintOperations::LastRetryAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_fedimint_operations_created_at")
                    .table(FedimintOperations::Table)
                    .col(FedimintOperations::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_fedimint_operations_expires_at")
                    .table(FedimintOperations::Table)
                    .col(FedimintOperations::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        // Create unique constraints
        manager
            .create_index(
                Index::create()
                    .name("idx_wallets_owner_unique")
                    .table(Wallets::Table)
                    .col(Wallets::OwnerId)
                    .col(Wallets::OwnerType)
                    .col(Wallets::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_fedimint_operations_fedimint_unique")
                    .table(FedimintOperations::Table)
                    .col(FedimintOperations::FedimintOperationId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop all indexes in reverse order
        let indexes = vec![
            "idx_fedimint_operations_fedimint_unique",
            "idx_wallets_owner_unique",
            "idx_fedimint_operations_expires_at",
            "idx_fedimint_operations_created_at",
            "idx_fedimint_operations_retry",
            "idx_fedimint_operations_status",
            "idx_fedimint_operations_fedimint_id",
            "idx_fedimint_operations_type_status",
            "idx_fedimint_operations_wallet_id",
            "idx_wallet_reserves_wallet_type",
            "idx_wallet_reserves_expires_at",
            "idx_wallet_reserves_reference",
            "idx_wallet_reserves_type",
            "idx_wallet_reserves_wallet_id",
            "idx_wallet_transactions_wallet_created_desc",
            "idx_wallet_transactions_expires_at",
            "idx_wallet_transactions_processed_at",
            "idx_wallet_transactions_created_at",
            "idx_wallet_transactions_fedimint_op",
            "idx_wallet_transactions_counterparty",
            "idx_wallet_transactions_external_id",
            "idx_wallet_transactions_type_status",
            "idx_wallet_transactions_wallet_id",
            "idx_wallets_last_sync_at",
            "idx_wallets_federation_id",
            "idx_wallets_status",
            "idx_wallets_owner",
        ];

        for index_name in indexes {
            manager
                .drop_index(Index::drop().name(index_name).to_owned())
                .await?;
        }

        Ok(())
    }
}

// Table identifiers (reused from the main migration)
#[derive(Iden)]
enum Wallets {
    Table,
    #[iden = "owner_id"]
    OwnerId,
    #[iden = "owner_type"]
    OwnerType,
    Name,
    Status,
    #[iden = "federation_id"]
    FederationId,
    #[iden = "last_sync_at"]
    LastSyncAt,
    #[iden = "created_at"]
    #[allow(dead_code)]
    CreatedAt,
}

#[derive(Iden)]
enum WalletTransactions {
    Table,
    #[iden = "wallet_id"]
    WalletId,
    #[iden = "transaction_type"]
    TransactionType,
    Status,
    #[iden = "external_id"]
    ExternalId,
    #[iden = "counterparty_id"]
    CounterpartyId,
    #[iden = "fedimint_operation_id"]
    FedimintOperationId,
    #[iden = "processed_at"]
    ProcessedAt,
    #[iden = "expires_at"]
    ExpiresAt,
    #[iden = "created_at"]
    CreatedAt,
}

#[derive(Iden)]
enum WalletReserves {
    Table,
    #[iden = "wallet_id"]
    WalletId,
    #[iden = "reserve_type"]
    ReserveType,
    Reference,
    #[iden = "expires_at"]
    ExpiresAt,
}

#[derive(Iden)]
enum FedimintOperations {
    Table,
    #[iden = "wallet_id"]
    WalletId,
    #[iden = "operation_type"]
    OperationType,
    Status,
    #[iden = "fedimint_operation_id"]
    FedimintOperationId,
    #[iden = "retry_count"]
    RetryCount,
    #[iden = "last_retry_at"]
    LastRetryAt,
    #[iden = "expires_at"]
    ExpiresAt,
    #[iden = "created_at"]
    CreatedAt,
}
