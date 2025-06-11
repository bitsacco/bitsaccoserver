#!/bin/bash

# Get access token for Bitsaccoserver members
# Usage: ./get-token.sh [admin|developer|test]

USER_TYPE="${1:-admin}"
KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"

case "$USER_TYPE" in
  "admin")
    USERNAME="admin@bitsaccoserver.org"
    PASSWORD="admin123"
    ;;
  "developer"|"dev")
    USERNAME="developer@bitsaccoserver.org" 
    PASSWORD="dev123"
    ;;
  "test")
    USERNAME="member@bitsaccoserver.org"
    PASSWORD="member123"
    ;;
  *)
    echo "❌ Invalid user type. Use: admin, developer, or test"
    exit 1
    ;;
esac

echo "🔑 Getting access token for $USERNAME..."

RESPONSE=$(curl -s -X POST "$KEYCLOAK_URL/realms/bitsaccoserver-dev/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=$USERNAME" \
  -d "password=$PASSWORD" \
  -d "grant_type=password" \
  -d "client_id=bitsaccoserver" \
  -d "client_secret=bitsaccoserver-secret-dev")

ACCESS_TOKEN=$(echo "$RESPONSE" | jq -r '.access_token')

if [ "$ACCESS_TOKEN" != "null" ] && [ -n "$ACCESS_TOKEN" ]; then
  echo "✅ Success! Copy this token to Swagger UI:"
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "$ACCESS_TOKEN"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  echo "📋 Steps:"
  echo "1. Go to http://localhost:4000/api/docs"
  echo "2. Click the 'Authorize' button (🔒)"
  echo "3. Paste the token above in 'bearerAuth'"
  echo "4. Click 'Authorize' then 'Close'"
  echo ""
  echo "🧪 Test with curl:"
  echo "curl -H 'Authorization: Bearer $ACCESS_TOKEN' http://localhost:4000/api/v1/profile"
  echo ""
  
  # Decode token to show user info
  if command -v jq >/dev/null 2>&1; then
    PAYLOAD=$(echo "$ACCESS_TOKEN" | cut -d'.' -f2)
    # Add padding if needed
    case $((${#PAYLOAD} % 4)) in
      2) PAYLOAD="${PAYLOAD}==" ;;
      3) PAYLOAD="${PAYLOAD}=" ;;
    esac
    
    USER_INFO=$(echo "$PAYLOAD" | base64 -d 2>/dev/null | jq -r '"\(.preferred_username // .sub) (\(.email // "no email"))"' 2>/dev/null)
    if [ -n "$USER_INFO" ] && [ "$USER_INFO" != "null" ]; then
      echo "👤 Token is for: $USER_INFO"
    fi
  fi
else
  echo "❌ Failed to get access token"
  echo "Response: $RESPONSE"
  exit 1
fi