use super::{RepositoryError, RepositoryResult};
use ::entity::{group_memberships, prelude::*};
use sea_orm::*;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct GroupMembershipRepository {
    db: Arc<DatabaseConnection>,
}

impl GroupMembershipRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        membership: group_memberships::ActiveModel,
    ) -> RepositoryResult<group_memberships::Model> {
        let result = GroupMemberships::insert(membership)
            .exec_with_returning(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<group_memberships::Model>> {
        let result = GroupMemberships::find_by_id(id).one(&*self.db).await?;
        Ok(result)
    }

    pub async fn find_by_id_required(
        &self,
        id: Uuid,
    ) -> RepositoryResult<group_memberships::Model> {
        self.find_by_id(id).await?.ok_or(RepositoryError::NotFound)
    }

    pub async fn find_by_group(
        &self,
        group_id: Uuid,
    ) -> RepositoryResult<Vec<group_memberships::Model>> {
        let result = GroupMemberships::find()
            .filter(group_memberships::Column::GroupId.eq(group_id))
            .order_by_desc(group_memberships::Column::JoinedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_active_by_group(
        &self,
        group_id: Uuid,
    ) -> RepositoryResult<Vec<group_memberships::Model>> {
        let result = GroupMemberships::find()
            .filter(group_memberships::Column::GroupId.eq(group_id))
            .filter(group_memberships::Column::IsActive.eq(true))
            .order_by_desc(group_memberships::Column::JoinedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_member(
        &self,
        member_id: Uuid,
    ) -> RepositoryResult<Vec<group_memberships::Model>> {
        let result = GroupMemberships::find()
            .filter(group_memberships::Column::MemberId.eq(member_id))
            .order_by_desc(group_memberships::Column::JoinedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_active_by_member(
        &self,
        member_id: Uuid,
    ) -> RepositoryResult<Vec<group_memberships::Model>> {
        let result = GroupMemberships::find()
            .filter(group_memberships::Column::MemberId.eq(member_id))
            .filter(group_memberships::Column::IsActive.eq(true))
            .order_by_desc(group_memberships::Column::JoinedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_group_and_member(
        &self,
        group_id: Uuid,
        member_id: Uuid,
    ) -> RepositoryResult<Option<group_memberships::Model>> {
        let result = GroupMemberships::find()
            .filter(group_memberships::Column::GroupId.eq(group_id))
            .filter(group_memberships::Column::MemberId.eq(member_id))
            .filter(group_memberships::Column::IsActive.eq(true))
            .one(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_role(
        &self,
        role: group_memberships::MembershipRole,
    ) -> RepositoryResult<Vec<group_memberships::Model>> {
        let result = GroupMemberships::find()
            .filter(group_memberships::Column::Role.eq(role))
            .filter(group_memberships::Column::IsActive.eq(true))
            .order_by_desc(group_memberships::Column::JoinedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_admins_by_group(
        &self,
        group_id: Uuid,
    ) -> RepositoryResult<Vec<group_memberships::Model>> {
        let result = GroupMemberships::find()
            .filter(group_memberships::Column::GroupId.eq(group_id))
            .filter(group_memberships::Column::Role.eq(group_memberships::MembershipRole::Admin))
            .filter(group_memberships::Column::IsActive.eq(true))
            .order_by_desc(group_memberships::Column::JoinedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn update(
        &self,
        id: Uuid,
        membership: group_memberships::ActiveModel,
    ) -> RepositoryResult<group_memberships::Model> {
        let existing = self.find_by_id_required(id).await?;

        let mut active_model: group_memberships::ActiveModel = existing.into();

        // Only update fields that are set in the input
        if membership.role.is_set() {
            active_model.role = membership.role;
        }
        if membership.left_at.is_set() {
            active_model.left_at = membership.left_at;
        }
        if membership.is_active.is_set() {
            active_model.is_active = membership.is_active;
        }
        if membership.permissions.is_set() {
            active_model.permissions = membership.permissions;
        }
        if membership.updated_by.is_set() {
            active_model.updated_by = membership.updated_by;
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn deactivate(
        &self,
        id: Uuid,
        updated_by: Option<Uuid>,
    ) -> RepositoryResult<group_memberships::Model> {
        let existing = self.find_by_id_required(id).await?;

        let mut active_model: group_memberships::ActiveModel = existing.into();
        active_model.is_active = Set(false);
        active_model.left_at = Set(Some(chrono::Utc::now().into()));
        if let Some(updated_by) = updated_by {
            active_model.updated_by = Set(Some(updated_by));
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn activate(
        &self,
        id: Uuid,
        updated_by: Option<Uuid>,
    ) -> RepositoryResult<group_memberships::Model> {
        let existing = self.find_by_id_required(id).await?;

        let mut active_model: group_memberships::ActiveModel = existing.into();
        active_model.is_active = Set(true);
        active_model.left_at = Set(None);
        if let Some(updated_by) = updated_by {
            active_model.updated_by = Set(Some(updated_by));
        }

        let result = active_model.update(&*self.db).await?;
        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> RepositoryResult<()> {
        let result = GroupMemberships::delete_by_id(id).exec(&*self.db).await?;

        if result.rows_affected == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    pub async fn count_by_group(&self, group_id: Uuid) -> RepositoryResult<u64> {
        let result = GroupMemberships::find()
            .filter(group_memberships::Column::GroupId.eq(group_id))
            .filter(group_memberships::Column::IsActive.eq(true))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn count_by_member(&self, member_id: Uuid) -> RepositoryResult<u64> {
        let result = GroupMemberships::find()
            .filter(group_memberships::Column::MemberId.eq(member_id))
            .filter(group_memberships::Column::IsActive.eq(true))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn count_by_role(
        &self,
        role: group_memberships::MembershipRole,
    ) -> RepositoryResult<u64> {
        let result = GroupMemberships::find()
            .filter(group_memberships::Column::Role.eq(role))
            .filter(group_memberships::Column::IsActive.eq(true))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn check_membership_exists(
        &self,
        group_id: Uuid,
        member_id: Uuid,
    ) -> RepositoryResult<bool> {
        let count = GroupMemberships::find()
            .filter(group_memberships::Column::GroupId.eq(group_id))
            .filter(group_memberships::Column::MemberId.eq(member_id))
            .filter(group_memberships::Column::IsActive.eq(true))
            .count(&*self.db)
            .await?;
        Ok(count > 0)
    }

    pub async fn check_is_admin(&self, group_id: Uuid, member_id: Uuid) -> RepositoryResult<bool> {
        let count = GroupMemberships::find()
            .filter(group_memberships::Column::GroupId.eq(group_id))
            .filter(group_memberships::Column::MemberId.eq(member_id))
            .filter(group_memberships::Column::Role.eq(group_memberships::MembershipRole::Admin))
            .filter(group_memberships::Column::IsActive.eq(true))
            .count(&*self.db)
            .await?;
        Ok(count > 0)
    }

    pub async fn with_group_and_member(
        &self,
        id: Uuid,
    ) -> RepositoryResult<
        Option<(
            group_memberships::Model,
            Option<::entity::groups::Model>,
            Option<::entity::members::Model>,
        )>,
    > {
        let membership = GroupMemberships::find_by_id(id).one(&*self.db).await?;

        match membership {
            Some(membership) => {
                let group = Groups::find_by_id(membership.group_id)
                    .one(&*self.db)
                    .await?;
                let member = Members::find_by_id(membership.member_id)
                    .one(&*self.db)
                    .await?;
                Ok(Some((membership, group, member)))
            }
            None => Ok(None),
        }
    }

    pub async fn find_groups_by_member(
        &self,
        member_id: Uuid,
    ) -> RepositoryResult<Vec<::entity::groups::Model>> {
        let groups = GroupMemberships::find()
            .filter(group_memberships::Column::MemberId.eq(member_id))
            .filter(group_memberships::Column::IsActive.eq(true))
            .find_also_related(Groups)
            .all(&*self.db)
            .await?;

        let result = groups.into_iter().filter_map(|(_, group)| group).collect();

        Ok(result)
    }

    pub async fn find_members_by_group_paginated(
        &self,
        group_id: Uuid,
        page: u32,
        limit: u32,
    ) -> RepositoryResult<Vec<::entity::members::Model>> {
        let offset = (page - 1) * limit;

        let members = GroupMemberships::find()
            .filter(group_memberships::Column::GroupId.eq(group_id))
            .filter(group_memberships::Column::IsActive.eq(true))
            .find_also_related(Members)
            .offset(offset as u64)
            .limit(limit as u64)
            .all(&*self.db)
            .await?;

        let result = members
            .into_iter()
            .filter_map(|(_, member)| member)
            .collect();

        Ok(result)
    }
}
