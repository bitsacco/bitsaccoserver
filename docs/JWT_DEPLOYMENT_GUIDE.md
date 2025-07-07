# JWT Public Key Configuration for Deployments

This guide explains how to configure JWT public key authentication for the Bitsacco Server application in different deployment environments.

## Overview

The application uses JWT tokens from Keycloak for authentication. In development, JWT validation is disabled for convenience, but production deployments require proper JWT configuration with Keycloak's public key.

## Development Environment

For local development, JWT validation is automatically disabled when no public key is configured:

```bash
# No JWT_PUBLIC_KEY needed - validation is bypassed
ENVIRONMENT=development
```

## Production Environment

### 1. Get Keycloak Public Key

There are several ways to obtain your Keycloak realm's public key:

#### Option A: From Keycloak Admin Console

1. Access Keycloak Admin Console: `http://your-keycloak-server:8080/admin`
2. Select your realm (e.g., "bitsaccoserver")
3. Go to **Realm Settings** → **Keys** tab
4. Find the **RS256** key and click **Public key**
5. Copy the key and format it as shown below

#### Option B: From JWKS Endpoint

Fetch the public key from Keycloak's JWKS endpoint:

```bash
# Get JWKS
curl http://your-keycloak-server:8080/realms/bitsaccoserver/protocol/openid-connect/certs

# Extract the public key from the response and convert to PEM format
```

#### Option C: Using OpenSSL (if you have the certificate)

```bash
# If you have the certificate file
openssl x509 -pubkey -noout -in keycloak.crt > jwt_public_key.pem
```

### 2. Environment Variable Configuration

Set the JWT configuration in your production environment:

```bash
# Required: JWT Public Key in PEM format
JWT_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAwqKWf6EZdJ7qRiNt7CKF
xXhE1fI4KJ7FJ1MHNB2c6SQO8vMyaBdHKj+qD4PpY9wd8EUKmJ8qLwJVFHkl4tX3
YzrMxR7q8XfYl1Q6fO8nF2bN3hGzRVdI6QqOLy6jZ8xN4qGxQUK3K9XaQ8JBpJcL
VbWe8vXfX1qzJL9xYzFcX2Y8Q7sJ3qJlO8nF6qKxH5Y7qXlH4cYJ1r8YbZ9R4sJ3
qKlX8X4YbQ8R1s9fJ6cF3hGqYzJ1LfX8cJ4Y8L5q7xQb3cV9YzXlH8cR4JbL6Yf5
LcX8YbQ4R1sJ3qKl9X8cJYzFcX2Y8Q7sJ3qJlO8nF6qKxH5Y7qXlH4cYJ1r8YbZ
9wIDAQAB
-----END PUBLIC KEY-----"

# Required: JWT Issuer (must match Keycloak realm)
JWT_ISSUER="http://your-keycloak-server:8080/realms/bitsaccoserver"

# Required: JWT Audience (must match your client ID)
JWT_AUDIENCE="bitsaccoserver-app"

# Keycloak configuration
KEYCLOAK_AUTH_SERVER_URL="http://your-keycloak-server:8080"
KEYCLOAK_REALM="bitsaccoserver"
KEYCLOAK_CLIENT_ID="bitsaccoserver-app"
KEYCLOAK_CLIENT_SECRET="your-client-secret"

# Application environment
ENVIRONMENT="production"
```

### 3. Docker Deployment

#### Option A: Environment Variables

```yaml
# docker-compose.yml
version: '3.8'
services:
  app:
    image: bitsaccoserver:latest
    environment:
      - JWT_PUBLIC_KEY=${JWT_PUBLIC_KEY}
      - JWT_ISSUER=http://keycloak:8080/realms/bitsaccoserver
      - JWT_AUDIENCE=bitsaccoserver-app
      - KEYCLOAK_AUTH_SERVER_URL=http://keycloak:8080
      - KEYCLOAK_REALM=bitsaccoserver
      - KEYCLOAK_CLIENT_ID=bitsaccoserver-app
      - KEYCLOAK_CLIENT_SECRET=${KEYCLOAK_CLIENT_SECRET}
      - ENVIRONMENT=production
```

#### Option B: Using Secrets (Recommended)

