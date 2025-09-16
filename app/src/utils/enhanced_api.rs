use crate::contexts::auth::use_auth_token;
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use js_sys;
use serde::{Deserialize, Serialize};

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
    pub code: Option<String>,
    pub retry_after: Option<u32>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError {
            message: format!("JSON serialization error: {}", err),
            code: Some("JSON_ERROR".to_string()),
            retry_after: None,
        }
    }
}

impl std::error::Error for ApiError {}

pub struct EnhancedApiClient;

impl EnhancedApiClient {
    pub fn get(url: &str) -> EnhancedRequestBuilder {
        EnhancedRequestBuilder::new("GET", url)
    }

    pub fn post(url: &str) -> EnhancedRequestBuilder {
        EnhancedRequestBuilder::new("POST", url)
    }

    pub fn put(url: &str) -> EnhancedRequestBuilder {
        EnhancedRequestBuilder::new("PUT", url)
    }

    pub fn patch(url: &str) -> EnhancedRequestBuilder {
        EnhancedRequestBuilder::new("PATCH", url)
    }

    pub fn delete(url: &str) -> EnhancedRequestBuilder {
        EnhancedRequestBuilder::new("DELETE", url)
    }
}

pub struct EnhancedRequestBuilder {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    with_auth: bool,
    retry_count: u32,
    timeout_ms: Option<u32>,
    cache_bust: bool,
    idempotent: bool,
}

