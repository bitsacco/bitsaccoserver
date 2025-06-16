# Keycloak Setup for Bitsaccoserver Service

This directory contains the Keycloak realm configuration for the Bitsaccoserver development environment.

## Quick Setup

1. **Start Keycloak** (if not already running):

   ```bash
   cd /path/to/bitsaccoserver
   docker compose up keycloak postgres -d
   ```

2. **Import the realm**:

   ```bash
   cd apps/server/keycloak
   ./import-realm.sh
   ```

3. **Verify setup** - Access Keycloak Admin Console:
   - URL: http://localhost:8080/admin/
   - Username: `admin`
   - Password: `admin123`
   - Check that `bitsaccoserver-dev` realm exists with members and clients

## Test Users

The realm includes the following test members:

| Email                        | Password  | Role                 | Permissions             |
| ---------------------------- | --------- | -------------------- | ----------------------- |
| admin@bitsaccoserver.org     | admin123  | bitsaccoserver-admin | Full console management |
| developer@bitsaccoserver.org | dev123    | bitsaccoserver-admin | API keys, view usage    |
| member@bitsaccoserver.org    | member123 | member               | Basic access            |

## Clients

### bitsaccoserver

- **Client ID**: `bitsaccoserver`
- **Secret**: `bitsaccoserver-secret-dev`
- **Type**: Confidential (for backend API)
- **Flows**: Authorization Code, Direct Access

- **Flows**: Authorization Code with PKCE

## Manual Import (Alternative)

If the script doesn't work, you can manually import:

1. Access Keycloak Admin Console
2. Click "Add realm"
3. Select "Import" and upload `bitsaccoserver-dev-realm.json`
4. Click "Create"

## Environment Configuration

Update your console service `.env.local`:

```bash
KEYCLOAK_AUTH_SERVER_URL=http://localhost:8080
KEYCLOAK_REALM=bitsaccoserver-dev
KEYCLOAK_CLIENT_ID=bitsaccoserver
KEYCLOAK_CLIENT_SECRET=bitsaccoserver-secret-dev
```

## Testing Authentication

Test the authentication with curl:

```bash
# Get access token
TOKEN=$(curl -s -X POST http://localhost:8080/realms/bitsaccoserver-dev/protocol/openid-connect/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=admin@bitsaccoserver.org" \
  -d "password=admin123" \
  -d "grant_type=password" \
  -d "client_id=bitsaccoserver" \
  -d "client_secret=bitsaccoserver-secret-dev" | jq -r '.access_token')

# Test console API
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/api/v1/profile
```

## Troubleshooting

- **Import fails**: Check Keycloak is running and accessible
- **401 errors**: Verify client secret and realm name
- **CORS issues**: Check console service CORS configuration
- **Token validation fails**: Ensure JWT secret matches between services
