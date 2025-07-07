use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server_addr: String,
    pub database_url: String,
    pub database: DatabaseConfig,
    pub keycloak: KeycloakConfig,
    pub jwt: JwtConfig,
    pub environment: String,
    pub log_level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KeycloakConfig {
    pub auth_server_url: String,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub public_key: String,
    pub issuer: String,
    pub audience: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://bitsaccoserver:password@localhost:5432/bitsaccoserver".to_string()
            }),
            database: DatabaseConfig {
                max_connections: env::var("DB_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                min_connections: env::var("DB_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
                acquire_timeout: env::var("DB_ACQUIRE_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                idle_timeout: env::var("DB_IDLE_TIMEOUT")
                    .unwrap_or_else(|_| "600".to_string())
                    .parse()
                    .unwrap_or(600),
                max_lifetime: env::var("DB_MAX_LIFETIME")
                    .unwrap_or_else(|_| "1800".to_string())
                    .parse()
                    .unwrap_or(1800),
            },
            keycloak: KeycloakConfig {
                auth_server_url: env::var("KEYCLOAK_AUTH_SERVER_URL")
                    .unwrap_or_else(|_| "http://localhost:8080".to_string()),
                realm: env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "bitsaccoserver".to_string()),
                client_id: env::var("KEYCLOAK_CLIENT_ID")
                    .unwrap_or_else(|_| "bitsaccoserver-app".to_string()),
                client_secret: env::var("KEYCLOAK_CLIENT_SECRET")
                    .unwrap_or_else(|_| "".to_string()),
            },
            jwt: JwtConfig {
                public_key: env::var("JWT_PUBLIC_KEY").unwrap_or_else(|_| "".to_string()), // Empty string for development
                issuer: env::var("JWT_ISSUER").unwrap_or_else(|_| {
                    format!(
                        "http://localhost:8080/realms/{}",
                        env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "bitsaccoserver".to_string())
                    )
                }),
                audience: env::var("JWT_AUDIENCE")
                    .unwrap_or_else(|_| "bitsaccoserver-app".to_string()),
            },
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        })
    }
}