impl EnhancedRequestBuilder {
    fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Accept".to_string(), "application/json".to_string()),
                ("X-Requested-With".to_string(), "XMLHttpRequest".to_string()),
            ],
            body: None,
            with_auth: true,
            retry_count: if method == "GET" { 3 } else { 1 }, // Only retry safe operations by default
            timeout_ms: Some(30000),                          // 30 second default timeout
            cache_bust: false,
            idempotent: method == "GET" || method == "PUT" || method == "DELETE",
        }
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self, serde_json::Error> {
        self.body = Some(serde_json::to_string(data)?);
        Ok(self)
    }

    pub fn without_auth(mut self) -> Self {
        self.with_auth = false;
        self
    }

    pub fn retry(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    pub fn timeout(mut self, timeout_ms: u32) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn no_timeout(mut self) -> Self {
        self.timeout_ms = None;
        self
    }

    pub fn cache_bust(mut self) -> Self {
        self.cache_bust = true;
        self
    }

    pub fn idempotent(mut self, is_idempotent: bool) -> Self {
        self.idempotent = is_idempotent;
        self
    }

    pub async fn send<T: for<'de> Deserialize<'de>>(self) -> ApiResult<T> {
        self.send_with_retry().await
    }

    pub async fn send_empty(self) -> ApiResult<()> {
        self.send_empty_with_retry().await
    }

    async fn send_with_retry<T: for<'de> Deserialize<'de>>(self) -> ApiResult<T> {
        let mut last_error = None;
        let max_attempts = if self.idempotent {
            self.retry_count + 1
        } else {
            1
        };

        for attempt in 0..max_attempts {
            if attempt > 0 {
                // Exponential backoff with jitter: 100ms, 200ms, 400ms, 800ms...
                let base_delay = 100 * (2_u32.pow(attempt - 1));
                let jitter = (js_sys::Math::random() * 50.0) as u32; // 0-50ms jitter
                let delay_ms = base_delay + jitter;

                tracing::debug!(
                    "Retrying request to {} (attempt {}/{}) after {}ms",
                    self.url,
                    attempt + 1,
                    max_attempts,
                    delay_ms
                );

                TimeoutFuture::new(delay_ms).await;
            }

            match self.execute_request().await {
                Ok(response) => {
                    return self.handle_response::<T>(response).await;
                }
                Err(e) => {
                    last_error = Some(e);

                    // Don't retry on certain error types
                    if let Some(ref error) = last_error {
                        if let Some(ref code) = error.code {
                            // Don't retry 4xx errors (except 429 rate limit)
                            if code.starts_with("HTTP_4") && !code.contains("429") {
                                tracing::debug!("Not retrying {} error: {}", code, error.message);
                                break;
                            }
                        }
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ApiError {
            message: "Unknown error occurred after retries".to_string(),
            code: Some("UNKNOWN_ERROR".to_string()),
            retry_after: None,
        }))
    }

    async fn send_empty_with_retry(self) -> ApiResult<()> {
        let mut last_error = None;
        let max_attempts = if self.idempotent {
            self.retry_count + 1
        } else {
            1
        };

        for attempt in 0..max_attempts {
            if attempt > 0 {
                let base_delay = 100 * (2_u32.pow(attempt - 1));
                let jitter = (js_sys::Math::random() * 50.0) as u32;
                let delay_ms = base_delay + jitter;
                TimeoutFuture::new(delay_ms).await;
            }

            match self.execute_request().await {
                Ok(response) => {
                    return self.handle_empty_response(response).await;
                }
                Err(e) => {
                    last_error = Some(e);

                    if let Some(ref error) = last_error {
                        if let Some(ref code) = error.code {
                            if code.starts_with("HTTP_4") && !code.contains("429") {
                                break;
                            }
                        }
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ApiError {
            message: "Unknown error occurred after retries".to_string(),
            code: Some("UNKNOWN_ERROR".to_string()),
            retry_after: None,
        }))
    }

    async fn execute_request(&self) -> Result<gloo_net::http::Response, ApiError> {
        let mut url = self.url.clone();

        // Add cache busting parameter if requested
        if self.cache_bust {
            let separator = if url.contains('?') { "&" } else { "?" };
            url = format!("{}{}t={}", url, separator, js_sys::Date::now() as u64);
        }

        let mut request = match self.method.as_str() {
            "GET" => Request::get(&url),
            "POST" => Request::post(&url),
            "PUT" => Request::put(&url),
            "PATCH" => Request::patch(&url),
            "DELETE" => Request::delete(&url),
            _ => {
                return Err(ApiError {
                    message: "Unsupported HTTP method".to_string(),
                    code: Some("INVALID_METHOD".to_string()),
                    retry_after: None,
                })
            }
        };

        // Add headers
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        // Add authentication header if required
        if self.with_auth {
            if let Some(token) = use_auth_token() {
                request = request.header("Authorization", &format!("Bearer {}", token));
            } else {
                return Err(ApiError {
                    message: "Authentication required but no token available".to_string(),
                    code: Some("NO_AUTH_TOKEN".to_string()),
                    retry_after: None,
                });
            }
        }

        // Add request ID for tracking
        let request_id = format!("req_{}", js_sys::Date::now() as u64);
        request = request.header("X-Request-ID", &request_id);

        // Add body if present and build request
        let request = if let Some(body) = &self.body {
            request.body(body.clone()).map_err(|e| ApiError {
                message: format!("Failed to set request body: {}", e),
                code: Some("REQUEST_BUILD_ERROR".to_string()),
                retry_after: None,
            })?
        } else {
            request.build().map_err(|e| ApiError {
                message: format!("Failed to build request: {}", e),
                code: Some("REQUEST_BUILD_ERROR".to_string()),
                retry_after: None,
            })?
        };

        // Send request with optional timeout
        let response = if let Some(_timeout_ms) = self.timeout_ms {
            // Note: gloo_net doesn't have built-in timeout support
            // In a real implementation, you'd use a timeout wrapper
            request.send().await.map_err(|e| {
                tracing::error!("Network error for {}: {}", url, e);
                ApiError {
                    message: if e.to_string().contains("network") {
                        "Network connection failed. Please check your internet connection."
                            .to_string()
                    } else {
                        format!("Request failed: {}", e)
                    },
                    code: Some("NETWORK_ERROR".to_string()),
                    retry_after: None,
                }
            })?
        } else {
            request.send().await.map_err(|e| {
                tracing::error!("Network error for {}: {}", url, e);
                ApiError {
                    message: format!("Network error: {}", e),
                    code: Some("NETWORK_ERROR".to_string()),
                    retry_after: None,
                }
            })?
        };

        Ok(response)
    }

    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: gloo_net::http::Response,
    ) -> ApiResult<T> {
        let status = response.status();

        if response.ok() {
            response.json::<T>().await.map_err(|e| {
                tracing::error!("JSON parsing error for {}: {}", self.url, e);
                ApiError {
                    message: "Invalid response format from server".to_string(),
                    code: Some("PARSE_ERROR".to_string()),
                    retry_after: None,
                }
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Extract retry-after header if present
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|h| h.parse::<u32>().ok());

            tracing::error!("HTTP error {} for {}: {}", status, self.url, error_text);

            let (error_code, user_message) = Self::categorize_error(status, &error_text);

            Err(ApiError {
                message: user_message,
                code: Some(format!("{}_{}", error_code, status)),
                retry_after,
            })
        }
    }

    async fn handle_empty_response(&self, response: gloo_net::http::Response) -> ApiResult<()> {
        let status = response.status();

        if response.ok() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|h| h.parse::<u32>().ok());

            let (error_code, user_message) = Self::categorize_error(status, &error_text);

            Err(ApiError {
                message: user_message,
                code: Some(format!("{}_{}", error_code, status)),
                retry_after,
            })
        }
    }

    fn categorize_error(status: u16, error_text: &str) -> (&'static str, String) {
        match status {
            400 => (
                "BAD_REQUEST",
                "Invalid request. Please check your input.".to_string(),
            ),
            401 => (
                "UNAUTHORIZED",
                "Your session has expired. Please log in again.".to_string(),
            ),
            403 => (
                "FORBIDDEN",
                "You don't have permission to perform this action.".to_string(),
            ),
            404 => (
                "NOT_FOUND",
                "The requested resource was not found.".to_string(),
            ),
            409 => (
                "CONFLICT",
                "This action conflicts with the current state. Please refresh and try again."
                    .to_string(),
            ),
            422 => ("VALIDATION_ERROR", {
                if error_text.len() > 200 {
                    "Validation failed. Please check your input.".to_string()
                } else {
                    error_text.to_string()
                }
            }),
            429 => (
                "RATE_LIMITED",
                "Too many requests. Please wait before trying again.".to_string(),
            ),
            500 => (
                "SERVER_ERROR",
                "Internal server error. Please try again later.".to_string(),
            ),
            502 => (
                "BAD_GATEWAY",
                "Service temporarily unavailable. Please try again later.".to_string(),
            ),
            503 => (
                "SERVICE_UNAVAILABLE",
                "Service temporarily unavailable. Please try again later.".to_string(),
            ),
            504 => (
                "GATEWAY_TIMEOUT",
                "Request timeout. Please try again.".to_string(),
            ),
            _ => ("HTTP_ERROR", {
                if error_text.len() > 100 {
                    format!("Request failed ({}). Please try again.", status)
                } else {
                    format!("HTTP {}: {}", status, error_text)
                }
            }),
        }
    }
}

// Helper functions for common patterns
pub async fn get_json<T: for<'de> Deserialize<'de>>(url: &str) -> ApiResult<T> {
    EnhancedApiClient::get(url).cache_bust().send().await
}

pub async fn post_json<T: Serialize, R: for<'de> Deserialize<'de>>(
    url: &str,
    data: &T,
) -> ApiResult<R> {
    EnhancedApiClient::post(url).json(data)?.send().await
}

pub async fn put_json<T: Serialize, R: for<'de> Deserialize<'de>>(
    url: &str,
    data: &T,
) -> ApiResult<R> {
    EnhancedApiClient::put(url)
        .json(data)?
        .idempotent(true)
        .retry(2)
        .send()
        .await
}

pub async fn delete_resource(url: &str) -> ApiResult<()> {
    EnhancedApiClient::delete(url)
        .idempotent(true)
        .retry(2)
        .send_empty()
        .await
}
