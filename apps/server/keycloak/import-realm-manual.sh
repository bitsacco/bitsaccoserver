#!/bin/bash

# Manual step-by-step realm creation for Keycloak
# Use this if the full import fails

set -e

KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"
KEYCLOAK_ADMIN="${KEYCLOAK_ADMIN:-admin}"
KEYCLOAK_ADMIN_PASSWORD="${KEYCLOAK_ADMIN_PASSWORD:-admin123}"

echo "🔧 Creating bitsaccoserver-dev realm manually..."

# Wait for Keycloak
until curl -f "$KEYCLOAK_URL/realms/master" >/dev/null 2>&1; do
  echo "   Waiting for Keycloak..."
  sleep 2
done

# Get admin token
ADMIN_TOKEN=$(curl -s -X POST "$KEYCLOAK_URL/realms/master/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=$KEYCLOAK_ADMIN" \
  -d "password=$KEYCLOAK_ADMIN_PASSWORD" \
  -d "grant_type=password" \
  -d "client_id=admin-cli" | jq -r '.access_token')

# 1. Create basic realm
echo "1️⃣ Creating realm..."
curl -s -X POST "$KEYCLOAK_URL/admin/realms" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "realm": "bitsaccoserver-dev",
    "displayName": "Bitsaccoserver Development", 
    "enabled": true,
    "loginWithEmailAllowed": true
  }'

# 2. Create bitsaccoserver client
echo "2️⃣ Creating bitsaccoserver client..."
curl -s -X POST "$KEYCLOAK_URL/admin/realms/bitsaccoserver-dev/clients" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "clientId": "bitsaccoserver",
    "enabled": true,
    "clientAuthenticatorType": "client-secret",
    "secret": "bitsaccoserver-secret-dev",
    "redirectUris": ["http://localhost:3000/*", "http://localhost:3001/*"],
    "webOrigins": ["http://localhost:3000", "http://localhost:3001"],
    "standardFlowEnabled": true,
    "directAccessGrantsEnabled": true,
    "publicClient": false
  }'

# 3. Create roles
echo "3️⃣ Creating roles..."
for role in "bitsaccoserver-admin:Full access to console management" "bitsaccoserver-developer:Developer access" "bitsaccoserver-member:Basic member"; do
  name="${role%%:*}"
  desc="${role#*:}"
  curl -s -X POST "$KEYCLOAK_URL/admin/realms/bitsaccoserver-dev/roles" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"name\": \"$name\", \"description\": \"$desc\"}"
done

# 4. Create users
echo "4️⃣ Creating users..."

# Admin user
curl -s -X POST "$KEYCLOAK_URL/admin/realms/bitsaccoserver-dev/users" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin@bitsaccoserver.org",
    "email": "admin@bitsaccoserver.org",
    "firstName": "Admin",
    "lastName": "User",
    "enabled": true,
    "emailVerified": true,
    "credentials": [{"type": "password", "value": "admin123", "temporary": false}],
    "realmRoles": ["bitsaccoserver-admin", "bitsaccoserver-member"]
  }'

# Developer user  
curl -s -X POST "$KEYCLOAK_URL/admin/realms/bitsaccoserver-dev/users" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "developer@bitsaccoserver.org",
    "email": "developer@bitsaccoserver.org", 
    "firstName": "Dev",
    "lastName": "User",
    "enabled": true,
    "emailVerified": true,
    "credentials": [{"type": "password", "value": "dev123", "temporary": false}],
    "realmRoles": ["bitsaccoserver-developer", "bitsaccoserver-member"]
  }'

# Test user
curl -s -X POST "$KEYCLOAK_URL/admin/realms/bitsaccoserver-dev/users" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "member@bitsaccoserver.org",
    "email": "member@bitsaccoserver.org",
    "firstName": "Test",
    "lastName": "User", 
    "enabled": true,
    "emailVerified": true,
    "credentials": [{"type": "password", "value": "member123", "temporary": false}],
    "realmRoles": ["bitsaccoserver-member"]
  }'

echo ""
echo "✅ Manual realm setup complete!"
echo ""
echo "🧪 Test login:"
echo "curl -X POST $KEYCLOAK_URL/realms/bitsaccoserver-dev/protocol/openid-connect/token \\"
echo "  -d 'username=admin@bitsaccoserver.org&password=admin123&grant_type=password&client_id=bitsaccoserver&client_secret=bitsaccoserver-secret-dev'"
echo ""
