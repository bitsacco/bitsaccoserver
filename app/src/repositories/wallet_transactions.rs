use sea_orm::*;
use std::sync::Arc;
use uuid::Uuid;

use super::{RepositoryError, RepositoryResult};
use ::entity::{
    sea_orm_active_enums::{TransactionStatus, TransactionType},
    wallet_transactions,
};

#[derive(Clone)]
pub struct WalletTransactionRepository {
    db: Arc<DatabaseConnection>,
}

impl WalletTransactionRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Create a new wallet transaction
    pub async fn create(
        &self,
        transaction: wallet_transactions::ActiveModel,
    ) -> RepositoryResult<wallet_transactions::Model> {
        let result = transaction.insert(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Find transaction by ID
    pub async fn find_by_id(
        &self,
        id: Uuid,
    ) -> RepositoryResult<Option<wallet_transactions::Model>> {
        let transaction = wallet_transactions::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await?;
        Ok(transaction)
    }

    /// Find transactions by wallet ID
    pub async fn find_by_wallet(
        &self,
        wallet_id: Uuid,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> RepositoryResult<Vec<wallet_transactions::Model>> {
        let mut query = wallet_transactions::Entity::find()
            .filter(wallet_transactions::Column::WalletId.eq(wallet_id))
            .order_by_desc(wallet_transactions::Column::CreatedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let transactions = query.all(self.db.as_ref()).await?;
        Ok(transactions)
    }

    /// Find transactions by external ID (e.g., Lightning invoice payment hash)
    pub async fn find_by_external_id(
        &self,
        external_id: String,
    ) -> RepositoryResult<Vec<wallet_transactions::Model>> {
        let transactions = wallet_transactions::Entity::find()
            .filter(wallet_transactions::Column::ExternalId.eq(external_id))
            .all(self.db.as_ref())
            .await?;
        Ok(transactions)
    }

    /// Find transactions by status
    pub async fn find_by_status(
        &self,
        status: TransactionStatus,
    ) -> RepositoryResult<Vec<wallet_transactions::Model>> {
        let transactions = wallet_transactions::Entity::find()
            .filter(wallet_transactions::Column::Status.eq(status))
            .order_by_desc(wallet_transactions::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;
        Ok(transactions)
    }

    /// Find pending transactions older than specified duration
    pub async fn find_expired_pending(
        &self,
        hours_old: i64,
    ) -> RepositoryResult<Vec<wallet_transactions::Model>> {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(hours_old);

        let transactions = wallet_transactions::Entity::find()
            .filter(wallet_transactions::Column::Status.eq(TransactionStatus::Pending))
            .filter(wallet_transactions::Column::CreatedAt.lt(cutoff))
            .all(self.db.as_ref())
            .await?;
        Ok(transactions)
    }

    /// Find transactions by counterparty
    pub async fn find_by_counterparty(
        &self,
        counterparty_id: Uuid,
    ) -> RepositoryResult<Vec<wallet_transactions::Model>> {
        let transactions = wallet_transactions::Entity::find()
            .filter(wallet_transactions::Column::CounterpartyId.eq(counterparty_id))
            .order_by_desc(wallet_transactions::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;
        Ok(transactions)
    }

    /// Find transactions by Fedimint operation ID
    pub async fn find_by_fedimint_operation(
        &self,
        operation_id: Uuid,
    ) -> RepositoryResult<Option<wallet_transactions::Model>> {
        let transaction = wallet_transactions::Entity::find()
            .filter(wallet_transactions::Column::FedimintOperationId.eq(operation_id))
            .one(self.db.as_ref())
            .await?;
        Ok(transaction)
    }

    /// Find transactions by type and wallet
    pub async fn find_by_type_and_wallet(
        &self,
        wallet_id: Uuid,
        transaction_type: TransactionType,
        limit: Option<u64>,
    ) -> RepositoryResult<Vec<wallet_transactions::Model>> {
        let mut query = wallet_transactions::Entity::find()
            .filter(wallet_transactions::Column::WalletId.eq(wallet_id))
            .filter(wallet_transactions::Column::TransactionType.eq(transaction_type))
            .order_by_desc(wallet_transactions::Column::CreatedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        let transactions = query.all(self.db.as_ref()).await?;
        Ok(transactions)
    }

    /// Update transaction
    pub async fn update(
        &self,
        transaction: wallet_transactions::ActiveModel,
    ) -> RepositoryResult<wallet_transactions::Model> {
        let result = transaction.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Update transaction status
    pub async fn update_status(
        &self,
        transaction_id: Uuid,
        status: TransactionStatus,
    ) -> RepositoryResult<wallet_transactions::Model> {
        let transaction = self
            .find_by_id(transaction_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut transaction: wallet_transactions::ActiveModel = transaction.into();
        transaction.status = Set(status);
        transaction.updated_at = Set(chrono::Utc::now().into());

        if status == TransactionStatus::Completed || status == TransactionStatus::Failed {
            transaction.processed_at = Set(Some(chrono::Utc::now().into()));
        }

        let result = transaction.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Update transaction with external ID (for tracking Lightning payments, etc.)
    pub async fn update_external_id(
        &self,
        transaction_id: Uuid,
        external_id: String,
    ) -> RepositoryResult<wallet_transactions::Model> {
        let transaction = self
            .find_by_id(transaction_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut transaction: wallet_transactions::ActiveModel = transaction.into();
        transaction.external_id = Set(Some(external_id));
        transaction.updated_at = Set(chrono::Utc::now().into());

        let result = transaction.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Get transaction summary for a wallet
    pub async fn get_wallet_summary(
        &self,
        wallet_id: Uuid,
        days: Option<i64>,
    ) -> RepositoryResult<WalletTransactionSummary> {
        let mut query = wallet_transactions::Entity::find()
            .filter(wallet_transactions::Column::WalletId.eq(wallet_id))
            .filter(wallet_transactions::Column::Status.eq(TransactionStatus::Completed));

        if let Some(days) = days {
            let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
            query = query.filter(wallet_transactions::Column::CreatedAt.gte(cutoff));
        }

        let transactions = query.all(self.db.as_ref()).await?;

        let mut total_in = 0i64;
        let mut total_out = 0i64;
        let mut total_fees = 0i64;
        let mut deposit_count = 0u64;
        let mut withdraw_count = 0u64;
        let mut payment_count = 0u64;

        for tx in &transactions {
            total_fees += tx.fee_msat;

            match tx.transaction_type {
                TransactionType::Deposit => {
                    total_in += tx.amount_msat;
                    deposit_count += 1;
                }
                TransactionType::Withdraw => {
                    total_out += tx.amount_msat;
                    withdraw_count += 1;
                }
                TransactionType::Payment => {
                    total_out += tx.amount_msat;
                    payment_count += 1;
                }
                TransactionType::Transfer => {
                    // For transfers, amount could be in or out depending on the wallet
                    // This needs more context about the transfer direction
                    if tx.amount_msat > 0 {
                        total_in += tx.amount_msat;
                    } else {
                        total_out += tx.amount_msat.abs();
                    }
                }
                _ => {
                    // Handle other transaction types as needed
                }
            }
        }

        Ok(WalletTransactionSummary {
            wallet_id,
            total_transactions: transactions.len() as u64,
            total_in_msat: total_in,
            total_out_msat: total_out,
            total_fees_msat: total_fees,
            deposit_count,
            withdraw_count,
            payment_count,
            period_days: days,
        })
    }

    /// Count transactions by wallet
    pub async fn count_by_wallet(&self, wallet_id: Uuid) -> RepositoryResult<u64> {
        let count = wallet_transactions::Entity::find()
            .filter(wallet_transactions::Column::WalletId.eq(wallet_id))
            .count(self.db.as_ref())
            .await?;
        Ok(count)
    }

    /// Count transactions by status and wallet
    pub async fn count_by_status_and_wallet(
        &self,
        wallet_id: Uuid,
        status: TransactionStatus,
    ) -> RepositoryResult<u64> {
        let count = wallet_transactions::Entity::find()
            .filter(wallet_transactions::Column::WalletId.eq(wallet_id))
            .filter(wallet_transactions::Column::Status.eq(status))
            .count(self.db.as_ref())
            .await?;
        Ok(count)
    }

    /// Get latest transaction for wallet
    pub async fn get_latest_for_wallet(
        &self,
        wallet_id: Uuid,
    ) -> RepositoryResult<Option<wallet_transactions::Model>> {
        let transaction = wallet_transactions::Entity::find()
            .filter(wallet_transactions::Column::WalletId.eq(wallet_id))
            .order_by_desc(wallet_transactions::Column::CreatedAt)
            .one(self.db.as_ref())
            .await?;
        Ok(transaction)
    }

    /// Delete transaction (should rarely be used, prefer status updates)
    pub async fn delete(&self, transaction_id: Uuid) -> RepositoryResult<()> {
        wallet_transactions::Entity::delete_by_id(transaction_id)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }
}

/// Summary statistics for wallet transactions
#[derive(Debug, Clone)]
pub struct WalletTransactionSummary {
    pub wallet_id: Uuid,
    pub total_transactions: u64,
    pub total_in_msat: i64,
    pub total_out_msat: i64,
    pub total_fees_msat: i64,
    pub deposit_count: u64,
    pub withdraw_count: u64,
    pub payment_count: u64,
    pub period_days: Option<i64>,
}
