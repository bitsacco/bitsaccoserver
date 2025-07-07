# Environment Variables Reference

This document lists all environment variables used by the Bitsacco Server application.

## Required Variables

### Database Configuration
```bash
DATABASE_URL="postgres://username:password@host:port/database"
```

### Keycloak Configuration
```bash
KEYCLOAK_AUTH_SERVER_URL="http://keycloak-server:8080"
KEYCLOAK_REALM="bitsaccoserver"
KEYCLOAK_CLIENT_ID="bitsaccoserver-app"
KEYCLOAK_CLIENT_SECRET="your-client-secret"
```

## Optional Variables

### JWT Configuration (Production Only)
```bash
# Leave empty for development, set for production
JWT_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----...-----END PUBLIC KEY-----"
JWT_ISSUER="http://keycloak-server:8080/realms/bitsaccoserver"
JWT_AUDIENCE="bitsaccoserver-app"
```

### Database Pool Settings
```bash
DB_MAX_CONNECTIONS="100"          # Default: 100
DB_MIN_CONNECTIONS="5"            # Default: 5
DB_ACQUIRE_TIMEOUT="30"           # Default: 30 seconds
DB_IDLE_TIMEOUT="600"             # Default: 600 seconds (10 minutes)
DB_MAX_LIFETIME="1800"            # Default: 1800 seconds (30 minutes)
```

### Server Configuration
```bash
SERVER_ADDR="0.0.0.0:3000"       # Default: 0.0.0.0:3000
ENVIRONMENT="production"          # Default: development
LOG_LEVEL="info"                  # Default: info (debug, info, warn, error)
```

### CORS Configuration (Production Only)
```bash
# Comma-separated list of allowed origins for CORS
CORS_ALLOWED_ORIGINS="https://app.yourdomain.com,https://admin.yourdomain.com"
```

## Validation

To validate your environment configuration, you can run:

```bash
# Check required variables are set
echo "DATABASE_URL: ${DATABASE_URL:-NOT_SET}"
echo "KEYCLOAK_AUTH_SERVER_URL: ${KEYCLOAK_AUTH_SERVER_URL:-NOT_SET}"
echo "ENVIRONMENT: ${ENVIRONMENT:-development}"

# For production, also check JWT variables
if [ "$ENVIRONMENT" = "production" ]; then
    echo "JWT_PUBLIC_KEY: ${JWT_PUBLIC_KEY:+SET}"
    echo "JWT_ISSUER: ${JWT_ISSUER:-NOT_SET}"
    echo "JWT_AUDIENCE: ${JWT_AUDIENCE:-NOT_SET}"
fi
```
