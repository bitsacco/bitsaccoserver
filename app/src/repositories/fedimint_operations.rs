use sea_orm::*;
use std::sync::Arc;
use uuid::Uuid;

use super::{RepositoryError, RepositoryResult};
use ::entity::{
    fedimint_operations,
    sea_orm_active_enums::{FedimintOperationType, TransactionStatus},
};

#[derive(Clone)]
pub struct FedimintOperationRepository {
    db: Arc<DatabaseConnection>,
}

impl FedimintOperationRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Create a new Fedimint operation
    pub async fn create(
        &self,
        operation: fedimint_operations::ActiveModel,
    ) -> RepositoryResult<fedimint_operations::Model> {
        let result = operation.insert(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Find operation by ID
    pub async fn find_by_id(
        &self,
        id: Uuid,
    ) -> RepositoryResult<Option<fedimint_operations::Model>> {
        let operation = fedimint_operations::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await?;
        Ok(operation)
    }

    /// Find operation by Fedimint operation ID
    pub async fn find_by_fedimint_id(
        &self,
        fedimint_id: String,
    ) -> RepositoryResult<Option<fedimint_operations::Model>> {
        let operation = fedimint_operations::Entity::find()
            .filter(fedimint_operations::Column::FedimintOperationId.eq(fedimint_id))
            .one(self.db.as_ref())
            .await?;
        Ok(operation)
    }

    /// Find operations by wallet ID
    pub async fn find_by_wallet(
        &self,
        wallet_id: Uuid,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> RepositoryResult<Vec<fedimint_operations::Model>> {
        let mut query = fedimint_operations::Entity::find()
            .filter(fedimint_operations::Column::WalletId.eq(wallet_id))
            .order_by_desc(fedimint_operations::Column::CreatedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let operations = query.all(self.db.as_ref()).await?;
        Ok(operations)
    }

    /// Find operations by status
    pub async fn find_by_status(
        &self,
        status: TransactionStatus,
    ) -> RepositoryResult<Vec<fedimint_operations::Model>> {
        let operations = fedimint_operations::Entity::find()
            .filter(fedimint_operations::Column::Status.eq(status))
            .order_by_desc(fedimint_operations::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;
        Ok(operations)
    }

    /// Find operations by type and wallet
    pub async fn find_by_type_and_wallet(
        &self,
        wallet_id: Uuid,
        operation_type: FedimintOperationType,
        limit: Option<u64>,
    ) -> RepositoryResult<Vec<fedimint_operations::Model>> {
        let mut query = fedimint_operations::Entity::find()
            .filter(fedimint_operations::Column::WalletId.eq(wallet_id))
            .filter(fedimint_operations::Column::OperationType.eq(operation_type))
            .order_by_desc(fedimint_operations::Column::CreatedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        let operations = query.all(self.db.as_ref()).await?;
        Ok(operations)
    }

    /// Find pending operations that need retry
    pub async fn find_pending_retries(
        &self,
        max_retries: i32,
    ) -> RepositoryResult<Vec<fedimint_operations::Model>> {
        let operations = fedimint_operations::Entity::find()
            .filter(fedimint_operations::Column::Status.eq(TransactionStatus::Pending))
            .filter(fedimint_operations::Column::RetryCount.lt(max_retries))
            .order_by_asc(fedimint_operations::Column::LastRetryAt)
            .all(self.db.as_ref())
            .await?;
        Ok(operations)
    }

    /// Find expired operations
    pub async fn find_expired(&self) -> RepositoryResult<Vec<fedimint_operations::Model>> {
        let now = chrono::Utc::now();

        let operations = fedimint_operations::Entity::find()
            .filter(
                Condition::any()
                    .add(fedimint_operations::Column::Status.eq(TransactionStatus::Pending))
                    .add(fedimint_operations::Column::Status.eq(TransactionStatus::Processing)),
            )
            .filter(fedimint_operations::Column::ExpiresAt.is_not_null())
            .filter(fedimint_operations::Column::ExpiresAt.lt(now))
            .all(self.db.as_ref())
            .await?;
        Ok(operations)
    }

    /// Update operation
    pub async fn update(
        &self,
        operation: fedimint_operations::ActiveModel,
    ) -> RepositoryResult<fedimint_operations::Model> {
        let result = operation.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Update operation status
    pub async fn update_status(
        &self,
        operation_id: Uuid,
        status: TransactionStatus,
        error_details: Option<String>,
    ) -> RepositoryResult<fedimint_operations::Model> {
        let operation = self
            .find_by_id(operation_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut operation: fedimint_operations::ActiveModel = operation.into();
        operation.status = Set(status);
        operation.updated_at = Set(chrono::Utc::now().into());

        if let Some(error) = error_details {
            operation.error_details = Set(Some(error));
        }

        if status == TransactionStatus::Completed || status == TransactionStatus::Failed {
            operation.processed_at = Set(Some(chrono::Utc::now().into()));
        }

        let result = operation.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Update operation response data
    pub async fn update_response(
        &self,
        operation_id: Uuid,
        response_data: serde_json::Value,
    ) -> RepositoryResult<fedimint_operations::Model> {
        let operation = self
            .find_by_id(operation_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut operation: fedimint_operations::ActiveModel = operation.into();
        operation.response = Set(Some(response_data));
        operation.updated_at = Set(chrono::Utc::now().into());

        let result = operation.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Increment retry count and update last retry timestamp
    pub async fn increment_retry_count(
        &self,
        operation_id: Uuid,
    ) -> RepositoryResult<fedimint_operations::Model> {
        let operation = self
            .find_by_id(operation_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut operation: fedimint_operations::ActiveModel = operation.into();
        operation.retry_count = Set(operation.retry_count.as_ref() + 1);
        operation.last_retry_at = Set(Some(chrono::Utc::now().into()));
        operation.updated_at = Set(chrono::Utc::now().into());

        let result = operation.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Get operation statistics for a wallet
    pub async fn get_wallet_stats(
        &self,
        wallet_id: Uuid,
        days: Option<i64>,
    ) -> RepositoryResult<FedimintOperationStats> {
        let mut query = fedimint_operations::Entity::find()
            .filter(fedimint_operations::Column::WalletId.eq(wallet_id));

        if let Some(days) = days {
            let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
            query = query.filter(fedimint_operations::Column::CreatedAt.gte(cutoff));
        }

        let operations = query.all(self.db.as_ref()).await?;

        let mut stats = FedimintOperationStats {
            wallet_id,
            total_operations: operations.len() as u64,
            pending_count: 0,
            completed_count: 0,
            failed_count: 0,
            deposit_count: 0,
            withdraw_count: 0,
            lightning_count: 0,
            total_amount_msat: 0,
            total_fees_msat: 0,
            avg_retry_count: 0.0,
            period_days: days,
        };

        let mut total_retries = 0i32;

        for op in &operations {
            match op.status {
                TransactionStatus::Pending | TransactionStatus::Processing => {
                    stats.pending_count += 1
                }
                TransactionStatus::Completed => stats.completed_count += 1,
                TransactionStatus::Failed
                | TransactionStatus::Cancelled
                | TransactionStatus::Expired => stats.failed_count += 1,
            }

            match op.operation_type {
                FedimintOperationType::Deposit => stats.deposit_count += 1,
                FedimintOperationType::Withdraw => stats.withdraw_count += 1,
                FedimintOperationType::Lightning => stats.lightning_count += 1,
                _ => {}
            }

            if let Some(amount) = op.amount_msat {
                stats.total_amount_msat += amount;
            }

            if let Some(fee) = op.fee_msat {
                stats.total_fees_msat += fee;
            }

            total_retries += op.retry_count;
        }

        if !operations.is_empty() {
            stats.avg_retry_count = total_retries as f64 / operations.len() as f64;
        }

        Ok(stats)
    }

    /// Count operations by wallet
    pub async fn count_by_wallet(&self, wallet_id: Uuid) -> RepositoryResult<u64> {
        let count = fedimint_operations::Entity::find()
            .filter(fedimint_operations::Column::WalletId.eq(wallet_id))
            .count(self.db.as_ref())
            .await?;
        Ok(count)
    }

    /// Count operations by status and wallet
    pub async fn count_by_status_and_wallet(
        &self,
        wallet_id: Uuid,
        status: TransactionStatus,
    ) -> RepositoryResult<u64> {
        let count = fedimint_operations::Entity::find()
            .filter(fedimint_operations::Column::WalletId.eq(wallet_id))
            .filter(fedimint_operations::Column::Status.eq(status))
            .count(self.db.as_ref())
            .await?;
        Ok(count)
    }

    /// Get latest operation for wallet
    pub async fn get_latest_for_wallet(
        &self,
        wallet_id: Uuid,
    ) -> RepositoryResult<Option<fedimint_operations::Model>> {
        let operation = fedimint_operations::Entity::find()
            .filter(fedimint_operations::Column::WalletId.eq(wallet_id))
            .order_by_desc(fedimint_operations::Column::CreatedAt)
            .one(self.db.as_ref())
            .await?;
        Ok(operation)
    }

    /// Mark expired operations as failed
    pub async fn mark_expired_as_failed(&self) -> RepositoryResult<u64> {
        let expired_ops = self.find_expired().await?;
        let mut count = 0u64;

        for op in expired_ops {
            let mut op: fedimint_operations::ActiveModel = op.into();
            op.status = Set(TransactionStatus::Expired);
            op.error_details = Set(Some("Operation expired".to_string()));
            op.processed_at = Set(Some(chrono::Utc::now().into()));
            op.updated_at = Set(chrono::Utc::now().into());

            op.update(self.db.as_ref()).await?;
            count += 1;
        }

        Ok(count)
    }

    /// Delete operation (should rarely be used)
    pub async fn delete(&self, operation_id: Uuid) -> RepositoryResult<()> {
        fedimint_operations::Entity::delete_by_id(operation_id)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }
}

/// Statistics for Fedimint operations
#[derive(Debug, Clone)]
pub struct FedimintOperationStats {
    pub wallet_id: Uuid,
    pub total_operations: u64,
    pub pending_count: u64,
    pub completed_count: u64,
    pub failed_count: u64,
    pub deposit_count: u64,
    pub withdraw_count: u64,
    pub lightning_count: u64,
    pub total_amount_msat: i64,
    pub total_fees_msat: i64,
    pub avg_retry_count: f64,
    pub period_days: Option<i64>,
}
