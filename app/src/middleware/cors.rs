use http::{HeaderName, HeaderValue, Method};
use std::env;
use tower_http::cors::{Any, CorsLayer};

pub fn cors_layer() -> CorsLayer {
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    let mut cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            HeaderName::from_static("accept"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("user-agent"),
            HeaderName::from_static("x-requested-with"),
        ]);

    // Configure CORS based on environment
    if environment == "development" {
        // Development: Allow any origin but no credentials for simplicity
        cors = cors.allow_origin(Any);
    } else {
        // Production: Allow specific origins and enable credentials
        let allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,https://yourdomain.com".to_string());

        let origins: Vec<HeaderValue> = allowed_origins
            .split(',')
            .filter_map(|origin| origin.trim().parse().ok())
            .collect();

        cors = cors.allow_origin(origins).allow_credentials(true);
    }

    cors
}
