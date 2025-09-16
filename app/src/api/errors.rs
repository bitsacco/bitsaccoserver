use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    #[error("Authentication failed: {message}")]
    Authentication { message: String },

    #[error("Authorization failed: {message}")]
    Authorization { message: String },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Server error: {message}")]
    Server { message: String },

    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() || error.is_connect() {
            ApiError::Network {
                message: error.to_string(),
            }
        } else if error.is_status() {
            match error.status().map(|s| s.as_u16()) {
                Some(401) => ApiError::Authentication {
                    message: "Unauthorized".to_string(),
                },
                Some(403) => ApiError::Authorization {
                    message: "Forbidden".to_string(),
                },
                Some(404) => ApiError::NotFound {
                    resource: "Resource not found".to_string(),
                },
                Some(409) => ApiError::Conflict {
                    message: "Conflict".to_string(),
                },
                Some(422) => ApiError::Validation {
                    message: "Validation failed".to_string(),
                },
                Some(status) if status >= 500 => ApiError::Server {
                    message: format!("Server error: {}", status),
                },
                _ => ApiError::Unknown {
                    message: error.to_string(),
                },
            }
        } else {
            ApiError::Unknown {
                message: error.to_string(),
            }
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        ApiError::Serialization {
            message: error.to_string(),
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
