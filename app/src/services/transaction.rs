use crate::repositories::{Repositories, RepositoryError};
use ::entity::{audit_logs, share_offers, shares};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct TransactionService {
    repositories: Repositories,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SharePurchaseTransaction {
    pub id: Uuid,
    pub transaction_type: TransactionType,
    pub owner_id: Uuid,
    pub owner_type: shares::OwnerType,
    pub share_offer_id: Uuid,
    pub quantity: rust_decimal::Decimal,
    pub share_value: rust_decimal::Decimal,
    pub total_value: rust_decimal::Decimal,
    pub status: TransactionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareTransferTransaction {
    pub id: Uuid,
    pub transaction_type: TransactionType,
    pub from_owner_id: Uuid,
    pub from_owner_type: shares::OwnerType,
    pub to_owner_id: Uuid,
    pub to_owner_type: shares::OwnerType,
    pub share_offer_id: Uuid,
    pub quantity: rust_decimal::Decimal,
    pub share_value: rust_decimal::Decimal,
    pub total_value: rust_decimal::Decimal,
    pub status: TransactionStatus,
    pub reason: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Purchase,
    Transfer,
    Adjustment,
    Split,
    Merge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
    Reversed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionHistory {
    pub transactions: Vec<TransactionLogEntry>,
    pub total_count: u64,
    pub filters_applied: TransactionFilters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionLogEntry {
    pub transaction_id: Uuid,
    pub transaction_type: TransactionType,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionFilters {
    pub owner_id: Option<Uuid>,
    pub owner_type: Option<shares::OwnerType>,
    pub transaction_type: Option<TransactionType>,
    pub status: Option<TransactionStatus>,
    pub date_from: Option<chrono::DateTime<chrono::Utc>>,
    pub date_to: Option<chrono::DateTime<chrono::Utc>>,
    pub share_offer_id: Option<Uuid>,
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Transaction validation error: {0}")]
    Validation(String),
    #[error("Transaction integrity error: {0}")]
    Integrity(String),
    #[error("Transaction not found: {0}")]
    NotFound(Uuid),
}

pub type TransactionServiceResult<T> = Result<T, TransactionError>;

impl TransactionService {
    pub fn new(repositories: Repositories) -> Self {
        Self { repositories }
    }

    /// Record a share purchase transaction
    pub async fn record_share_purchase(
        &self,
        purchase: &shares::Model,
        _offer: &share_offers::Model,
        created_by: Option<Uuid>,
    ) -> TransactionServiceResult<SharePurchaseTransaction> {
        let transaction_id = Uuid::new_v4();

        // Create audit log entry for the purchase
        let audit_entry = audit_logs::ActiveModel {
            id: Set(Uuid::new_v4()),
            table_name: Set("shares".to_string()),
            record_id: Set(purchase.id),
            operation: Set("purchase".to_string()),
            old_values: Set(None),
            new_values: Set(Some(serde_json::to_value(purchase).map_err(|e| {
                TransactionError::Integrity(format!("Failed to serialize purchase data: {}", e))
            })?)),
            changed_by: Set(created_by),
            changed_at: Set(chrono::Utc::now().into()),
            ip_address: Set(None),
            user_agent: Set(None),
        };

        self.repositories.audit_logs.create(audit_entry).await?;

        let transaction = SharePurchaseTransaction {
            id: transaction_id,
            transaction_type: TransactionType::Purchase,
            owner_id: purchase.owner_id,
            owner_type: purchase.owner_type,
            share_offer_id: purchase.share_offer_id,
            quantity: purchase.share_quantity,
            share_value: purchase.share_value,
            total_value: purchase.total_value,
            status: TransactionStatus::Completed,
            created_at: chrono::Utc::now(),
            created_by,
        };

        Ok(transaction)
    }

    /// Record a share transfer transaction
    pub async fn record_share_transfer(
        &self,
        from_share: Option<&shares::Model>,
        to_share: &shares::Model,
        quantity_transferred: rust_decimal::Decimal,
        reason: Option<String>,
        created_by: Option<Uuid>,
    ) -> TransactionServiceResult<ShareTransferTransaction> {
        let transaction_id = Uuid::new_v4();

        let audit_entry = audit_logs::ActiveModel {
            id: Set(Uuid::new_v4()),
            table_name: Set("shares".to_string()),
            record_id: Set(to_share.id),
            operation: Set("transfer".to_string()),
            old_values: Set(
                from_share.map(|s| serde_json::to_value(s).unwrap_or(serde_json::Value::Null))
            ),
            new_values: Set(Some(serde_json::to_value(to_share).map_err(|e| {
                TransactionError::Integrity(format!("Failed to serialize transfer data: {}", e))
            })?)),
            changed_by: Set(created_by),
            changed_at: Set(chrono::Utc::now().into()),
            ip_address: Set(None),
            user_agent: Set(None),
        };

        self.repositories.audit_logs.create(audit_entry).await?;

        let (from_owner_id, from_owner_type) = if let Some(from) = from_share {
            (from.owner_id, from.owner_type)
        } else {
            // Full transfer case - use the original ownership info from metadata
            (to_share.owner_id, to_share.owner_type)
        };

        let transaction = ShareTransferTransaction {
            id: transaction_id,
            transaction_type: TransactionType::Transfer,
            from_owner_id,
            from_owner_type,
            to_owner_id: to_share.owner_id,
            to_owner_type: to_share.owner_type,
            share_offer_id: to_share.share_offer_id,
            quantity: quantity_transferred,
            share_value: to_share.share_value,
            total_value: quantity_transferred * to_share.share_value,
            status: TransactionStatus::Completed,
            reason,
            created_at: chrono::Utc::now(),
            created_by,
        };

        Ok(transaction)
    }

    /// Get transaction history for a specific owner
    pub async fn get_transaction_history(
        &self,
        filters: TransactionFilters,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> TransactionServiceResult<TransactionHistory> {
        let audit_logs = self
            .repositories
            .audit_logs
            .find_filtered(
                filters.owner_id,
                filters.transaction_type.as_ref().map(|t| match t {
                    TransactionType::Purchase => "purchase",
                    TransactionType::Transfer => "transfer",
                    TransactionType::Adjustment => "adjustment",
                    TransactionType::Split => "split",
                    TransactionType::Merge => "merge",
                }),
                filters.date_from,
                filters.date_to,
                limit,
                offset,
            )
            .await?;

        let transactions = audit_logs
            .into_iter()
            .map(|log| TransactionLogEntry {
                transaction_id: log.id, // Use log.id as transaction_id since we don't have a separate transaction_id field
                transaction_type: self.parse_transaction_type(&log.operation),
                entity_type: log.table_name,
                entity_id: log.record_id,
                action: log.operation,
                old_value: log.old_values,
                new_value: log.new_values,
                metadata: None, // No metadata field in current audit_logs structure
                created_at: log.changed_at.naive_utc().and_utc(),
                created_by: log.changed_by,
            })
            .collect();

        let total_count = self
            .repositories
            .audit_logs
            .count_filtered(
                filters.owner_id,
                filters.transaction_type.as_ref().map(|t| match t {
                    TransactionType::Purchase => "purchase",
                    TransactionType::Transfer => "transfer",
                    TransactionType::Adjustment => "adjustment",
                    TransactionType::Split => "split",
                    TransactionType::Merge => "merge",
                }),
                filters.date_from,
                filters.date_to,
            )
            .await?;

        Ok(TransactionHistory {
            transactions,
            total_count,
            filters_applied: filters,
        })
    }

    /// Get transaction summary for an owner
    pub async fn get_transaction_summary(
        &self,
        owner_id: Uuid,
        owner_type: shares::OwnerType,
    ) -> TransactionServiceResult<TransactionSummary> {
        let shares = self
            .repositories
            .shares
            .find_by_owner(owner_id, owner_type)
            .await?;

        let total_purchases = shares.len() as u64;
        let total_quantity: rust_decimal::Decimal = shares.iter().map(|s| s.share_quantity).sum();
        let total_value: rust_decimal::Decimal = shares.iter().map(|s| s.total_value).sum();

        // Count transfers where this owner was involved
        let transfer_count = self
            .repositories
            .audit_logs
            .count_transfers_for_owner(owner_id)
            .await?;

        Ok(TransactionSummary {
            owner_id,
            owner_type,
            total_purchases,
            total_transfers: transfer_count,
            total_share_quantity: total_quantity,
            total_investment_value: total_value,
            first_purchase_date: shares
                .iter()
                .map(|s| s.created_at.naive_utc().and_utc())
                .min(),
            last_transaction_date: shares
                .iter()
                .filter_map(|s| s.last_transaction_at.map(|t| t.naive_utc().and_utc()))
                .max(),
        })
    }

    /// Validate transaction integrity
    pub async fn validate_transaction_integrity(
        &self,
        transaction_id: Uuid,
    ) -> TransactionServiceResult<bool> {
        let audit_logs = self
            .repositories
            .audit_logs
            .find_by_transaction_id(transaction_id)
            .await?;

        if audit_logs.is_empty() {
            return Err(TransactionError::NotFound(transaction_id));
        }

        // Validate that all related audit logs have consistent data
        for log in &audit_logs {
            if log.id != transaction_id {
                return Ok(false);
            }

            // Additional integrity checks can be added here
            // For example, validating that share quantities match expected values
        }

        Ok(true)
    }

    fn parse_transaction_type(&self, action: &str) -> TransactionType {
        match action {
            "purchase" => TransactionType::Purchase,
            "transfer" => TransactionType::Transfer,
            "adjustment" => TransactionType::Adjustment,
            "split" => TransactionType::Split,
            "merge" => TransactionType::Merge,
            _ => TransactionType::Adjustment, // Default fallback
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub owner_id: Uuid,
    pub owner_type: shares::OwnerType,
    pub total_purchases: u64,
    pub total_transfers: u64,
    pub total_share_quantity: rust_decimal::Decimal,
    pub total_investment_value: rust_decimal::Decimal,
    pub first_purchase_date: Option<chrono::DateTime<chrono::Utc>>,
    pub last_transaction_date: Option<chrono::DateTime<chrono::Utc>>,
}
