use reqwest::{Client, ClientBuilder, Method, RequestBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use super::jwt_validator::{Claims, JwtValidator};
use super::middleware::MiddlewareChain;
use crate::api::{config::ApiConfig, errors::ApiResult};

#[derive(Debug, Clone)]
pub struct NestJsClient {
    client: Arc<Client>,
    base_url: String,
    max_retries: u32,
    jwt_validator: JwtValidator,
    _middleware: MiddlewareChain,
}

impl NestJsClient {
    pub fn new(config: &ApiConfig) -> ApiResult<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| crate::api::errors::ApiError::Network {
                message: e.to_string(),
            })?;

        // TODO: Load JWT secret from environment or config
        let jwt_secret =
            std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());
        let jwt_validator = JwtValidator::new(&jwt_secret);

        // Initialize middleware chain with default interceptors
        let middleware = MiddlewareChain::default();

        Ok(Self {
            client: Arc::new(client),
            base_url: config.base_url.clone(),
            max_retries: config.max_retries,
            jwt_validator,
            _middleware: middleware,
        })
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.request(Method::GET, path)
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.request(Method::POST, path)
    }

    pub fn put(&self, path: &str) -> RequestBuilder {
        self.request(Method::PUT, path)
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.request(Method::DELETE, path)
    }

    pub fn patch(&self, path: &str) -> RequestBuilder {
        self.request(Method::PATCH, path)
    }

    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}{}", self.base_url.trim_end_matches('/'), path)
        };

        self.client.request(method, &url)
    }

    pub async fn send_json<T, R>(&self, request: RequestBuilder, body: &T) -> ApiResult<R>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let mut attempts = 0;

        loop {
            let cloned_request =
                request
                    .try_clone()
                    .ok_or_else(|| crate::api::errors::ApiError::Network {
                        message: "Failed to clone request for retry".to_string(),
                    })?;

            let result = cloned_request
                .header("Content-Type", "application/json")
                .json(body)
                .send()
                .await;

            match result {
                Ok(response) => {
                    if attempts < self.max_retries && self.should_retry(&response) {
                        attempts += 1;
                        let delay = Duration::from_millis(100 * (1 << attempts));
                        sleep(delay).await;
                        continue;
                    }
                    return self.handle_response(response).await;
                }
                Err(e) => {
                    if attempts < self.max_retries && self.should_retry_error(&e) {
                        attempts += 1;
                        let delay = Duration::from_millis(100 * (1 << attempts));
                        sleep(delay).await;
                        continue;
                    }
                    return Err(e.into());
                }
            }
        }
    }

    pub async fn send<R>(&self, request: RequestBuilder) -> ApiResult<R>
    where
        R: DeserializeOwned,
    {
        let mut attempts = 0;

        loop {
            let cloned_request =
                request
                    .try_clone()
                    .ok_or_else(|| crate::api::errors::ApiError::Network {
                        message: "Failed to clone request for retry".to_string(),
                    })?;

            let result = cloned_request.send().await;

            match result {
                Ok(response) => {
                    if attempts < self.max_retries && self.should_retry(&response) {
                        attempts += 1;
                        let delay = Duration::from_millis(100 * (1 << attempts));
                        sleep(delay).await;
                        continue;
                    }
                    return self.handle_response(response).await;
                }
                Err(e) => {
                    if attempts < self.max_retries && self.should_retry_error(&e) {
                        attempts += 1;
                        let delay = Duration::from_millis(100 * (1 << attempts));
                        sleep(delay).await;
                        continue;
                    }
                    return Err(e.into());
                }
            }
        }
    }

    fn should_retry(&self, response: &reqwest::Response) -> bool {
        // Retry on 5xx server errors and 429 Too Many Requests
        response.status().is_server_error() || response.status() == 429
    }

    fn should_retry_error(&self, error: &reqwest::Error) -> bool {
        // Retry on timeout, connection errors, but not on client errors
        error.is_timeout() || error.is_connect()
    }

    /// Validate a JWT token and return the claims
    pub fn validate_jwt(&self, token: &str) -> ApiResult<Claims> {
        self.jwt_validator.validate_token(token)
    }

    /// Create a request with Authorization header if token is valid
    pub fn authenticated_request(
        &self,
        method: Method,
        path: &str,
        token: &str,
    ) -> ApiResult<RequestBuilder> {
        // Validate token first
        let claims = self.validate_jwt(token)?;

        // Check if token is expired
        if self.jwt_validator.is_token_expired(&claims) {
            return Err(crate::api::errors::ApiError::Authentication {
                message: "JWT token has expired".to_string(),
            });
        }

        Ok(self
            .request(method, path)
            .header("Authorization", format!("Bearer {}", token)))
    }

    async fn handle_response<R>(&self, response: reqwest::Response) -> ApiResult<R>
    where
        R: DeserializeOwned,
    {
        let status = response.status();
        let text = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&text).map_err(|e| crate::api::errors::ApiError::Serialization {
                message: format!("Failed to parse response: {} - Body: {}", e, text),
            })
        } else {
            // Try to parse error response
            if let Ok(error_response) = serde_json::from_str::<serde_json::Value>(&text) {
                let message = error_response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&text)
                    .to_string();

                match status.as_u16() {
                    401 => Err(crate::api::errors::ApiError::Authentication { message }),
                    403 => Err(crate::api::errors::ApiError::Authorization { message }),
                    404 => Err(crate::api::errors::ApiError::NotFound { resource: message }),
                    409 => Err(crate::api::errors::ApiError::Conflict { message }),
                    422 => Err(crate::api::errors::ApiError::Validation { message }),
                    _ if status.is_server_error() => {
                        Err(crate::api::errors::ApiError::Server { message })
                    }
                    _ => Err(crate::api::errors::ApiError::Unknown { message }),
                }
            } else {
                Err(crate::api::errors::ApiError::Server {
                    message: format!("HTTP {}: {}", status, text),
                })
            }
        }
    }
}
