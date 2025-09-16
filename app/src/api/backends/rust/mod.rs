// Placeholder for future Rust backend implementation

use crate::api::{config::ApiConfig, errors::ApiResult};

pub struct RustBackend {
    // Will be implemented when migrating from NestJS
}

impl RustBackend {
    pub fn new(_config: &ApiConfig) -> ApiResult<Self> {
        // TODO: Implement when ready to migrate from NestJS
        unimplemented!("Rust backend not yet implemented")
    }
}
