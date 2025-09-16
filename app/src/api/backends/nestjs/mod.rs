pub mod auth;
pub mod client;
pub mod endpoints;
pub mod groups;
pub mod jwt_validator;
pub mod middleware;
pub mod users;
pub mod wallets;

use crate::api::{config::ApiConfig, errors::ApiResult};

pub use auth::NestJsAuthApi;
pub use client::NestJsClient;
pub use groups::NestJsGroupsApi;
pub use jwt_validator::JwtValidator;
pub use middleware::MiddlewareChain;
pub use users::NestJsUsersApi;
pub use wallets::NestJsWalletsApi;

pub struct NestJsBackend {
    pub auth: NestJsAuthApi,
    pub users: NestJsUsersApi,
    pub groups: NestJsGroupsApi,
    pub wallets: NestJsWalletsApi,
}

impl NestJsBackend {
    pub fn new(config: &ApiConfig) -> ApiResult<Self> {
        let client = NestJsClient::new(config)?;
        let auth = NestJsAuthApi::new(client.clone());
        let users = NestJsUsersApi::new(client.clone());
        let groups = NestJsGroupsApi::new(client.clone());
        let wallets = NestJsWalletsApi::new(client.clone());

        Ok(Self {
            auth,
            users,
            groups,
            wallets,
        })
    }
}
