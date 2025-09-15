use super::RepositoryResult;
use ::entity::{audit_logs, prelude::*};
use sea_orm::{prelude::Expr, *};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuditLogRepository {
    db: Arc<DatabaseConnection>,
}

impl AuditLogRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        audit_log: audit_logs::ActiveModel,
    ) -> RepositoryResult<audit_logs::Model> {
        let result = AuditLogs::insert(audit_log)
            .exec_with_returning(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<audit_logs::Model>> {
        let result = AuditLogs::find_by_id(id).one(&*self.db).await?;
        Ok(result)
    }

    pub async fn find_by_table_and_record(
        &self,
        table_name: &str,
        record_id: Uuid,
    ) -> RepositoryResult<Vec<audit_logs::Model>> {
        let result = AuditLogs::find()
            .filter(audit_logs::Column::TableName.eq(table_name))
            .filter(audit_logs::Column::RecordId.eq(record_id))
            .order_by_desc(audit_logs::Column::ChangedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_transaction_id(
        &self,
        transaction_id: Uuid,
    ) -> RepositoryResult<Vec<audit_logs::Model>> {
        let result = AuditLogs::find()
            .filter(audit_logs::Column::RecordId.eq(transaction_id))
            .order_by_desc(audit_logs::Column::ChangedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_filtered(
        &self,
        _owner_id: Option<Uuid>,
        action: Option<&str>,
        date_from: Option<chrono::DateTime<chrono::Utc>>,
        date_to: Option<chrono::DateTime<chrono::Utc>>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> RepositoryResult<Vec<audit_logs::Model>> {
        let mut query = AuditLogs::find();

        if let Some(action_filter) = action {
            query = query.filter(audit_logs::Column::Operation.eq(action_filter));
        }

        if let Some(from) = date_from {
            query = query.filter(audit_logs::Column::ChangedAt.gte(from));
        }

        if let Some(to) = date_to {
            query = query.filter(audit_logs::Column::ChangedAt.lte(to));
        }

        query = query.order_by_desc(audit_logs::Column::ChangedAt);

        if let Some(offset_val) = offset {
            query = query.offset(offset_val);
        }

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }

        let result = query.all(&*self.db).await?;
        Ok(result)
    }

    pub async fn count_filtered(
        &self,
        _owner_id: Option<Uuid>,
        action: Option<&str>,
        date_from: Option<chrono::DateTime<chrono::Utc>>,
        date_to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> RepositoryResult<u64> {
        let mut query = AuditLogs::find();

        if let Some(action_filter) = action {
            query = query.filter(audit_logs::Column::Operation.eq(action_filter));
        }

        if let Some(from) = date_from {
            query = query.filter(audit_logs::Column::ChangedAt.gte(from));
        }

        if let Some(to) = date_to {
            query = query.filter(audit_logs::Column::ChangedAt.lte(to));
        }

        let result = query.count(&*self.db).await?;
        Ok(result)
    }

    pub async fn count_transfers_for_owner(&self, _owner_id: Uuid) -> RepositoryResult<u64> {
        let result = AuditLogs::find()
            .filter(audit_logs::Column::Operation.eq("transfer"))
            .filter(audit_logs::Column::TableName.eq("shares"))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_table(
        &self,
        table_name: &str,
    ) -> RepositoryResult<Vec<audit_logs::Model>> {
        let result = AuditLogs::find()
            .filter(audit_logs::Column::TableName.eq(table_name))
            .order_by_desc(audit_logs::Column::ChangedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_user(&self, user_id: Uuid) -> RepositoryResult<Vec<audit_logs::Model>> {
        let result = AuditLogs::find()
            .filter(audit_logs::Column::ChangedBy.eq(user_id))
            .order_by_desc(audit_logs::Column::ChangedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_operation(
        &self,
        operation: &str,
    ) -> RepositoryResult<Vec<audit_logs::Model>> {
        let result = AuditLogs::find()
            .filter(audit_logs::Column::Operation.eq(operation))
            .order_by_desc(audit_logs::Column::ChangedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_recent(&self, limit: u64) -> RepositoryResult<Vec<audit_logs::Model>> {
        let result = AuditLogs::find()
            .order_by_desc(audit_logs::Column::ChangedAt)
            .limit(limit)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> RepositoryResult<Vec<audit_logs::Model>> {
        let result = AuditLogs::find()
            .filter(audit_logs::Column::ChangedAt.between(start_date, end_date))
            .order_by_desc(audit_logs::Column::ChangedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_paginated(
        &self,
        page: u64,
        per_page: u64,
    ) -> RepositoryResult<(Vec<audit_logs::Model>, u64)> {
        let paginator = AuditLogs::find()
            .order_by_desc(audit_logs::Column::ChangedAt)
            .paginate(&*self.db, per_page);

        let total_pages = paginator.num_pages().await?;
        let logs = paginator.fetch_page(page.saturating_sub(1)).await?;

        Ok((logs, total_pages))
    }

    pub async fn search(
        &self,
        table_name: Option<&str>,
        operation: Option<&str>,
        user_id: Option<Uuid>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> RepositoryResult<Vec<audit_logs::Model>> {
        let mut query = AuditLogs::find();

        if let Some(table) = table_name {
            query = query.filter(audit_logs::Column::TableName.eq(table));
        }

        if let Some(op) = operation {
            query = query.filter(audit_logs::Column::Operation.eq(op));
        }

        if let Some(user) = user_id {
            query = query.filter(audit_logs::Column::ChangedBy.eq(user));
        }

        if let Some(start) = start_date {
            query = query.filter(audit_logs::Column::ChangedAt.gte(start));
        }

        if let Some(end) = end_date {
            query = query.filter(audit_logs::Column::ChangedAt.lte(end));
        }

        let result = query
            .order_by_desc(audit_logs::Column::ChangedAt)
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn count(&self) -> RepositoryResult<u64> {
        let result = AuditLogs::find().count(&*self.db).await?;
        Ok(result)
    }

    pub async fn count_by_table(&self, table_name: &str) -> RepositoryResult<u64> {
        let result = AuditLogs::find()
            .filter(audit_logs::Column::TableName.eq(table_name))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn count_by_operation(&self, operation: &str) -> RepositoryResult<u64> {
        let result = AuditLogs::find()
            .filter(audit_logs::Column::Operation.eq(operation))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn count_by_user(&self, user_id: Uuid) -> RepositoryResult<u64> {
        let result = AuditLogs::find()
            .filter(audit_logs::Column::ChangedBy.eq(user_id))
            .count(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn delete_old_logs(
        &self,
        older_than: chrono::DateTime<chrono::Utc>,
    ) -> RepositoryResult<u64> {
        let result = AuditLogs::delete_many()
            .filter(audit_logs::Column::ChangedAt.lt(older_than))
            .exec(&*self.db)
            .await?;

        Ok(result.rows_affected)
    }

    // Get statistics about audit log activity
    pub async fn get_activity_stats(&self) -> RepositoryResult<Vec<(String, u64)>> {
        let result: Vec<(String, u64)> = AuditLogs::find()
            .select_only()
            .column(audit_logs::Column::Operation)
            .column_as(audit_logs::Column::Id.count(), "count")
            .group_by(audit_logs::Column::Operation)
            .order_by_desc(Expr::col((audit_logs::Entity, audit_logs::Column::Id)).count())
            .into_tuple()
            .all(&*self.db)
            .await?;
        Ok(result)
    }

    pub async fn get_table_activity_stats(&self) -> RepositoryResult<Vec<(String, u64)>> {
        let result: Vec<(String, u64)> = AuditLogs::find()
            .select_only()
            .column(audit_logs::Column::TableName)
            .column_as(audit_logs::Column::Id.count(), "count")
            .group_by(audit_logs::Column::TableName)
            .order_by_desc(Expr::col((audit_logs::Entity, audit_logs::Column::Id)).count())
            .into_tuple()
            .all(&*self.db)
            .await?;
        Ok(result)
    }
}