```yaml
# docker-compose.yml
version: '3.8'
services:
  app:
    image: bitsaccoserver:latest
    environment:
      - JWT_ISSUER=http://keycloak:8080/realms/bitsaccoserver
      - JWT_AUDIENCE=bitsaccoserver-app
      - KEYCLOAK_AUTH_SERVER_URL=http://keycloak:8080
      - KEYCLOAK_REALM=bitsaccoserver
      - KEYCLOAK_CLIENT_ID=bitsaccoserver-app
      - ENVIRONMENT=production
    secrets:
      - jwt_public_key
      - keycloak_client_secret
    command: |
      sh -c '
        export JWT_PUBLIC_KEY="$$(cat /run/secrets/jwt_public_key)"
        export KEYCLOAK_CLIENT_SECRET="$$(cat /run/secrets/keycloak_client_secret)"
        ./app
      '

secrets:
  jwt_public_key:
    file: ./secrets/jwt_public_key.pem
  keycloak_client_secret:
    file: ./secrets/keycloak_client_secret.txt
```

### 4. Kubernetes Deployment

#### Create Secret for JWT Public Key

```bash
# Create secret from file
kubectl create secret generic jwt-config \
  --from-file=public-key=./jwt_public_key.pem \
  --from-literal=client-secret=your-client-secret
```

#### Deployment YAML

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bitsaccoserver
spec:
  replicas: 3
  selector:
    matchLabels:
      app: bitsaccoserver
  template:
    metadata:
      labels:
        app: bitsaccoserver
    spec:
      containers:
      - name: app
        image: bitsaccoserver:latest
        env:
        - name: JWT_PUBLIC_KEY
          valueFrom:
            secretKeyRef:
              name: jwt-config
              key: public-key
        - name: JWT_ISSUER
          value: "http://keycloak-service:8080/realms/bitsaccoserver"
        - name: JWT_AUDIENCE
          value: "bitsaccoserver-app"
        - name: KEYCLOAK_AUTH_SERVER_URL
          value: "http://keycloak-service:8080"
        - name: KEYCLOAK_REALM
          value: "bitsaccoserver"
        - name: KEYCLOAK_CLIENT_ID
          value: "bitsaccoserver-app"
        - name: KEYCLOAK_CLIENT_SECRET
          valueFrom:
            secretKeyRef:
              name: jwt-config
              key: client-secret
        - name: ENVIRONMENT
          value: "production"
```

## Troubleshooting

### Common Issues

1. **InvalidKeyFormat Error**
   ```
   Error: Error(InvalidKeyFormat)
   ```
   - Ensure the public key is in proper PEM format
   - Verify the key starts with `-----BEGIN PUBLIC KEY-----`
   - Check for extra whitespace or line ending issues

2. **JWT Validation Failures**
   ```
   JWT validation failed: Invalid signature
   ```
   - Verify the public key matches your Keycloak realm
   - Check that the issuer URL is correct
   - Ensure the audience matches your client ID

3. **Connection Issues**
   ```
   Failed to connect to Keycloak
   ```
   - Verify Keycloak server is accessible
   - Check network connectivity between services
   - Ensure correct ports and URLs

### Validation Script

Create a simple validation script to test your JWT configuration:

```bash
#!/bin/bash
# validate-jwt-config.sh

echo "Testing JWT configuration..."

# Test Keycloak connectivity
if curl -f -s "$KEYCLOAK_AUTH_SERVER_URL/realms/$KEYCLOAK_REALM" > /dev/null; then
    echo "✓ Keycloak server is accessible"
else
    echo "✗ Cannot reach Keycloak server"
    exit 1
fi

# Test JWKS endpoint
if curl -f -s "$KEYCLOAK_AUTH_SERVER_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/certs" > /dev/null; then
    echo "✓ JWKS endpoint is accessible"
else
    echo "✗ Cannot reach JWKS endpoint"
    exit 1
fi

# Validate public key format
if echo "$JWT_PUBLIC_KEY" | openssl rsa -pubin -text -noout > /dev/null 2>&1; then
    echo "✓ JWT public key format is valid"
else
    echo "✗ JWT public key format is invalid"
    exit 1
fi

echo "All JWT configuration checks passed!"
```

## Security Best Practices

1. **Key Rotation**: Regularly rotate your Keycloak keys and update the application configuration
2. **Secure Storage**: Store JWT public keys and client secrets securely using secrets management
3. **Environment Separation**: Use different Keycloak realms for different environments
4. **Access Control**: Limit access to JWT configuration and secrets
5. **Monitoring**: Monitor JWT validation failures and authentication metrics

## Production Checklist

- [ ] Keycloak realm is properly configured
- [ ] JWT public key is obtained from production Keycloak
- [ ] Environment variables are set correctly
- [ ] Client secret is stored securely
- [ ] JWT validation is enabled (ENVIRONMENT != "development")
- [ ] Network connectivity between app and Keycloak is verified
- [ ] Monitoring and logging are configured
- [ ] Key rotation procedures are documented