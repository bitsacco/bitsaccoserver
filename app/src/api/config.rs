use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Backend {
    #[default]
    NestJs,
    Rust,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub backend: Backend,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl ApiConfig {
    pub fn new(backend: Backend, base_url: String) -> Self {
        Self {
            backend,
            base_url,
            ..Default::default()
        }
    }

    pub fn from_env() -> Self {
        let backend = match std::env::var("API_BACKEND").as_deref() {
            Ok("rust") => Backend::Rust,
            _ => Backend::NestJs,
        };

        let base_url = match backend {
            Backend::NestJs => std::env::var("NESTJS_API_URL")
                .unwrap_or_else(|_| "http://localhost:4000/v1".to_string()),
            Backend::Rust => std::env::var("RUST_API_URL")
                .unwrap_or_else(|_| "http://localhost:5000".to_string()),
        };

        Self {
            backend,
            base_url,
            timeout_seconds: 30,
            max_retries: 3,
        }
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}
