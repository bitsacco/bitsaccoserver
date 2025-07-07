-- Initialize database extensions and create keycloak database
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- Create keycloak database for Keycloak service
CREATE DATABASE keycloak;
GRANT ALL PRIVILEGES ON DATABASE keycloak TO bitsaccoserver;
