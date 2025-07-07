pub use sea_orm_migration::prelude::*;

mod m20250703_000001_initial_schema;
mod m20250703_000002_indexes_and_views;
mod m20250703_000003_cache_triggers;
mod m20250703_000004_advanced_triggers;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250703_000001_initial_schema::Migration),
            Box::new(m20250703_000002_indexes_and_views::Migration),
            Box::new(m20250703_000003_cache_triggers::Migration),
            Box::new(m20250703_000004_advanced_triggers::Migration),
        ]
    }
}
