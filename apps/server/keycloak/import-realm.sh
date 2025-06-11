#!/bin/bash

# Import Keycloak realm for Bitsaccoserver development
# This script imports the bitsaccoserver-dev realm configuration into Keycloak

set -e

KEYCLOAK_URL="${KEYCLOAK_URL:-http://0.0.0.0:8080}"
KEYCLOAK_ADMIN="${KEYCLOAK_ADMIN:-admin}"
KEYCLOAK_ADMIN_PASSWORD="${KEYCLOAK_ADMIN_PASSWORD:-admin123}"
REALM_FILE="${REALM_FILE:-./bitsaccoserver-dev-realm.json}"

echo "🔧 Setting up Keycloak realm for Bitsaccoserver development..."
echo "   Keycloak URL: $KEYCLOAK_URL"
echo "   Realm file: $REALM_FILE"

# Wait for Keycloak to be ready
echo "⏳ Waiting for Keycloak to be ready..."
MAX_RETRIES=30
RETRY_COUNT=0

until curl -f "$KEYCLOAK_URL/realms/master" >/dev/null 2>&1; do
  RETRY_COUNT=$((RETRY_COUNT + 1))
  if [ $RETRY_COUNT -ge $MAX_RETRIES ]; then
    echo "❌ Timeout waiting for Keycloak. Check if it's running at $KEYCLOAK_URL"
    exit 1
  fi
  echo "   Waiting for Keycloak... (attempt $RETRY_COUNT/$MAX_RETRIES)"
  sleep 5
done

echo "✅ Keycloak is ready!"

# Get admin access token
echo "🔑 Getting admin access token..."
ADMIN_TOKEN=$(curl -s -X POST "$KEYCLOAK_URL/realms/master/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=$KEYCLOAK_ADMIN" \
  -d "password=$KEYCLOAK_ADMIN_PASSWORD" \
  -d "grant_type=password" \
  -d "client_id=admin-cli" | jq -r '.access_token')

if [ "$ADMIN_TOKEN" = "null" ] || [ -z "$ADMIN_TOKEN" ]; then
  echo "❌ Failed to get admin token. Check admin credentials."
  exit 1
fi

echo "✅ Got admin token"

# Check if realm already exists
echo "🔍 Checking if bitsaccoserver-dev realm exists..."
REALM_EXISTS=$(curl -s -o /dev/null -w "%{http_code}" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  "$KEYCLOAK_URL/admin/realms/bitsaccoserver-dev")

if [ "$REALM_EXISTS" = "200" ]; then
  echo "⚠️  Realm 'bitsaccoserver-dev' already exists. Updating..."
  
  # Update existing realm
  curl -s -X PUT \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d @"$REALM_FILE" \
    "$KEYCLOAK_URL/admin/realms/bitsaccoserver-dev"
  
  echo "✅ Realm 'bitsaccoserver-dev' updated successfully!"
else
  echo "➕ Creating new realm 'bitsaccoserver-dev'..."
  
  # Create new realm
  echo "📄 Validating realm file..."
  if ! jq empty "$REALM_FILE" 2>/dev/null; then
    echo "❌ Invalid JSON in realm file: $REALM_FILE"
    exit 1
  fi
  
  RESPONSE_FILE="/tmp/keycloak_response_$$.json"
  RESPONSE=$(curl -s -w "%{http_code}" -o "$RESPONSE_FILE" \
    -X POST \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d @"$REALM_FILE" \
    "$KEYCLOAK_URL/admin/realms")
  
  if [ "$RESPONSE" = "201" ]; then
    echo "✅ Realm 'bitsaccoserver-dev' created successfully!"
  else
    echo "❌ Failed to create realm. HTTP status: $RESPONSE"
    echo "Response:"
    cat "$RESPONSE_FILE"
    echo ""
    echo "💡 Debug info:"
    echo "   Admin token length: ${#ADMIN_TOKEN}"
    echo "   Realm file size: $(wc -c < "$REALM_FILE") bytes"
    
    # Try to get more specific error
    if [ -f "$RESPONSE_FILE" ]; then
      ERROR_MSG=$(jq -r '.errorMessage // .error_description // .error // "Unknown error"' "$RESPONSE_FILE" 2>/dev/null || echo "Could not parse error")
      echo "   Error message: $ERROR_MSG"
    fi
    
    rm -f "$RESPONSE_FILE"
    
    echo ""
    echo "🔄 Trying manual realm creation as fallback..."
    if [ -x "./import-realm-manual.sh" ]; then
      ./import-realm-manual.sh
      exit $?
    else
      echo "❌ Manual import script not found"
      exit 1
    fi
  fi
  
  rm -f "$RESPONSE_FILE"
fi

echo ""
echo "🎉 Keycloak setup complete!"
echo ""
echo "📋 Test Users Created:"
echo "   👤 Admin User:"
echo "      Email: admin@bitsaccoserver.org"
echo "      Password: admin123"
echo "      Roles: bitsaccoserver-admin, bitsaccoserver-member"
echo ""
echo "   👤 Developer User:"
echo "      Email: developer@bitsaccoserver.org" 
echo "      Password: dev123"
echo "      Roles: bitsaccoserver-developer, bitsaccoserver-member"
echo ""
echo "   👤 Basic Test User:"
echo "      Email: member@bitsaccoserver.org"
echo "      Password: member123"
echo "      Roles: bitsaccoserver-member"
echo ""
echo "🔧 Console Service Configuration:"
echo "   KEYCLOAK_AUTH_SERVER_URL=$KEYCLOAK_URL"
echo "   KEYCLOAK_REALM=bitsaccoserver-dev"
echo "   KEYCLOAK_CLIENT_ID=bitsaccoserver"
echo "   KEYCLOAK_CLIENT_SECRET=bitsaccoserver-secret-dev"
echo ""
echo "🌐 Access Keycloak Admin Console:"
echo "   URL: $KEYCLOAK_URL/admin/"
echo "   Username: $KEYCLOAK_ADMIN"
echo "   Password: $KEYCLOAK_ADMIN_PASSWORD"
echo ""