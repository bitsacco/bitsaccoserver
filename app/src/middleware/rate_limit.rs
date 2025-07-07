use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use moka::future::Cache;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_duration: Duration,
    pub cache: Cache<String, RateLimitEntry>,
}

#[derive(Debug, Clone)]
pub struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

impl RateLimitConfig {
    pub fn new(max_requests: u32, window_duration: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(window_duration)
            .build();

        Self {
            max_requests,
            window_duration,
            cache,
        }
    }
}

pub async fn rate_limit_middleware(
    State(rate_limit_config): State<Arc<RateLimitConfig>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|header| header.to_str().ok())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|header| header.to_str().ok())
        })
        .unwrap_or("unknown")
        .to_string();

    let now = Instant::now();

    let mut should_allow = true;

    if let Some(mut entry) = rate_limit_config.cache.get(&client_ip).await {
        // Check if we're still in the same window
        if now.duration_since(entry.window_start) < rate_limit_config.window_duration {
            if entry.count >= rate_limit_config.max_requests {
                should_allow = false;
            } else {
                entry.count += 1;
                rate_limit_config.cache.insert(client_ip, entry).await;
            }
        } else {
            // New window, reset count
            let new_entry = RateLimitEntry {
                count: 1,
                window_start: now,
            };
            rate_limit_config.cache.insert(client_ip, new_entry).await;
        }
    } else {
        // First request from this IP
        let new_entry = RateLimitEntry {
            count: 1,
            window_start: now,
        };
        rate_limit_config.cache.insert(client_ip, new_entry).await;
    }

    if should_allow {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}

pub fn rate_limit_layer() -> tower::layer::util::Identity {
    // For now, return identity layer - will be replaced with actual rate limiting
    tower::layer::util::Identity::new()
}
