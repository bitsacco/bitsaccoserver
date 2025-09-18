use serde::Deserialize;
use std::env;

/// Simplified configuration for frontend-only application
/// All backend functionality is delegated to external APIs
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Server address for the Leptos frontend application
    pub server_addr: String,

    /// Backend API configuration
    pub api_backend: String,
    pub nestjs_api_url: String,

    /// Application environment (development, production)
    pub environment: String,

    /// Log level for the application
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:3030".to_string()),
            api_backend: env::var("API_BACKEND").unwrap_or_else(|_| "nestjs".to_string()),
            nestjs_api_url: env::var("NESTJS_API_URL")
                .unwrap_or_else(|_| "http://localhost:4000".to_string()),
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        })
    }

    /// Check if the application is running in development mode
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }

    /// Get the configured backend API base URL
    pub fn get_api_url(&self) -> String {
        match self.api_backend.as_str() {
            "nestjs" => self.nestjs_api_url.clone(),
            "rust" => {
                // Rust backend is not implemented, return NestJS URL with a warning
                tracing::warn!("Rust backend is not implemented, falling back to NestJS");
                self.nestjs_api_url.clone()
            }
            _ => {
                tracing::warn!(
                    "Unknown backend '{}', defaulting to NestJS",
                    self.api_backend
                );
                self.nestjs_api_url.clone()
            }
        }
    }
}
