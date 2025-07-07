use app::middleware::auth::JwtConfig;
use app::middleware::cors::cors_layer;
use app::repositories::Repositories;
use app::server::{AppConfig, AppState};
use app::services::{auth::KeycloakConfig, Services};
use app::App;
use axum::Router;
use leptos_axum::{generate_route_list, handle_server_fns, LeptosRoutes};
use migration::MigratorTrait;
use std::sync::Arc;
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

    // Initialize database connection with connection pooling
    let database = {
        let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
        opt.max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.acquire_timeout,
            ))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime))
            .sqlx_logging(true)
            .sqlx_logging_level(tracing::log::LevelFilter::Info);

        sea_orm::Database::connect(opt).await?
    };

    // Run database migrations
    migration::Migrator::up(&database, None).await?;

    tracing::info!("Database connected and migrations applied");

    // Initialize JWT configuration
    let jwt_config = JwtConfig::new(
        &config.jwt.public_key,
        &config.jwt.issuer,
        &config.jwt.audience,
    )?;

    // Create application state
    let app_state = Arc::new(AppState {
        database: database.clone(),
        config: config.clone(),
        jwt_config,
    });

    // Initialize repositories
    let repositories = Repositories::new(Arc::new(database.clone()));

    // Initialize Keycloak configuration for services
    let keycloak_config = KeycloakConfig {
        realm: config.keycloak.realm.clone(),
        client_id: config.keycloak.client_id.clone(),
        client_secret: config.keycloak.client_secret.clone(),
        server_url: config.keycloak.auth_server_url.clone(),
    };

    // Initialize services
    let services = Services::new(
        Arc::new(database.clone()),
        repositories.clone(),
        keycloak_config,
    );

    // Create Leptos options for SSR-only mode
    let leptos_options = leptos::config::LeptosOptions::builder()
        .output_name("bitsaccoserver")
        .site_root("target/site")
        .site_pkg_dir("pkg")
        .env(leptos::config::Env::DEV)
        .site_addr(std::net::SocketAddr::from(([0, 0, 0, 0], 3000)))
        .build();
    let routes = generate_route_list(App);

    // Create the main router
    let app = Router::new()
        // Leptos routes first
        .leptos_routes(&leptos_options, routes, App)
        // API routes - comprehensive REST API
        .nest(
            "/api",
            app::api::create_api_router(repositories.clone(), services.clone()).with_state(()),
        )
        // Server functions
        .route("/api/{*fn_name}", axum::routing::any(handle_server_fns))
        // Serve static files (needed for hot-reload assets)
        .nest_service(
            "/pkg",
            ServeDir::new(format!("{}/pkg", leptos_options.site_root)),
        )
        // Serve public assets (CSS, images, etc.)
        .nest_service("/assets", ServeDir::new("public"))
        // Fallback for 404s
        .fallback(|| async { "Page not found" })
        // Middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors_layer()),
        )
        .with_state(leptos_options);

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.server_addr).await?;
    tracing::info!("Server listening on {}", config.server_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
