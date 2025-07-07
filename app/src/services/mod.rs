pub mod analytics;
pub mod auth;
pub mod share_purchase;
pub mod transaction;
pub mod validation;

use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::repositories::Repositories;

#[derive(Clone)]
pub struct Services {
    pub analytics: analytics::AnalyticsService,
    pub auth: auth::AuthService,
    pub share_purchase: share_purchase::SharePurchaseService,
    pub transaction: transaction::TransactionService,
    pub validation: validation::ValidationService,
}

impl Services {
    pub fn new(
        _db: Arc<DatabaseConnection>,
        repositories: Repositories,
        keycloak_config: auth::KeycloakConfig,
    ) -> Self {
        Self {
            analytics: analytics::AnalyticsService::new(repositories.clone()),
            auth: auth::AuthService::new(repositories.clone(), keycloak_config),
            share_purchase: share_purchase::SharePurchaseService::new(repositories.clone()),
            transaction: transaction::TransactionService::new(repositories.clone()),
            validation: validation::ValidationService::new(repositories),
        }
    }
}
