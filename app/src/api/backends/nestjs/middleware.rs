use async_trait::async_trait;
use reqwest::{Request, Response};
use std::sync::Arc;

use crate::api::errors::ApiResult;

/// Trait for implementing request/response interceptors
#[async_trait]
pub trait Interceptor: Send + Sync {
    /// Called before the request is sent
    async fn before_request(&self, request: &mut Request) -> ApiResult<()>;

    /// Called after the response is received
    async fn after_response(
        &self,
        request: &Request,
        response: &Response,
        duration: std::time::Duration,
    ) -> ApiResult<()>;

    /// Called when an error occurs
    async fn on_error(
        &self,
        request: &Request,
        error: &(dyn std::error::Error + Send + Sync),
    ) -> ApiResult<()>;
}

/// Logging interceptor for API requests and responses
#[derive(Debug, Clone)]
pub struct LoggingInterceptor {
    pub log_requests: bool,
    pub log_responses: bool,
    pub log_bodies: bool,
}

impl Default for LoggingInterceptor {
    fn default() -> Self {
        Self {
            log_requests: true,
            log_responses: true,
            log_bodies: false, // Don't log bodies by default for security
        }
    }
}

#[async_trait]
impl Interceptor for LoggingInterceptor {
    async fn before_request(&self, request: &mut Request) -> ApiResult<()> {
        if self.log_requests {
            let method = request.method();
            let url = request.url();

            #[cfg(feature = "ssr")]
            {
                tracing::info!("API Request: {} {}", method, url);

                if self.log_bodies {
                    if let Some(body) = request.body() {
                        tracing::debug!("Request body: {:?}", body);
                    }
                }
            }

            #[cfg(not(feature = "ssr"))]
            {
                web_sys::console::log_1(&format!("API Request: {} {}", method, url).into());
            }
        }
        Ok(())
    }

    async fn after_response(
        &self,
        request: &Request,
        response: &Response,
        duration: std::time::Duration,
    ) -> ApiResult<()> {
        if self.log_responses {
            let method = request.method();
            let url = request.url();
            let status = response.status();

            #[cfg(feature = "ssr")]
            {
                tracing::info!(
                    "API Response: {} {} -> {} ({}ms)",
                    method,
                    url,
                    status,
                    duration.as_millis()
                );
            }

            #[cfg(not(feature = "ssr"))]
            {
                web_sys::console::log_1(
                    &format!(
                        "API Response: {} {} -> {} ({}ms)",
                        method,
                        url,
                        status,
                        duration.as_millis()
                    )
                    .into(),
                );
            }
        }
        Ok(())
    }

    async fn on_error(
        &self,
        request: &Request,
        error: &(dyn std::error::Error + Send + Sync),
    ) -> ApiResult<()> {
        let method = request.method();
        let url = request.url();

        #[cfg(feature = "ssr")]
        {
            tracing::error!("API Error: {} {} -> {}", method, url, error);
        }

        #[cfg(not(feature = "ssr"))]
        {
            web_sys::console::error_1(
                &format!("API Error: {} {} -> {}", method, url, error).into(),
            );
        }

        Ok(())
    }
}

/// Telemetry interceptor for metrics collection
#[derive(Debug, Clone)]
pub struct TelemetryInterceptor {
    pub collect_metrics: bool,
}

impl Default for TelemetryInterceptor {
    fn default() -> Self {
        Self {
            collect_metrics: true,
        }
    }
}

#[async_trait]
impl Interceptor for TelemetryInterceptor {
    async fn before_request(&self, _request: &mut Request) -> ApiResult<()> {
        // Could implement request counting, rate limiting, etc.
        Ok(())
    }

    async fn after_response(
        &self,
        request: &Request,
        response: &Response,
        duration: std::time::Duration,
    ) -> ApiResult<()> {
        if self.collect_metrics {
            let method = request.method().to_string();
            let status = response.status().as_u16();
            let duration_ms = duration.as_millis() as u64;

            #[cfg(feature = "ssr")]
            {
                // In a real implementation, you'd send these metrics to your monitoring system
                tracing::debug!(
                    method = %method,
                    status = %status,
                    duration_ms = %duration_ms,
                    "API metrics collected"
                );
            }

            #[cfg(not(feature = "ssr"))]
            {
                // In client-side, you might send metrics to analytics
                web_sys::console::log_1(
                    &format!("Metrics: {} -> {} ({}ms)", method, status, duration_ms).into(),
                );
            }
        }
        Ok(())
    }

