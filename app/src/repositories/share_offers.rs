use super::{RepositoryError, RepositoryResult};
use ::entity::{prelude::*, share_offers};
use sea_orm::*;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ShareOfferRepository {
    db: Arc<DatabaseConnection>,
}

impl ShareOfferRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        offer: share_offers::ActiveModel,
    ) -> RepositoryResult<share_offers::Model> {
        let result = ShareOffers::insert(offer)
            .exec_with_returning(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<share_offers::Model>> {
        let result = ShareOffers::find_by_id(id).one(&*self.db).await?;
        Ok(result)
    }

    pub async fn find_by_id_required(&self, id: Uuid) -> RepositoryResult<share_offers::Model> {
        self.find_by_id(id).await?.ok_or(RepositoryError::NotFound)
    }

    pub async fn find_all(&self) -> RepositoryResult<Vec<share_offers::Model>> {
        let result = ShareOffers::find()
            .order_by_desc(share_offers::Column::CreatedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_status(
        &self,
        status: share_offers::ShareOfferStatus,
    ) -> RepositoryResult<Vec<share_offers::Model>> {
        let result = ShareOffers::find()
            .filter(share_offers::Column::Status.eq(status))
            .order_by_desc(share_offers::Column::CreatedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_active_offers(&self) -> RepositoryResult<Vec<share_offers::Model>> {
        let now = chrono::Utc::now();
        let result = ShareOffers::find()
            .filter(share_offers::Column::Status.eq(share_offers::ShareOfferStatus::Active))
            .filter(
                Condition::any()
                    .add(share_offers::Column::ValidFrom.is_null())
                    .add(share_offers::Column::ValidFrom.lte(now)),
            )
            .filter(
                Condition::any()
                    .add(share_offers::Column::ValidUntil.is_null())
                    .add(share_offers::Column::ValidUntil.gt(now)),
            )
            .filter(share_offers::Column::SharesRemaining.gt(rust_decimal::Decimal::ZERO))
            .order_by_asc(share_offers::Column::PricePerShare)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_available_for_purchase(
        &self,
        quantity: rust_decimal::Decimal,
    ) -> RepositoryResult<Vec<share_offers::Model>> {
        let now = chrono::Utc::now();
        let result = ShareOffers::find()
            .filter(share_offers::Column::Status.eq(share_offers::ShareOfferStatus::Active))
            .filter(
                Condition::any()
                    .add(share_offers::Column::ValidFrom.is_null())
                    .add(share_offers::Column::ValidFrom.lte(now)),
            )
            .filter(
                Condition::any()
                    .add(share_offers::Column::ValidUntil.is_null())
                    .add(share_offers::Column::ValidUntil.gt(now)),
            )
            .filter(share_offers::Column::SharesRemaining.gte(quantity))
            .filter(
                Condition::any()
                    .add(share_offers::Column::MinPurchaseQuantity.is_null())
                    .add(share_offers::Column::MinPurchaseQuantity.lte(quantity)),
            )
            .filter(
                Condition::any()
                    .add(share_offers::Column::MaxPurchaseQuantity.is_null())
                    .add(share_offers::Column::MaxPurchaseQuantity.gte(quantity)),
            )
            .order_by_asc(share_offers::Column::PricePerShare)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_expiring_soon(
        &self,
        hours: i64,
    ) -> RepositoryResult<Vec<share_offers::Model>> {
        let future_time = chrono::Utc::now() + chrono::Duration::hours(hours);
        let result = ShareOffers::find()
            .filter(share_offers::Column::Status.eq(share_offers::ShareOfferStatus::Active))
            .filter(share_offers::Column::ValidUntil.is_not_null())
            .filter(share_offers::Column::ValidUntil.lte(future_time))
            .filter(share_offers::Column::ValidUntil.gt(chrono::Utc::now()))
            .order_by_asc(share_offers::Column::ValidUntil)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn update(
        &self,
        id: Uuid,
        offer: share_offers::ActiveModel,
    ) -> RepositoryResult<share_offers::Model> {
        let existing = self.find_by_id_required(id).await?;

        let mut active_model: share_offers::ActiveModel = existing.into();

        // Only update fields that are set in the input
        if offer.name.is_set() {
            active_model.name = offer.name;
        }
        if offer.description.is_set() {
            active_model.description = offer.description;
        }
        if offer.price_per_share.is_set() {
            active_model.price_per_share = offer.price_per_share;
        }
        if offer.total_shares_available.is_set() {
            active_model.total_shares_available = offer.total_shares_available;
        }
        if offer.status.is_set() {
            active_model.status = offer.status;
        }
        if offer.valid_from.is_set() {
            active_model.valid_from = offer.valid_from;
        }
        if offer.valid_until.is_set() {
            active_model.valid_until = offer.valid_until;
        }
        if offer.min_purchase_quantity.is_set() {
            active_model.min_purchase_quantity = offer.min_purchase_quantity;
        }
        if offer.max_purchase_quantity.is_set() {
            active_model.max_purchase_quantity = offer.max_purchase_quantity;
        }
        if offer.settings.is_set() {
            active_model.settings = offer.settings;
        }
        if offer.metadata.is_set() {
            active_model.metadata = offer.metadata;
        }
        if offer.updated_by.is_set() {
            active_model.updated_by = offer.updated_by;
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn update_shares_sold(
        &self,
        id: Uuid,
        quantity_purchased: rust_decimal::Decimal,
        updated_by: Option<Uuid>,
    ) -> RepositoryResult<share_offers::Model> {
        let existing = self.find_by_id_required(id).await?;

        let new_shares_sold = existing.shares_sold + quantity_purchased;
        let total_shares_available = existing.total_shares_available;

        // Validate that we don't oversell
        if new_shares_sold > total_shares_available {
            return Err(RepositoryError::ValidationError(
                "Cannot sell more shares than available in offer".to_string(),
            ));
        }

        let mut active_model: share_offers::ActiveModel = existing.into();
        active_model.shares_sold = Set(new_shares_sold);
        // Note: shares_remaining will be automatically calculated by database trigger

        // Auto-complete the offer if all shares are sold
        if new_shares_sold >= total_shares_available {
            active_model.status = Set(share_offers::ShareOfferStatus::Completed);
        }

        if let Some(updated_by) = updated_by {
            active_model.updated_by = Set(Some(updated_by));
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn activate_offer(
        &self,
        id: Uuid,
        updated_by: Option<Uuid>,
    ) -> RepositoryResult<share_offers::Model> {
        let existing = self.find_by_id_required(id).await?;

        // Validate that offer can be activated
        if existing.status != share_offers::ShareOfferStatus::Draft {
            return Err(RepositoryError::ValidationError(
                "Only draft offers can be activated".to_string(),
            ));
        }

        let mut active_model: share_offers::ActiveModel = existing.into();
        active_model.status = Set(share_offers::ShareOfferStatus::Active);
        if let Some(updated_by) = updated_by {
            active_model.updated_by = Set(Some(updated_by));
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn pause_offer(
        &self,
        id: Uuid,
        updated_by: Option<Uuid>,
    ) -> RepositoryResult<share_offers::Model> {
        let existing = self.find_by_id_required(id).await?;

        // Validate that offer can be paused
        if existing.status != share_offers::ShareOfferStatus::Active {
            return Err(RepositoryError::ValidationError(
                "Only active offers can be paused".to_string(),
            ));
        }

        let mut active_model: share_offers::ActiveModel = existing.into();
        active_model.status = Set(share_offers::ShareOfferStatus::Paused);
        if let Some(updated_by) = updated_by {
            active_model.updated_by = Set(Some(updated_by));
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn cancel_offer(
        &self,
        id: Uuid,
        updated_by: Option<Uuid>,
    ) -> RepositoryResult<share_offers::Model> {
        let existing = self.find_by_id_required(id).await?;

        // Validate that offer can be cancelled
        if matches!(
            existing.status,
            share_offers::ShareOfferStatus::Completed | share_offers::ShareOfferStatus::Cancelled
        ) {
            return Err(RepositoryError::ValidationError(
                "Cannot cancel completed or already cancelled offers".to_string(),
            ));
        }

        let mut active_model: share_offers::ActiveModel = existing.into();
        active_model.status = Set(share_offers::ShareOfferStatus::Cancelled);
        if let Some(updated_by) = updated_by {
            active_model.updated_by = Set(Some(updated_by));
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> RepositoryResult<()> {
        let existing = self.find_by_id_required(id).await?;

        // Validate that offer can be deleted (only draft offers)
        if existing.status != share_offers::ShareOfferStatus::Draft {
            return Err(RepositoryError::ValidationError(
                "Only draft offers can be deleted".to_string(),
            ));
        }

        let result = ShareOffers::delete_by_id(id).exec(&*self.db).await?;

        if result.rows_affected == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    pub async fn count(&self) -> RepositoryResult<u64> {
        let result = ShareOffers::find().count(&*self.db).await?;
        Ok(result)
    }

    pub async fn count_by_status(
        &self,
        status: share_offers::ShareOfferStatus,
    ) -> RepositoryResult<u64> {
        let result = ShareOffers::find()
            .filter(share_offers::Column::Status.eq(status))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn total_shares_offered(&self) -> RepositoryResult<rust_decimal::Decimal> {
        let result: Option<rust_decimal::Decimal> = ShareOffers::find()
            .select_only()
            .column_as(share_offers::Column::TotalSharesAvailable.sum(), "total")
            .into_tuple::<rust_decimal::Decimal>()
            .one(&*self.db)
            .await?;

        Ok(result.unwrap_or(rust_decimal::Decimal::ZERO))
    }

    pub async fn total_shares_sold(&self) -> RepositoryResult<rust_decimal::Decimal> {
        let result: Option<rust_decimal::Decimal> = ShareOffers::find()
            .select_only()
            .column_as(share_offers::Column::SharesSold.sum(), "total")
            .into_tuple::<rust_decimal::Decimal>()
            .one(&*self.db)
            .await?;

        Ok(result.unwrap_or(rust_decimal::Decimal::ZERO))
    }

    pub async fn with_shares(
        &self,
        id: Uuid,
    ) -> RepositoryResult<Option<(share_offers::Model, Vec<::entity::shares::Model>)>> {
        let offer = ShareOffers::find_by_id(id)
            .find_with_related(Shares)
            .all(&*self.db)
            .await?;

        match offer.into_iter().next() {
            Some((offer, shares)) => Ok(Some((offer, shares))),
            None => Ok(None),
        }
    }

    pub async fn check_purchase_validity(
        &self,
        offer_id: Uuid,
        quantity: rust_decimal::Decimal,
    ) -> RepositoryResult<bool> {
        let offer = self.find_by_id_required(offer_id).await?;
        let now = chrono::Utc::now();

        // Check if offer is active
        if offer.status != share_offers::ShareOfferStatus::Active {
            return Ok(false);
        }

        // Check validity dates
        if let Some(valid_from) = offer.valid_from {
            if now < valid_from {
                return Ok(false);
            }
        }

        if let Some(valid_until) = offer.valid_until {
            if now > valid_until {
                return Ok(false);
            }
        }

        // Check quantity constraints
        if quantity > offer.shares_remaining {
            return Ok(false);
        }

        if let Some(min_qty) = offer.min_purchase_quantity {
            if quantity < min_qty {
                return Ok(false);
            }
        }

        if let Some(max_qty) = offer.max_purchase_quantity {
            if quantity > max_qty {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
