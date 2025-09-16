use super::{RepositoryError, RepositoryResult};
use ::entity::{members, prelude::*};
use sea_orm::*;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct MemberRepository {
    db: Arc<DatabaseConnection>,
}

impl MemberRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(&self, member: members::ActiveModel) -> RepositoryResult<members::Model> {
        let result = Members::insert(member)
            .exec_with_returning(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<members::Model>> {
        let result = Members::find_by_id(id).one(&*self.db).await?;
        Ok(result)
    }

    pub async fn find_by_id_required(&self, id: Uuid) -> RepositoryResult<members::Model> {
        self.find_by_id(id).await?.ok_or(RepositoryError::NotFound)
    }

    pub async fn find_by_member_number(
        &self,
        member_number: &str,
    ) -> RepositoryResult<Option<members::Model>> {
        let result = Members::find()
            .filter(members::Column::MemberNumber.eq(member_number))
            .one(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_email(&self, email: &str) -> RepositoryResult<Option<members::Model>> {
        let result = Members::find()
            .filter(members::Column::Email.eq(email))
            .one(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_keycloak_user_id(
        &self,
        keycloak_user_id: &str,
    ) -> RepositoryResult<Option<members::Model>> {
        let result = Members::find()
            .filter(members::Column::KeycloakUserId.eq(keycloak_user_id))
            .one(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_all(&self) -> RepositoryResult<Vec<members::Model>> {
        let result = Members::find()
            .order_by_asc(members::Column::MemberNumber)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_status(
        &self,
        status: members::MemberStatus,
    ) -> RepositoryResult<Vec<members::Model>> {
        let result = Members::find()
            .filter(members::Column::Status.eq(status))
            .order_by_asc(members::Column::MemberNumber)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn search(&self, query: &str) -> RepositoryResult<Vec<members::Model>> {
        let search_term = format!("%{}%", query);
        let result = Members::find()
            .filter(
                Condition::any()
                    .add(members::Column::Name.like(&search_term))
                    .add(members::Column::Email.like(&search_term))
                    .add(members::Column::MemberNumber.like(&search_term)),
            )
            .order_by_asc(members::Column::MemberNumber)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_paginated(
        &self,
        page: u64,
        per_page: u64,
    ) -> RepositoryResult<(Vec<members::Model>, u64)> {
        let paginator = Members::find()
            .order_by_asc(members::Column::MemberNumber)
            .paginate(&*self.db, per_page);

        let total_pages = paginator.num_pages().await?;
        let members = paginator.fetch_page(page.saturating_sub(1)).await?;

        Ok((members, total_pages))
    }

    pub async fn update(
        &self,
        id: Uuid,
        member: members::ActiveModel,
    ) -> RepositoryResult<members::Model> {
        let existing = self.find_by_id_required(id).await?;

        let mut active_model: members::ActiveModel = existing.into();

        // Only update fields that are set in the input
        if member.name.is_set() {
            active_model.name = member.name;
        }
        if member.email.is_set() {
            active_model.email = member.email;
        }
        if member.phone.is_set() {
            active_model.phone = member.phone;
        }
        if member.status.is_set() {
            active_model.status = member.status;
        }
        if member.keycloak_user_id.is_set() {
            active_model.keycloak_user_id = member.keycloak_user_id;
        }
        if member.profile.is_set() {
            active_model.profile = member.profile;
        }
        if member.updated_by.is_set() {
            active_model.updated_by = member.updated_by;
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> RepositoryResult<()> {
        let result = Members::delete_by_id(id).exec(&*self.db).await?;

        if result.rows_affected == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    pub async fn count(&self) -> RepositoryResult<u64> {
        let result = Members::find().count(&*self.db).await?;
        Ok(result)
    }

    pub async fn count_by_status(&self, status: members::MemberStatus) -> RepositoryResult<u64> {
        let result = Members::find()
            .filter(members::Column::Status.eq(status))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn with_groups(
        &self,
        id: Uuid,
    ) -> RepositoryResult<Option<(members::Model, Vec<::entity::groups::Model>)>> {
        // First get the member
        let member = Members::find_by_id(id).one(&*self.db).await?;

        match member {
            Some(member) => {
                // Get groups through group_memberships
                let groups = member
                    .find_related(GroupMemberships)
                    .find_with_related(Groups)
                    .all(&*self.db)
                    .await?
                    .into_iter()
                    .flat_map(|(_, groups)| groups)
                    .collect::<Vec<_>>();

                Ok(Some((member, groups)))
            }
            None => Ok(None),
        }
    }

    pub async fn with_shares(
        &self,
        id: Uuid,
    ) -> RepositoryResult<Option<(members::Model, Vec<::entity::shares::Model>)>> {
        let member = Members::find_by_id(id).one(&*self.db).await?;

        match member {
            Some(member) => {
                // Find shares owned by this member
                let shares = Shares::find()
                    .filter(::entity::shares::Column::OwnerId.eq(id))
                    .filter(
                        ::entity::shares::Column::OwnerType.eq(::entity::shares::OwnerType::Member),
                    )
                    .all(&*self.db)
                    .await?;
                Ok(Some((member, shares)))
            }
            None => Ok(None),
        }
    }

    pub async fn check_member_number_unique(
        &self,
        member_number: &str,
        exclude_id: Option<Uuid>,
    ) -> RepositoryResult<bool> {
        let mut query = Members::find().filter(members::Column::MemberNumber.eq(member_number));

        if let Some(id) = exclude_id {
            query = query.filter(members::Column::Id.ne(id));
        }

        let count = query.count(&*self.db).await?;
        Ok(count == 0)
    }

    pub async fn check_email_unique(
        &self,
        email: &str,
        exclude_id: Option<Uuid>,
    ) -> RepositoryResult<bool> {
        let mut query = Members::find().filter(members::Column::Email.eq(email));

        if let Some(id) = exclude_id {
            query = query.filter(members::Column::Id.ne(id));
        }

        let count = query.count(&*self.db).await?;
        Ok(count == 0)
    }
}
