use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create initial tables - will be implemented in Phase 2
        manager
            .create_table(
                Table::create()
                    .table(TempTable::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TempTable::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TempTable::Name).string().not_null())
                    .col(
                        ColumnDef::new(TempTable::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TempTable::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TempTable {
    Table,
    Id,
    Name,
    CreatedAt,
}
