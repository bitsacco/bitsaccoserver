pub use sea_orm_migration::prelude::*;

mod m20250703_000001_initial_schema;
mod m20250703_000002_indexes_and_views;
mod m20250703_000003_cache_triggers;
mod m20250703_000004_advanced_triggers;
mod m20250803_000001_wallet_tables;
mod m20250803_000002_wallet_indexes;
mod m20250803_000003_lnurl_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250703_000001_initial_schema::Migration),
            Box::new(m20250703_000002_indexes_and_views::Migration),
            Box::new(m20250703_000003_cache_triggers::Migration),
            Box::new(m20250703_000004_advanced_triggers::Migration),
            Box::new(m20250803_000001_wallet_tables::Migration),
            Box::new(m20250803_000002_wallet_indexes::Migration),
            Box::new(m20250803_000003_lnurl_tables::Migration),
        ]
    }
}
