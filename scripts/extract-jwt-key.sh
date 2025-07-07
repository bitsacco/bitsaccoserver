#!/bin/bash

# Script to extract JWT public key from Keycloak
# Usage: ./extract-jwt-key.sh <keycloak-url> <realm-name>

set -e

KEYCLOAK_URL="${1:-http://localhost:8080}"
REALM_NAME="${2:-bitsaccoserver}"

echo "Extracting JWT public key from Keycloak..."
echo "Keycloak URL: $KEYCLOAK_URL"
echo "Realm: $REALM_NAME"
echo

# Construct JWKS URL
JWKS_URL="${KEYCLOAK_URL}/realms/${REALM_NAME}/protocol/openid-connect/certs"

echo "Fetching JWKS from: $JWKS_URL"

# Fetch JWKS and extract the first RSA key
JWKS_RESPONSE=$(curl -s "$JWKS_URL")

if [ $? -ne 0 ]; then
    echo "Error: Failed to fetch JWKS from Keycloak"
    echo "Please check if Keycloak is running and accessible at: $KEYCLOAK_URL"
    exit 1
fi

# Check if response contains keys
if ! echo "$JWKS_RESPONSE" | jq -e '.keys' > /dev/null 2>&1; then
    echo "Error: Invalid JWKS response or no keys found"
    echo "Response: $JWKS_RESPONSE"
    exit 1
fi

# Extract the first RSA key's modulus and exponent
KEY_DATA=$(echo "$JWKS_RESPONSE" | jq -r '.keys[] | select(.kty=="RSA" and .use=="sig") | {n: .n, e: .e} | @base64')

if [ -z "$KEY_DATA" ] || [ "$KEY_DATA" = "null" ]; then
    echo "Error: No RSA signing keys found in JWKS"
    echo "Available keys:"
    echo "$JWKS_RESPONSE" | jq -r '.keys[] | "- \(.kty) key with use: \(.use // "none")"'
    exit 1
fi

# Note: Converting from JWKS to PEM requires additional tools
# For now, provide instructions on how to get the key

echo "✓ Found RSA signing key in JWKS"
echo
echo "JWKS RSA Key Details:"
echo "$JWKS_RESPONSE" | jq -r '.keys[] | select(.kty=="RSA" and .use=="sig") | {kid: .kid, alg: .alg, use: .use}'
echo

echo "To get the PEM format public key, you have several options:"
echo
echo "1. Copy from Keycloak Admin Console:"
echo "   - Go to: ${KEYCLOAK_URL}/admin"
echo "   - Navigate to: Realm Settings > Keys"
echo "   - Find the RS256 key and click 'Public key'"
echo "   - Copy the key and format as PEM"
echo
echo "2. Use the key from JWKS (requires additional conversion):"
echo "   The JWKS endpoint is: $JWKS_URL"
echo
echo "3. Manual PEM format (if you have the certificate):"
cat << 'EOF'
   JWT_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----
   MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...
   -----END PUBLIC KEY-----"
EOF
echo
echo "4. For development, leave JWT_PUBLIC_KEY empty to disable validation"

# Try to provide a direct link to admin console
echo
echo "Direct link to Keycloak keys:"
echo "${KEYCLOAK_URL}/admin/master/console/#/${REALM_NAME}/realm-settings/keys"