    async fn on_error(
        &self,
        request: &Request,
        error: &(dyn std::error::Error + Send + Sync),
    ) -> ApiResult<()> {
        if self.collect_metrics {
            let method = request.method().to_string();

            #[cfg(feature = "ssr")]
            {
                tracing::debug!(
                    method = %method,
                    error = %error,
                    "API error metrics collected"
                );
            }

            #[cfg(not(feature = "ssr"))]
            {
                web_sys::console::log_1(&format!("Error metrics: {} -> {}", method, error).into());
            }
        }
        Ok(())
    }
}

/// Rate limiting interceptor
#[derive(Debug, Clone)]
pub struct RateLimitInterceptor {
    pub requests_per_second: u32,
    pub max_retries: u32,
}

impl Default for RateLimitInterceptor {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            max_retries: 3,
        }
    }
}

#[async_trait]
impl Interceptor for RateLimitInterceptor {
    async fn before_request(&self, _request: &mut Request) -> ApiResult<()> {
        // Basic rate limiting - in production you'd want a more sophisticated implementation
        // with token bucket or sliding window algorithms
        Ok(())
    }

    async fn after_response(
        &self,
        _request: &Request,
        response: &Response,
        _duration: std::time::Duration,
    ) -> ApiResult<()> {
        // Check for rate limiting responses (429 Too Many Requests)
        if response.status() == 429 {
            #[cfg(feature = "ssr")]
            {
                tracing::warn!("Rate limit exceeded, response status: 429");
            }

            #[cfg(not(feature = "ssr"))]
            {
                web_sys::console::warn_1(&"Rate limit exceeded, response status: 429".into());
            }
        }
        Ok(())
    }

    async fn on_error(
        &self,
        _request: &Request,
        _error: &(dyn std::error::Error + Send + Sync),
    ) -> ApiResult<()> {
        Ok(())
    }
}

/// Middleware chain that manages multiple interceptors
#[derive(Clone)]
pub struct MiddlewareChain {
    interceptors: Vec<Arc<dyn Interceptor>>,
}

impl std::fmt::Debug for MiddlewareChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiddlewareChain")
            .field("interceptor_count", &self.interceptors.len())
            .finish()
    }
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            interceptors: Vec::new(),
        }
    }

    pub fn add_interceptor<T: Interceptor + 'static>(mut self, interceptor: T) -> Self {
        self.interceptors.push(Arc::new(interceptor));
        self
    }

    pub fn with_logging(self) -> Self {
        self.add_interceptor(LoggingInterceptor::default())
    }

    pub fn with_telemetry(self) -> Self {
        self.add_interceptor(TelemetryInterceptor::default())
    }

    pub fn with_rate_limiting(self) -> Self {
        self.add_interceptor(RateLimitInterceptor::default())
    }

    pub async fn before_request(&self, request: &mut Request) -> ApiResult<()> {
        for interceptor in &self.interceptors {
            interceptor.before_request(request).await?;
        }
        Ok(())
    }

    pub async fn after_response(
        &self,
        request: &Request,
        response: &Response,
        duration: std::time::Duration,
    ) -> ApiResult<()> {
        for interceptor in &self.interceptors {
            interceptor
                .after_response(request, response, duration)
                .await?;
        }
        Ok(())
    }

    pub async fn on_error(
        &self,
        request: &Request,
        error: &(dyn std::error::Error + Send + Sync),
    ) -> ApiResult<()> {
        for interceptor in &self.interceptors {
            interceptor.on_error(request, error).await?;
        }
        Ok(())
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new().with_logging().with_telemetry()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Method;

    #[tokio::test]
    async fn test_middleware_chain() {
        let chain = MiddlewareChain::new().with_logging().with_telemetry();

        assert_eq!(chain.interceptors.len(), 2);
    }

    #[tokio::test]
    async fn test_logging_interceptor() {
        let interceptor = LoggingInterceptor::default();
        let mut request = reqwest::Request::new(Method::GET, "http://example.com".parse().unwrap());

        // Should not panic
        let result = interceptor.before_request(&mut request).await;
        assert!(result.is_ok());
    }
}
