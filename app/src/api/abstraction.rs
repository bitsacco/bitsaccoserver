use std::sync::Arc;

use crate::api::{
    backends::{nestjs::NestJsBackend, rust::RustBackend},
    config::{ApiConfig, Backend},
    errors::ApiResult,
    traits::groups::GroupsApi,
    traits::wallets::WalletsApi,
    traits::{AuthApi, UsersApi},
};

pub enum ApiBackend {
    NestJs(Arc<NestJsBackend>),
    Rust(Arc<RustBackend>),
}

impl ApiBackend {
    pub fn auth(&self) -> Box<dyn AuthApi> {
        match self {
            ApiBackend::NestJs(backend) => Box::new(backend.auth.clone()),
            ApiBackend::Rust(_backend) => {
                unimplemented!("Rust backend auth not yet implemented")
            }
        }
    }

    pub fn users(&self) -> Box<dyn UsersApi> {
        match self {
            ApiBackend::NestJs(backend) => Box::new(backend.users.clone()),
            ApiBackend::Rust(_backend) => {
                unimplemented!("Rust backend users not yet implemented")
            }
        }
    }

    pub fn groups(&self) -> Box<dyn GroupsApi> {
        match self {
            ApiBackend::NestJs(backend) => Box::new(backend.groups.clone()),
            ApiBackend::Rust(_backend) => {
                unimplemented!("Rust backend groups not yet implemented")
            }
        }
    }

    pub fn wallets(&self) -> Box<dyn WalletsApi> {
        match self {
            ApiBackend::NestJs(backend) => Box::new(backend.wallets.clone()),
            ApiBackend::Rust(_backend) => {
                unimplemented!("Rust backend wallets not yet implemented")
            }
        }
    }
}

pub struct AbstractedApiClient {
    backend: ApiBackend,
}

impl AbstractedApiClient {
    pub fn new(config: ApiConfig) -> ApiResult<Self> {
        let backend = match config.backend {
            Backend::NestJs => {
                let nestjs_backend = NestJsBackend::new(&config)?;
                ApiBackend::NestJs(Arc::new(nestjs_backend))
            }
            Backend::Rust => {
                let rust_backend = RustBackend::new(&config)?;
                ApiBackend::Rust(Arc::new(rust_backend))
            }
        };

        Ok(Self { backend })
    }

    pub fn default() -> ApiResult<Self> {
        let config = ApiConfig::default();
        Self::new(config)
    }

    pub fn auth(&self) -> Box<dyn AuthApi> {
        self.backend.auth()
    }

    pub fn users(&self) -> Box<dyn UsersApi> {
        self.backend.users()
    }

    pub fn groups(&self) -> Box<dyn GroupsApi> {
        self.backend.groups()
    }

    pub fn wallets(&self) -> Box<dyn WalletsApi> {
        self.backend.wallets()
    }
}
