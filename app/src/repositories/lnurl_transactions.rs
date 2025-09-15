use sea_orm::*;
use std::sync::Arc;
use uuid::Uuid;

use ::entity::{lnurl_transactions, sea_orm_active_enums::{LnurlTransactionStatus, LnurlTransactionType}};
use super::{RepositoryError, RepositoryResult};

#[derive(Clone)]
pub struct LnurlTransactionRepository {
    db: Arc<DatabaseConnection>,
}

impl LnurlTransactionRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Create a new LNURL transaction
    pub async fn create(&self, transaction: lnurl_transactions::ActiveModel) -> RepositoryResult<lnurl_transactions::Model> {
        let result = transaction.insert(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Find LNURL transaction by ID
    pub async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<lnurl_transactions::Model>> {
        let transaction = lnurl_transactions::Entity::find_by_id(id).one(self.db.as_ref()).await?;
        Ok(transaction)
    }

    /// Find LNURL transaction by k1 (LNURL-auth identifier)
    pub async fn find_by_k1(&self, k1: &str) -> RepositoryResult<Option<lnurl_transactions::Model>> {
        let transaction = lnurl_transactions::Entity::find()
            .filter(lnurl_transactions::Column::K1.eq(k1))
            .one(self.db.as_ref())
            .await?;
        Ok(transaction)
    }

    /// Find LNURL transaction by payment hash
    pub async fn find_by_payment_hash(&self, payment_hash: &str) -> RepositoryResult<Option<lnurl_transactions::Model>> {
        let transaction = lnurl_transactions::Entity::find()
            .filter(lnurl_transactions::Column::PaymentHash.eq(payment_hash))
            .one(self.db.as_ref())
            .await?;
        Ok(transaction)
    }

    /// Find LNURL transactions by lightning address
    pub async fn find_by_lightning_address(&self, address_id: Uuid) -> RepositoryResult<Vec<lnurl_transactions::Model>> {
        let transactions = lnurl_transactions::Entity::find()
            .filter(lnurl_transactions::Column::LightningAddressId.eq(address_id))
            .order_by_desc(lnurl_transactions::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;
        Ok(transactions)
    }

    /// Find LNURL transactions by wallet transaction ID
    pub async fn find_by_wallet_transaction(&self, wallet_transaction_id: Uuid) -> RepositoryResult<Option<lnurl_transactions::Model>> {
        let transaction = lnurl_transactions::Entity::find()
            .filter(lnurl_transactions::Column::WalletTransactionId.eq(wallet_transaction_id))
            .one(self.db.as_ref())
            .await?;
        Ok(transaction)
    }

    /// Find LNURL transactions by type and status
    pub async fn find_by_type_and_status(
        &self,
        transaction_type: LnurlTransactionType,
        status: LnurlTransactionStatus,
    ) -> RepositoryResult<Vec<lnurl_transactions::Model>> {
        let transactions = lnurl_transactions::Entity::find()
            .filter(lnurl_transactions::Column::TransactionType.eq(transaction_type))
            .filter(lnurl_transactions::Column::Status.eq(status))
            .order_by_desc(lnurl_transactions::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;
        Ok(transactions)
    }

    /// Find pending LNURL transactions that have expired
    pub async fn find_expired_pending(&self) -> RepositoryResult<Vec<lnurl_transactions::Model>> {
        let now = chrono::Utc::now();
        
        let transactions = lnurl_transactions::Entity::find()
            .filter(lnurl_transactions::Column::Status.eq(LnurlTransactionStatus::Pending))
            .filter(lnurl_transactions::Column::ExpiresAt.lt(now))
            .all(self.db.as_ref())
            .await?;
        Ok(transactions)
    }

    /// Update LNURL transaction status
    pub async fn update_status(
        &self,
        transaction_id: Uuid,
        status: LnurlTransactionStatus,
        error_details: Option<String>,
    ) -> RepositoryResult<lnurl_transactions::Model> {
        let transaction = self
            .find_by_id(transaction_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut transaction: lnurl_transactions::ActiveModel = transaction.into();
        transaction.status = Set(status);
        transaction.updated_at = Set(chrono::Utc::now().into());
        
        if let Some(error) = error_details {
            transaction.error_details = Set(Some(error));
        }

        // Set processed_at if status is completed or failed
        match status {
            LnurlTransactionStatus::Completed | LnurlTransactionStatus::Failed | LnurlTransactionStatus::Expired => {
                transaction.processed_at = Set(Some(chrono::Utc::now().into()));
            }
            _ => {}
        }

        let result = transaction.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Update LNURL transaction with payment details
    pub async fn update_payment_details(
        &self,
        transaction_id: Uuid,
        invoice: Option<String>,
        payment_hash: Option<String>,
        preimage: Option<String>,
        amount_msat: Option<i64>,
    ) -> RepositoryResult<lnurl_transactions::Model> {
        let transaction = self
            .find_by_id(transaction_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut transaction: lnurl_transactions::ActiveModel = transaction.into();
        
        if let Some(inv) = invoice {
            transaction.invoice = Set(Some(inv));
        }
        if let Some(hash) = payment_hash {
            transaction.payment_hash = Set(Some(hash));
        }
        if let Some(pre) = preimage {
            transaction.preimage = Set(Some(pre));
        }
        if let Some(amount) = amount_msat {
            transaction.amount_msat = Set(Some(amount));
        }
        
        transaction.updated_at = Set(chrono::Utc::now().into());

        let result = transaction.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Update LNURL transaction
    pub async fn update(&self, transaction: lnurl_transactions::ActiveModel) -> RepositoryResult<lnurl_transactions::Model> {
        let result = transaction.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Get transaction history for a lightning address with pagination
    pub async fn get_history(
        &self,
        address_id: Uuid,
        limit: u64,
        offset: u64,
    ) -> RepositoryResult<Vec<lnurl_transactions::Model>> {
        let transactions = lnurl_transactions::Entity::find()
            .filter(lnurl_transactions::Column::LightningAddressId.eq(address_id))
            .order_by_desc(lnurl_transactions::Column::CreatedAt)
            .limit(limit)
            .offset(offset)
            .all(self.db.as_ref())
            .await?;
        Ok(transactions)
    }

    /// Count transactions for a lightning address
    pub async fn count_by_address(&self, address_id: Uuid) -> RepositoryResult<u64> {
        let count = lnurl_transactions::Entity::find()
            .filter(lnurl_transactions::Column::LightningAddressId.eq(address_id))
            .count(self.db.as_ref())
            .await?;
        Ok(count)
    }

    /// Get total amount received for a lightning address
    pub async fn get_total_received(&self, address_id: Uuid) -> RepositoryResult<i64> {
        let result: Option<i64> = lnurl_transactions::Entity::find()
            .filter(lnurl_transactions::Column::LightningAddressId.eq(address_id))
            .filter(lnurl_transactions::Column::Status.eq(LnurlTransactionStatus::Completed))
            .filter(lnurl_transactions::Column::TransactionType.eq(LnurlTransactionType::Pay))
            .select_only()
            .column_as(lnurl_transactions::Column::AmountMsat.sum(), "total_received")
            .into_tuple()
            .one(self.db.as_ref())
            .await?;

        Ok(result.unwrap_or(0))
    }

    /// Get recent transactions count for rate limiting
    pub async fn get_recent_count(
        &self,
        address_id: Uuid,
        since_minutes: i64,
    ) -> RepositoryResult<u64> {
        let cutoff = chrono::Utc::now() - chrono::Duration::minutes(since_minutes);
        
        let count = lnurl_transactions::Entity::find()
            .filter(lnurl_transactions::Column::LightningAddressId.eq(address_id))
            .filter(lnurl_transactions::Column::CreatedAt.gte(cutoff))
            .count(self.db.as_ref())
            .await?;
        Ok(count)
    }

    /// Mark expired transactions
    pub async fn mark_expired(&self) -> RepositoryResult<u64> {
        let expired_transactions = self.find_expired_pending().await?;
        let mut count = 0;

        for transaction in expired_transactions {
            self.update_status(
                transaction.id,
                LnurlTransactionStatus::Expired,
                Some("Transaction expired".to_string()),
            ).await?;
            count += 1;
        }

        Ok(count)
    }

    /// Delete old transactions (for cleanup)
    pub async fn delete_old(&self, days_old: i64) -> RepositoryResult<u64> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days_old);
        
        let result = lnurl_transactions::Entity::delete_many()
            .filter(lnurl_transactions::Column::CreatedAt.lt(cutoff))
            .filter(lnurl_transactions::Column::Status.ne(LnurlTransactionStatus::Pending))
            .filter(lnurl_transactions::Column::Status.ne(LnurlTransactionStatus::Processing))
            .exec(self.db.as_ref())
            .await?;

        Ok(result.rows_affected)
    }
}