use app::server::AppConfig;
use app::App;
use axum::Router;
use leptos_axum::{generate_route_list, handle_server_fns, LeptosRoutes};
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bitsaccoserver_app=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::from_env()?;

    tracing::info!(
        "Starting Bitsacco Dashboard - Pure Frontend Mode with API delegation to configured backend"
    );

    // Create Leptos options for SSR-only mode
    let leptos_options = leptos::config::LeptosOptions::builder()
        .output_name("bitsaccoserver")
        .site_root("target/site")
        .site_pkg_dir("pkg")
        .env(leptos::config::Env::DEV)
        .site_addr(std::net::SocketAddr::from(([0, 0, 0, 0], 3030)))
        .build();
    let routes = generate_route_list(App);

    // Create the main router - clean frontend-only application
    let app = Router::new()
        // Leptos routes first
        .leptos_routes(&leptos_options, routes, App)
        // Server functions for API delegation
        .route("/api/{*fn_name}", axum::routing::any(handle_server_fns))
        // API info endpoint
        .route("/api/info", axum::routing::get(api_info))
        .route("/api/health", axum::routing::get(health_check))
        // Serve static files (needed for hot-reload assets)
        .nest_service(
            "/pkg",
            ServeDir::new(format!("{}/pkg", leptos_options.site_root)),
        )
        // Serve public assets (CSS, images, etc.)
        .nest_service("/assets", ServeDir::new("public"))
        // Fallback for 404s
        .fallback(|| async { "Page not found" })
        // Middleware - minimal setup for frontend-only mode
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors_layer()),
        )
        .with_state(leptos_options);

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.server_addr).await?;
    tracing::info!(
        "Dashboard server listening on {} - API backend: {}",
        config.server_addr,
        std::env::var("API_BACKEND").unwrap_or_else(|_| "nestjs (default)".to_string())
    );

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "bitsacco-dashboard-frontend",
        "mode": "frontend_only",
        "backend": std::env::var("API_BACKEND").unwrap_or_else(|_| "nestjs".to_string())
    }))
}

async fn api_info() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "name": "Bitsacco Dashboard Frontend",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Leptos dashboard for SACCO management system",
        "mode": "frontend_only",
        "backend": std::env::var("API_BACKEND").unwrap_or_else(|_| "nestjs".to_string()),
        "api_delegation": "All API calls delegated to configured backend adapter",
        "supported_backends": ["nestjs", "rust (error-only adapter)"]
    }))
}

// Minimal CORS layer for frontend-only mode
fn cors_layer() -> tower_http::cors::CorsLayer {
    tower_http::cors::CorsLayer::new()
        .allow_origin(
            "http://localhost:3030"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_origin(
            "http://127.0.0.1:3030"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true)
}
