use super::{RepositoryError, RepositoryResult};
use ::entity::{prelude::*, shares};
use sea_orm::*;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ShareRepository {
    db: Arc<DatabaseConnection>,
}

impl ShareRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(&self, share: shares::ActiveModel) -> RepositoryResult<shares::Model> {
        let result = Shares::insert(share).exec_with_returning(&*self.db).await?;
        Ok(result)
    }

    pub async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<shares::Model>> {
        let result = Shares::find_by_id(id).one(&*self.db).await?;
        Ok(result)
    }

    pub async fn find_by_id_required(&self, id: Uuid) -> RepositoryResult<shares::Model> {
        self.find_by_id(id).await?.ok_or(RepositoryError::NotFound)
    }

    pub async fn find_all(&self) -> RepositoryResult<Vec<shares::Model>> {
        let result = Shares::find()
            .order_by_desc(shares::Column::CreatedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_owner(
        &self,
        owner_id: Uuid,
        owner_type: shares::OwnerType,
    ) -> RepositoryResult<Vec<shares::Model>> {
        let result = Shares::find()
            .filter(shares::Column::OwnerId.eq(owner_id))
            .filter(shares::Column::OwnerType.eq(owner_type))
            .order_by_desc(shares::Column::TotalValue)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_member(&self, member_id: Uuid) -> RepositoryResult<Vec<shares::Model>> {
        self.find_by_owner(member_id, shares::OwnerType::Member)
            .await
    }

    pub async fn find_by_group(&self, group_id: Uuid) -> RepositoryResult<Vec<shares::Model>> {
        self.find_by_owner(group_id, shares::OwnerType::Group).await
    }

    pub async fn find_by_offer(&self, offer_id: Uuid) -> RepositoryResult<Vec<shares::Model>> {
        let result = Shares::find()
            .filter(shares::Column::ShareOfferId.eq(offer_id))
            .order_by_desc(shares::Column::CreatedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_owner_with_id(
        &self,
        owner_id: Uuid,
    ) -> RepositoryResult<Option<shares::Model>> {
        let result = Shares::find()
            .filter(shares::Column::OwnerId.eq(owner_id))
            .one(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_top_shareholders(&self, limit: u64) -> RepositoryResult<Vec<shares::Model>> {
        let result = Shares::find()
            .order_by_desc(shares::Column::TotalValue)
            .limit(limit)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_top_shareholders_by_type(
        &self,
        owner_type: shares::OwnerType,
        limit: u64,
    ) -> RepositoryResult<Vec<shares::Model>> {
        let result = Shares::find()
            .filter(shares::Column::OwnerType.eq(owner_type))
            .order_by_desc(shares::Column::TotalValue)
            .limit(limit)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn update(
        &self,
        id: Uuid,
        share: shares::ActiveModel,
    ) -> RepositoryResult<shares::Model> {
        let existing = self.find_by_id_required(id).await?;

        let mut active_model: shares::ActiveModel = existing.into();

        // Only update fields that are set in the input
        if share.owner_id.is_set() {
            active_model.owner_id = share.owner_id;
        }
        if share.owner_type.is_set() {
            active_model.owner_type = share.owner_type;
        }
        if share.share_offer_id.is_set() {
            active_model.share_offer_id = share.share_offer_id;
        }
        if share.share_quantity.is_set() {
            active_model.share_quantity = share.share_quantity;
        }
        if share.share_value.is_set() {
            active_model.share_value = share.share_value;
        }
        if share.total_value.is_set() {
            active_model.total_value = share.total_value;
        }
        if share.last_transaction_at.is_set() {
            active_model.last_transaction_at = share.last_transaction_at;
        }
        if share.updated_by.is_set() {
            active_model.updated_by = share.updated_by;
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn update_share_quantity(
        &self,
        id: Uuid,
        new_share_quantity: rust_decimal::Decimal,
        updated_by: Option<Uuid>,
    ) -> RepositoryResult<shares::Model> {
        let existing = self.find_by_id_required(id).await?;

        let mut active_model: shares::ActiveModel = existing.into();
        active_model.share_quantity = Set(new_share_quantity);
        // Note: total_value will be automatically calculated by the database trigger
        if let Some(updated_by) = updated_by {
            active_model.updated_by = Set(Some(updated_by));
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn update_share_value(
        &self,
        id: Uuid,
        new_share_value: rust_decimal::Decimal,
        updated_by: Option<Uuid>,
    ) -> RepositoryResult<shares::Model> {
        let existing = self.find_by_id_required(id).await?;

        let mut active_model: shares::ActiveModel = existing.into();
        active_model.share_value = Set(new_share_value);
        // Note: total_value will be automatically calculated by the database trigger
        if let Some(updated_by) = updated_by {
            active_model.updated_by = Set(Some(updated_by));
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> RepositoryResult<()> {
        let result = Shares::delete_by_id(id).exec(&*self.db).await?;

        if result.rows_affected == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    pub async fn count(&self) -> RepositoryResult<u64> {
        let result = Shares::find().count(&*self.db).await?;
        Ok(result)
    }

    pub async fn count_by_owner(
        &self,
        owner_id: Uuid,
        owner_type: shares::OwnerType,
    ) -> RepositoryResult<u64> {
        let result = Shares::find()
            .filter(shares::Column::OwnerId.eq(owner_id))
            .filter(shares::Column::OwnerType.eq(owner_type))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn count_by_group(&self, group_id: Uuid) -> RepositoryResult<u64> {
        self.count_by_owner(group_id, shares::OwnerType::Group)
            .await
    }

    pub async fn count_by_member(&self, member_id: Uuid) -> RepositoryResult<u64> {
        self.count_by_owner(member_id, shares::OwnerType::Member)
            .await
    }

    pub async fn count_by_owner_type(
        &self,
        owner_type: shares::OwnerType,
    ) -> RepositoryResult<u64> {
        let result = Shares::find()
            .filter(shares::Column::OwnerType.eq(owner_type))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn total_value_by_owner(
        &self,
        owner_id: Uuid,
        owner_type: shares::OwnerType,
    ) -> RepositoryResult<rust_decimal::Decimal> {
        let result: Option<rust_decimal::Decimal> = Shares::find()
            .filter(shares::Column::OwnerId.eq(owner_id))
            .filter(shares::Column::OwnerType.eq(owner_type))
            .select_only()
            .column_as(shares::Column::TotalValue.sum(), "total")
            .into_tuple::<rust_decimal::Decimal>()
            .one(&*self.db)
            .await?;

        Ok(result.unwrap_or_else(|| rust_decimal::Decimal::ZERO))
    }

    pub async fn total_value_by_group(
        &self,
        group_id: Uuid,
    ) -> RepositoryResult<rust_decimal::Decimal> {
        self.total_value_by_owner(group_id, shares::OwnerType::Group)
            .await
    }

    pub async fn total_value_by_member(
        &self,
        member_id: Uuid,
    ) -> RepositoryResult<rust_decimal::Decimal> {
        self.total_value_by_owner(member_id, shares::OwnerType::Member)
            .await
    }

    pub async fn total_shares_quantity(&self) -> RepositoryResult<rust_decimal::Decimal> {
        let result: Option<rust_decimal::Decimal> = Shares::find()
            .select_only()
            .column_as(shares::Column::ShareQuantity.sum(), "total")
            .into_tuple::<rust_decimal::Decimal>()
            .one(&*self.db)
            .await?;

        Ok(result.unwrap_or_else(|| rust_decimal::Decimal::ZERO))
    }

    pub async fn total_shares_by_owner_type(
        &self,
        owner_type: shares::OwnerType,
    ) -> RepositoryResult<rust_decimal::Decimal> {
        let result: Option<rust_decimal::Decimal> = Shares::find()
            .filter(shares::Column::OwnerType.eq(owner_type))
            .select_only()
            .column_as(shares::Column::ShareQuantity.sum(), "total")
            .into_tuple::<rust_decimal::Decimal>()
            .one(&*self.db)
            .await?;

        Ok(result.unwrap_or_else(|| rust_decimal::Decimal::ZERO))
    }

    pub async fn average_share_value(&self) -> RepositoryResult<rust_decimal::Decimal> {
        // Since all shares have the same value globally, we can just get any share's value
        let share = Shares::find().one(&*self.db).await?;

        match share {
            Some(share) => Ok(share.share_value),
            None => Ok(rust_decimal::Decimal::ZERO),
        }
    }

    pub async fn with_owner_details(
        &self,
        id: Uuid,
    ) -> RepositoryResult<
        Option<(
            shares::Model,
            Option<::entity::groups::Model>,
            Option<::entity::members::Model>,
        )>,
    > {
        let share = Shares::find_by_id(id).one(&*self.db).await?;

        match share {
            Some(share) => {
                let (group, member) = match share.owner_type {
                    shares::OwnerType::Group => {
                        let group = Groups::find_by_id(share.owner_id).one(&*self.db).await?;
                        (group, None)
                    }
                    shares::OwnerType::Member => {
                        let member = Members::find_by_id(share.owner_id).one(&*self.db).await?;
                        (None, member)
                    }
                };
                Ok(Some((share, group, member)))
            }
            None => Ok(None),
        }
    }

    pub async fn check_owner_has_shares(
        &self,
        owner_id: Uuid,
        owner_type: shares::OwnerType,
    ) -> RepositoryResult<bool> {
        let count = Shares::find()
            .filter(shares::Column::OwnerId.eq(owner_id))
            .filter(shares::Column::OwnerType.eq(owner_type))
            .count(&*self.db)
            .await?;
        Ok(count > 0)
    }
}
