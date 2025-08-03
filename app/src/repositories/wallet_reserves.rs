use sea_orm::*;
use std::sync::Arc;
use uuid::Uuid;

use ::entity::{wallet_reserves, sea_orm_active_enums::ReserveType};
use super::{RepositoryError, RepositoryResult};

#[derive(Clone)]
pub struct WalletReserveRepository {
    db: Arc<DatabaseConnection>,
}

impl WalletReserveRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Create a new wallet reserve
    pub async fn create(&self, reserve: wallet_reserves::ActiveModel) -> RepositoryResult<wallet_reserves::Model> {
        let result = reserve.insert(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Find reserve by ID
    pub async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<wallet_reserves::Model>> {
        let reserve = wallet_reserves::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await?;
        Ok(reserve)
    }

    /// Find reserves by wallet ID
    pub async fn find_by_wallet(&self, wallet_id: Uuid) -> RepositoryResult<Vec<wallet_reserves::Model>> {
        let reserves = wallet_reserves::Entity::find()
            .filter(wallet_reserves::Column::WalletId.eq(wallet_id))
            .order_by_desc(wallet_reserves::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;
        Ok(reserves)
    }

    /// Find reserves by wallet and type
    pub async fn find_by_wallet_and_type(
        &self,
        wallet_id: Uuid,
        reserve_type: ReserveType,
    ) -> RepositoryResult<Vec<wallet_reserves::Model>> {
        let reserves = wallet_reserves::Entity::find()
            .filter(wallet_reserves::Column::WalletId.eq(wallet_id))
            .filter(wallet_reserves::Column::ReserveType.eq(reserve_type))
            .order_by_desc(wallet_reserves::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;
        Ok(reserves)
    }

    /// Find reserve by reference (transaction ID, operation ID, etc.)
    pub async fn find_by_reference(&self, reference: String) -> RepositoryResult<Option<wallet_reserves::Model>> {
        let reserve = wallet_reserves::Entity::find()
            .filter(wallet_reserves::Column::Reference.eq(reference))
            .one(self.db.as_ref())
            .await?;
        Ok(reserve)
    }

    /// Find expired reserves
    pub async fn find_expired(&self) -> RepositoryResult<Vec<wallet_reserves::Model>> {
        let now = chrono::Utc::now();
        
        let reserves = wallet_reserves::Entity::find()
            .filter(wallet_reserves::Column::ExpiresAt.is_not_null())
            .filter(wallet_reserves::Column::ExpiresAt.lt(now))
            .all(self.db.as_ref())
            .await?;
        Ok(reserves)
    }

    /// Update reserve
    pub async fn update(&self, reserve: wallet_reserves::ActiveModel) -> RepositoryResult<wallet_reserves::Model> {
        let result = reserve.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Update reserve amount
    pub async fn update_amount(
        &self,
        reserve_id: Uuid,
        new_amount: i64,
    ) -> RepositoryResult<wallet_reserves::Model> {
        let reserve = self
            .find_by_id(reserve_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut reserve: wallet_reserves::ActiveModel = reserve.into();
        reserve.amount_msat = Set(new_amount);
        reserve.updated_at = Set(chrono::Utc::now().into());

        let result = reserve.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Get total reserves by wallet and type
    pub async fn get_total_by_wallet_and_type(
        &self,
        wallet_id: Uuid,
        reserve_type: ReserveType,
    ) -> RepositoryResult<i64> {
        let result: Option<i64> = wallet_reserves::Entity::find()
            .filter(wallet_reserves::Column::WalletId.eq(wallet_id))
            .filter(wallet_reserves::Column::ReserveType.eq(reserve_type))
            .select_only()
            .column_as(wallet_reserves::Column::AmountMsat.sum(), "total_amount")
            .into_tuple()
            .one(self.db.as_ref())
            .await?;

        Ok(result.unwrap_or(0))
    }

    /// Get reserve summary for a wallet
    pub async fn get_wallet_summary(&self, wallet_id: Uuid) -> RepositoryResult<WalletReserveSummary> {
        let available = self
            .get_total_by_wallet_and_type(wallet_id, ReserveType::Available)
            .await?;

        let pending = self
            .get_total_by_wallet_and_type(wallet_id, ReserveType::Pending)
            .await?;

        let locked = self
            .get_total_by_wallet_and_type(wallet_id, ReserveType::Locked)
            .await?;

        let emergency = self
            .get_total_by_wallet_and_type(wallet_id, ReserveType::Emergency)
            .await?;

        Ok(WalletReserveSummary {
            wallet_id,
            available_msat: available,
            pending_msat: pending,
            locked_msat: locked,
            emergency_msat: emergency,
            total_msat: available + pending + locked + emergency,
        })
    }

    /// Create or update a reserve
    pub async fn upsert_reserve(
        &self,
        wallet_id: Uuid,
        reserve_type: ReserveType,
        amount_msat: i64,
        reference: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        created_by: Option<Uuid>,
    ) -> RepositoryResult<wallet_reserves::Model> {
        // Try to find existing reserve with the same reference if provided
        if let Some(ref_str) = &reference {
            if let Some(existing) = self.find_by_reference(ref_str.clone()).await? {
                // Update existing reserve
                let mut reserve: wallet_reserves::ActiveModel = existing.into();
                reserve.amount_msat = Set(amount_msat);
                reserve.updated_at = Set(chrono::Utc::now().into());
                return self.update(reserve).await;
            }
        }

        // Create new reserve
        let reserve = wallet_reserves::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_id: Set(wallet_id),
            reserve_type: Set(reserve_type),
            amount_msat: Set(amount_msat),
            reference: Set(reference),
            expires_at: Set(expires_at.map(Into::into)),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(created_by),
            updated_by: Set(created_by),
        };

        self.create(reserve).await
    }

    /// Transfer reserves (atomically move amount from one type to another)
    pub async fn transfer_reserves(
        &self,
        wallet_id: Uuid,
        from_type: ReserveType,
        to_type: ReserveType,
        amount_msat: i64,
        reference: Option<String>,
        created_by: Option<Uuid>,
    ) -> RepositoryResult<(wallet_reserves::Model, wallet_reserves::Model)> {
        let txn = self.db.begin().await?;

        // Check if sufficient funds in source reserve type
        let available_amount = self
            .get_total_by_wallet_and_type(wallet_id, from_type.clone())
            .await?;

        if available_amount < amount_msat {
            return Err(RepositoryError::Validation(format!(
                "Insufficient {} reserves: {} < {}",
                reserve_type_to_string(from_type),
                available_amount,
                amount_msat
            )));
        }

        // Deduct from source reserve type
        let deduction = wallet_reserves::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_id: Set(wallet_id),
            reserve_type: Set(from_type),
            amount_msat: Set(-amount_msat), // Negative amount for deduction
            reference: Set(reference.clone()),
            expires_at: Set(None),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(created_by),
            updated_by: Set(created_by),
        };

        let deduction_record = deduction.insert(&txn).await?;

        // Add to destination reserve type
        let addition = wallet_reserves::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_id: Set(wallet_id),
            reserve_type: Set(to_type),
            amount_msat: Set(amount_msat), // Positive amount for addition
            reference: Set(reference),
            expires_at: Set(None),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(created_by),
            updated_by: Set(created_by),
        };

        let addition_record = addition.insert(&txn).await?;

        txn.commit().await?;

        Ok((deduction_record, addition_record))
    }

    /// Clean up expired reserves
    pub async fn cleanup_expired(&self) -> RepositoryResult<u64> {
        let expired_reserves = self.find_expired().await?;
        let count = expired_reserves.len() as u64;

        for reserve in expired_reserves {
            self.delete(reserve.id).await?;
        }

        Ok(count)
    }

    /// Count reserves by wallet
    pub async fn count_by_wallet(&self, wallet_id: Uuid) -> RepositoryResult<u64> {
        let count = wallet_reserves::Entity::find()
            .filter(wallet_reserves::Column::WalletId.eq(wallet_id))
            .count(self.db.as_ref())
            .await?;
        Ok(count)
    }

    /// Delete reserve
    pub async fn delete(&self, reserve_id: Uuid) -> RepositoryResult<()> {
        wallet_reserves::Entity::delete_by_id(reserve_id)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    /// Delete reserves by reference (useful for cleanup)
    pub async fn delete_by_reference(&self, reference: String) -> RepositoryResult<u64> {
        let result = wallet_reserves::Entity::delete_many()
            .filter(wallet_reserves::Column::Reference.eq(reference))
            .exec(self.db.as_ref())
            .await?;
        Ok(result.rows_affected)
    }
}

/// Summary of wallet reserves by type
#[derive(Debug, Clone)]
pub struct WalletReserveSummary {
    pub wallet_id: Uuid,
    pub available_msat: i64,
    pub pending_msat: i64,
    pub locked_msat: i64,
    pub emergency_msat: i64,
    pub total_msat: i64,
}

/// Helper function to convert reserve type to string
fn reserve_type_to_string(reserve_type: ReserveType) -> &'static str {
    match reserve_type {
        ReserveType::Available => "available",
        ReserveType::Pending => "pending", 
        ReserveType::Locked => "locked",
        ReserveType::Emergency => "emergency",
    }
}