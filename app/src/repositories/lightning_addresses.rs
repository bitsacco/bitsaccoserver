use sea_orm::*;
use std::sync::Arc;
use uuid::Uuid;

use super::{RepositoryError, RepositoryResult};
use ::entity::lightning_addresses;

#[derive(Clone)]
pub struct LightningAddressRepository {
    db: Arc<DatabaseConnection>,
}

impl LightningAddressRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Create a new lightning address
    pub async fn create(
        &self,
        address: lightning_addresses::ActiveModel,
    ) -> RepositoryResult<lightning_addresses::Model> {
        let result = address.insert(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Find lightning address by ID
    pub async fn find_by_id(
        &self,
        id: Uuid,
    ) -> RepositoryResult<Option<lightning_addresses::Model>> {
        let address = lightning_addresses::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await?;
        Ok(address)
    }

    /// Find lightning address by username (case-insensitive)
    pub async fn find_by_username(
        &self,
        username: &str,
    ) -> RepositoryResult<Option<lightning_addresses::Model>> {
        let address = lightning_addresses::Entity::find()
            .filter(lightning_addresses::Column::Username.eq(username.to_lowercase()))
            .one(self.db.as_ref())
            .await?;
        Ok(address)
    }

    /// Find lightning address by username and domain
    pub async fn find_by_username_and_domain(
        &self,
        username: &str,
        domain: &str,
    ) -> RepositoryResult<Option<lightning_addresses::Model>> {
        let address = lightning_addresses::Entity::find()
            .filter(lightning_addresses::Column::Username.eq(username.to_lowercase()))
            .filter(lightning_addresses::Column::Domain.eq(domain))
            .one(self.db.as_ref())
            .await?;
        Ok(address)
    }

    /// Find all lightning addresses for a wallet
    pub async fn find_by_wallet_id(
        &self,
        wallet_id: Uuid,
    ) -> RepositoryResult<Vec<lightning_addresses::Model>> {
        let addresses = lightning_addresses::Entity::find()
            .filter(lightning_addresses::Column::WalletId.eq(wallet_id))
            .all(self.db.as_ref())
            .await?;
        Ok(addresses)
    }

    /// Find active lightning addresses for a wallet
    pub async fn find_active_by_wallet_id(
        &self,
        wallet_id: Uuid,
    ) -> RepositoryResult<Vec<lightning_addresses::Model>> {
        let addresses = lightning_addresses::Entity::find()
            .filter(lightning_addresses::Column::WalletId.eq(wallet_id))
            .filter(lightning_addresses::Column::IsActive.eq(true))
            .all(self.db.as_ref())
            .await?;
        Ok(addresses)
    }

    /// Check if username is available (case-insensitive)
    pub async fn is_username_available(
        &self,
        username: &str,
        domain: &str,
    ) -> RepositoryResult<bool> {
        let count = lightning_addresses::Entity::find()
            .filter(lightning_addresses::Column::Username.eq(username.to_lowercase()))
            .filter(lightning_addresses::Column::Domain.eq(domain))
            .count(self.db.as_ref())
            .await?;
        Ok(count == 0)
    }

    /// Update lightning address
    pub async fn update(
        &self,
        address: lightning_addresses::ActiveModel,
    ) -> RepositoryResult<lightning_addresses::Model> {
        let result = address.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Update last used timestamp
    pub async fn update_last_used(
        &self,
        address_id: Uuid,
    ) -> RepositoryResult<lightning_addresses::Model> {
        let address = self
            .find_by_id(address_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut address: lightning_addresses::ActiveModel = address.into();
        address.last_used_at = Set(Some(chrono::Utc::now().into()));
        address.updated_at = Set(chrono::Utc::now().into());

        let result = address.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Activate or deactivate lightning address
    pub async fn set_active_status(
        &self,
        address_id: Uuid,
        is_active: bool,
    ) -> RepositoryResult<lightning_addresses::Model> {
        let address = self
            .find_by_id(address_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut address: lightning_addresses::ActiveModel = address.into();
        address.is_active = Set(is_active);
        address.updated_at = Set(chrono::Utc::now().into());

        let result = address.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Update amount limits
    pub async fn update_limits(
        &self,
        address_id: Uuid,
        min_sendable_msat: i64,
        max_sendable_msat: i64,
    ) -> RepositoryResult<lightning_addresses::Model> {
        let address = self
            .find_by_id(address_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut address: lightning_addresses::ActiveModel = address.into();
        address.min_sendable_msat = Set(min_sendable_msat);
        address.max_sendable_msat = Set(max_sendable_msat);
        address.updated_at = Set(chrono::Utc::now().into());

        let result = address.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Soft delete lightning address (deactivate)
    pub async fn delete(&self, address_id: Uuid) -> RepositoryResult<()> {
        self.set_active_status(address_id, false).await?;
        Ok(())
    }

    /// Get lightning addresses by domain
    pub async fn find_by_domain(
        &self,
        domain: &str,
    ) -> RepositoryResult<Vec<lightning_addresses::Model>> {
        let addresses = lightning_addresses::Entity::find()
            .filter(lightning_addresses::Column::Domain.eq(domain))
            .filter(lightning_addresses::Column::IsActive.eq(true))
            .all(self.db.as_ref())
            .await?;
        Ok(addresses)
    }

    /// Count lightning addresses for a wallet
    pub async fn count_by_wallet(&self, wallet_id: Uuid) -> RepositoryResult<u64> {
        let count = lightning_addresses::Entity::find()
            .filter(lightning_addresses::Column::WalletId.eq(wallet_id))
            .filter(lightning_addresses::Column::IsActive.eq(true))
            .count(self.db.as_ref())
            .await?;
        Ok(count)
    }

    /// Find addresses that haven't been used recently (for cleanup/analytics)
    pub async fn find_unused_addresses(
        &self,
        days_unused: i64,
    ) -> RepositoryResult<Vec<lightning_addresses::Model>> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days_unused);

        let addresses = lightning_addresses::Entity::find()
            .filter(lightning_addresses::Column::IsActive.eq(true))
            .filter(
                Condition::any()
                    .add(lightning_addresses::Column::LastUsedAt.is_null())
                    .add(lightning_addresses::Column::LastUsedAt.lt(cutoff)),
            )
            .all(self.db.as_ref())
            .await?;
        Ok(addresses)
    }

    /// Validate address ownership by wallet
    pub async fn validate_ownership(
        &self,
        address_id: Uuid,
        wallet_id: Uuid,
    ) -> RepositoryResult<bool> {
        let address = lightning_addresses::Entity::find()
            .filter(lightning_addresses::Column::Id.eq(address_id))
            .filter(lightning_addresses::Column::WalletId.eq(wallet_id))
            .one(self.db.as_ref())
            .await?;

        Ok(address.is_some())
    }
}
