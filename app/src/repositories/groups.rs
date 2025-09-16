use super::{RepositoryError, RepositoryResult};
use ::entity::{groups, prelude::*};
use sea_orm::*;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct GroupRepository {
    db: Arc<DatabaseConnection>,
}

impl GroupRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(&self, group: groups::ActiveModel) -> RepositoryResult<groups::Model> {
        let result = Groups::insert(group).exec_with_returning(&*self.db).await?;
        Ok(result)
    }

    pub async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<groups::Model>> {
        let result = Groups::find_by_id(id).one(&*self.db).await?;
        Ok(result)
    }

    pub async fn find_by_id_required(&self, id: Uuid) -> RepositoryResult<groups::Model> {
        self.find_by_id(id).await?.ok_or(RepositoryError::NotFound)
    }

    pub async fn find_all(&self) -> RepositoryResult<Vec<groups::Model>> {
        let result = Groups::find().all(&*self.db).await?;
        Ok(result)
    }

    pub async fn find_by_type(
        &self,
        group_type: groups::GroupType,
    ) -> RepositoryResult<Vec<groups::Model>> {
        let result = Groups::find()
            .filter(groups::Column::GroupType.eq(group_type))
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_parent(
        &self,
        parent_id: Option<Uuid>,
    ) -> RepositoryResult<Vec<groups::Model>> {
        let query = if let Some(parent_id) = parent_id {
            Groups::find().filter(groups::Column::ParentId.eq(parent_id))
        } else {
            Groups::find().filter(groups::Column::ParentId.is_null())
        };

        let result = query.all(&*self.db).await?;
        Ok(result)
    }

    pub async fn find_children(&self, parent_id: Uuid) -> RepositoryResult<Vec<groups::Model>> {
        let result = Groups::find()
            .filter(groups::Column::ParentId.eq(parent_id))
            .order_by_asc(groups::Column::SortOrder)
            .order_by_asc(groups::Column::Name)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_hierarchy_from_root(&self) -> RepositoryResult<Vec<groups::Model>> {
        let result = Groups::find()
            .order_by_asc(groups::Column::Level)
            .order_by_asc(groups::Column::SortOrder)
            .order_by_asc(groups::Column::Name)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn search(&self, query: &str) -> RepositoryResult<Vec<groups::Model>> {
        let search_term = format!("%{}%", query);
        let result = Groups::find()
            .filter(
                Condition::any()
                    .add(groups::Column::Name.like(&search_term))
                    .add(groups::Column::Description.like(&search_term)),
            )
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn update(
        &self,
        id: Uuid,
        group: groups::ActiveModel,
    ) -> RepositoryResult<groups::Model> {
        let existing = self.find_by_id_required(id).await?;

        let mut active_model: groups::ActiveModel = existing.into();

        // Only update fields that are set in the input
        if group.name.is_set() {
            active_model.name = group.name;
        }
        if group.description.is_set() {
            active_model.description = group.description;
        }
        if group.group_type.is_set() {
            active_model.group_type = group.group_type;
        }
        if group.status.is_set() {
            active_model.status = group.status;
        }
        if group.parent_id.is_set() {
            active_model.parent_id = group.parent_id;
        }
        if group.sort_order.is_set() {
            active_model.sort_order = group.sort_order;
        }
        if group.settings.is_set() {
            active_model.settings = group.settings;
        }
        if group.metadata.is_set() {
            active_model.metadata = group.metadata;
        }
        if group.updated_by.is_set() {
            active_model.updated_by = group.updated_by;
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> RepositoryResult<()> {
        let result = Groups::delete_by_id(id).exec(&*self.db).await?;

        if result.rows_affected == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    pub async fn count(&self) -> RepositoryResult<u64> {
        let result = Groups::find().count(&*self.db).await?;
        Ok(result)
    }

    pub async fn count_by_type(&self, group_type: groups::GroupType) -> RepositoryResult<u64> {
        let result = Groups::find()
            .filter(groups::Column::GroupType.eq(group_type))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn count_by_status(&self, status: groups::GroupStatus) -> RepositoryResult<u64> {
        let result = Groups::find()
            .filter(groups::Column::Status.eq(status))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn with_members(
        &self,
        id: Uuid,
    ) -> RepositoryResult<Option<(groups::Model, Vec<::entity::members::Model>)>> {
        // First get the group
        let group = Groups::find_by_id(id).one(&*self.db).await?;

        match group {
            Some(group) => {
                // Get members through group_memberships
                let members = group
                    .find_related(GroupMemberships)
                    .find_with_related(Members)
                    .all(&*self.db)
                    .await?
                    .into_iter()
                    .flat_map(|(_, members)| members)
                    .collect::<Vec<_>>();

                Ok(Some((group, members)))
            }
            None => Ok(None),
        }
    }

    pub async fn with_shares(
        &self,
        id: Uuid,
    ) -> RepositoryResult<Option<(groups::Model, Vec<::entity::shares::Model>)>> {
        let group = Groups::find_by_id(id).one(&*self.db).await?;

        match group {
            Some(group) => {
                // Find shares owned by this group
                let shares = Shares::find()
                    .filter(::entity::shares::Column::OwnerId.eq(id))
                    .filter(
                        ::entity::shares::Column::OwnerType.eq(::entity::shares::OwnerType::Group),
                    )
                    .all(&*self.db)
                    .await?;
                Ok(Some((group, shares)))
            }
            None => Ok(None),
        }
    }
}
