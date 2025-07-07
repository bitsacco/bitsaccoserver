use super::config::AppConfig;
use crate::middleware::auth::JwtConfig;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
    pub config: AppConfig,
    pub jwt_config: JwtConfig,
}
