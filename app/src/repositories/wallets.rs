use sea_orm::*;
use std::sync::Arc;
use uuid::Uuid;

use super::{RepositoryError, RepositoryResult};
use ::entity::{sea_orm_active_enums::WalletStatus, wallets};

#[derive(Clone)]
pub struct WalletRepository {
    db: Arc<DatabaseConnection>,
}

impl WalletRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Create a new wallet
    pub async fn create(&self, wallet: wallets::ActiveModel) -> RepositoryResult<wallets::Model> {
        let result = wallet.insert(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Find wallet by ID
    pub async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<wallets::Model>> {
        let wallet = wallets::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await?;
        Ok(wallet)
    }

    /// Find wallets by owner
    pub async fn find_by_owner(
        &self,
        owner_id: Uuid,
        owner_type: String,
    ) -> RepositoryResult<Vec<wallets::Model>> {
        let wallets = wallets::Entity::find()
            .filter(wallets::Column::OwnerId.eq(owner_id))
            .filter(wallets::Column::OwnerType.eq(owner_type))
            .all(self.db.as_ref())
            .await?;
        Ok(wallets)
    }

    /// Find wallet by owner and name (for uniqueness)
    pub async fn find_by_owner_and_name(
        &self,
        owner_id: Uuid,
        owner_type: String,
        name: String,
    ) -> RepositoryResult<Option<wallets::Model>> {
        let wallet = wallets::Entity::find()
            .filter(wallets::Column::OwnerId.eq(owner_id))
            .filter(wallets::Column::OwnerType.eq(owner_type))
            .filter(wallets::Column::Name.eq(name))
            .one(self.db.as_ref())
            .await?;
        Ok(wallet)
    }

    /// Find wallets by federation ID
    pub async fn find_by_federation(
        &self,
        federation_id: String,
    ) -> RepositoryResult<Vec<wallets::Model>> {
        let wallets = wallets::Entity::find()
            .filter(wallets::Column::FederationId.eq(federation_id))
            .all(self.db.as_ref())
            .await?;
        Ok(wallets)
    }

    /// Find wallets by owner and federation
    pub async fn find_by_owner_and_federation(
        &self,
        owner_id: Uuid,
        federation_id: &str,
    ) -> RepositoryResult<Vec<wallets::Model>> {
        let wallets = wallets::Entity::find()
            .filter(wallets::Column::OwnerId.eq(owner_id))
            .filter(wallets::Column::FederationId.eq(federation_id))
            .all(self.db.as_ref())
            .await?;
        Ok(wallets)
    }

    /// Find wallets by status
    pub async fn find_by_status(
        &self,
        status: WalletStatus,
    ) -> RepositoryResult<Vec<wallets::Model>> {
        let wallets = wallets::Entity::find()
            .filter(wallets::Column::Status.eq(status))
            .all(self.db.as_ref())
            .await?;
        Ok(wallets)
    }

    /// Update wallet
    pub async fn update(&self, wallet: wallets::ActiveModel) -> RepositoryResult<wallets::Model> {
        let result = wallet.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Update wallet balance
    pub async fn update_balance(
        &self,
        wallet_id: Uuid,
        balance_msat: i64,
        pending_in_msat: i64,
        pending_out_msat: i64,
    ) -> RepositoryResult<wallets::Model> {
        let wallet = self
            .find_by_id(wallet_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut wallet: wallets::ActiveModel = wallet.into();
        wallet.balance_msat = Set(balance_msat);
        wallet.pending_in_msat = Set(pending_in_msat);
        wallet.pending_out_msat = Set(pending_out_msat);
        wallet.updated_at = Set(chrono::Utc::now().into());

        let result = wallet.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Update wallet sync timestamp
    pub async fn update_sync_timestamp(&self, wallet_id: Uuid) -> RepositoryResult<wallets::Model> {
        let wallet = self
            .find_by_id(wallet_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut wallet: wallets::ActiveModel = wallet.into();
        wallet.last_sync_at = Set(Some(chrono::Utc::now().into()));
        wallet.updated_at = Set(chrono::Utc::now().into());

        let result = wallet.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Alias for update_sync_timestamp
    pub async fn update_last_sync(&self, wallet_id: Uuid) -> RepositoryResult<wallets::Model> {
        self.update_sync_timestamp(wallet_id).await
    }

    /// Update wallet status
    pub async fn update_status(
        &self,
        wallet_id: Uuid,
        status: WalletStatus,
    ) -> RepositoryResult<wallets::Model> {
        let wallet = self
            .find_by_id(wallet_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut wallet: wallets::ActiveModel = wallet.into();
        wallet.status = Set(status);
        wallet.updated_at = Set(chrono::Utc::now().into());

        let result = wallet.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Delete wallet (soft delete by setting status to closed)
    pub async fn delete(&self, wallet_id: Uuid) -> RepositoryResult<()> {
        self.update_status(wallet_id, WalletStatus::Closed).await?;
        Ok(())
    }

    /// Get wallets needing sync (last_sync_at is old or null)
    pub async fn find_stale_wallets(
        &self,
        max_age_hours: i64,
    ) -> RepositoryResult<Vec<wallets::Model>> {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours);

        let wallets = wallets::Entity::find()
            .filter(wallets::Column::Status.eq(WalletStatus::Active))
            .filter(
                Condition::any()
                    .add(wallets::Column::LastSyncAt.is_null())
                    .add(wallets::Column::LastSyncAt.lt(cutoff)),
            )
            .all(self.db.as_ref())
            .await?;
        Ok(wallets)
    }

    /// Get total balance across all wallets for an owner
    pub async fn get_total_balance_for_owner(
        &self,
        owner_id: Uuid,
        owner_type: String,
    ) -> RepositoryResult<i64> {
        let result: Option<i64> = wallets::Entity::find()
            .filter(wallets::Column::OwnerId.eq(owner_id))
            .filter(wallets::Column::OwnerType.eq(owner_type))
            .filter(wallets::Column::Status.eq(WalletStatus::Active))
            .select_only()
            .column_as(wallets::Column::BalanceMsat.sum(), "total_balance")
            .into_tuple()
            .one(self.db.as_ref())
            .await?;

        Ok(result.unwrap_or(0))
    }

    /// Count wallets by owner
    pub async fn count_by_owner(
        &self,
        owner_id: Uuid,
        owner_type: String,
    ) -> RepositoryResult<u64> {
        let count = wallets::Entity::find()
            .filter(wallets::Column::OwnerId.eq(owner_id))
            .filter(wallets::Column::OwnerType.eq(owner_type))
            .count(self.db.as_ref())
            .await?;
        Ok(count)
    }

    /// Validate wallet ownership
    pub async fn validate_ownership(
        &self,
        wallet_id: Uuid,
        owner_id: Uuid,
        owner_type: String,
    ) -> RepositoryResult<bool> {
        let wallet = wallets::Entity::find()
            .filter(wallets::Column::Id.eq(wallet_id))
            .filter(wallets::Column::OwnerId.eq(owner_id))
            .filter(wallets::Column::OwnerType.eq(owner_type))
            .one(self.db.as_ref())
            .await?;

        Ok(wallet.is_some())
    }
}
