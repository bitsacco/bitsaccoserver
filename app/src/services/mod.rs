pub mod analytics;
pub mod auth;
pub mod fedimint;
pub mod share_purchase;
pub mod transaction;
pub mod validation;
pub mod wallet;

use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::repositories::Repositories;

#[derive(Clone)]
pub struct Services {
    pub analytics: analytics::AnalyticsService,
    pub auth: auth::AuthService,
    pub fedimint: fedimint::FedimintClientService,
    pub share_purchase: share_purchase::SharePurchaseService,
    pub transaction: transaction::TransactionService,
    pub validation: validation::ValidationService,
    pub wallet: wallet::WalletService,
}

impl Services {
    pub fn new(
        _db: Arc<DatabaseConnection>,
        repositories: Repositories,
        keycloak_config: auth::KeycloakConfig,
        fedimint_config: fedimint::FedimintConfig,
    ) -> Self {
        let fedimint_service = fedimint::FedimintClientService::new(fedimint_config, repositories.clone());
        
        Self {
            analytics: analytics::AnalyticsService::new(repositories.clone()),
            auth: auth::AuthService::new(repositories.clone(), keycloak_config),
            fedimint: fedimint_service.clone(),
            share_purchase: share_purchase::SharePurchaseService::new(repositories.clone()),
            transaction: transaction::TransactionService::new(repositories.clone()),
            validation: validation::ValidationService::new(repositories.clone()),
            wallet: wallet::WalletService::new(repositories, fedimint_service),
        }
    }
}
