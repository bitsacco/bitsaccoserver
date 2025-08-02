# Shares API Compatibility Layer

This document describes the shares compatibility layer implemented to ensure 100% backward compatibility with existing NestJS clients during the migration to Rust/Leptos.

## Overview

The shares compatibility layer provides NestJS-compatible endpoints at `/shares/*` (without the `/api` prefix) that map to our internal Rust implementation. This allows existing clients to continue working without changes during the migration period.

## Endpoints Implemented

### Share Offers

#### `POST /shares/offer`
Creates a new share offer with NestJS-compatible request format.

**Request Body:**
```json
{
  "name": "Share Offer Name",
  "description": "Optional description",
  "pricePerShare": "1000.00",
  "totalSharesAvailable": "100.00", 
  "validFrom": "2024-01-01T00:00:00Z",
  "validUntil": "2024-12-31T23:59:59Z",
  "minPurchaseQuantity": "1.00",
  "maxPurchaseQuantity": "50.00",
  "settings": {...},
  "metadata": {...},
  "createdBy": "uuid-string"
}
```

**Response (201):**
```json
{
  "id": "uuid-string",
  "name": "Share Offer Name",
  "description": "Optional description",
  "pricePerShare": "1000.00",
  "totalSharesAvailable": "100.00",
  "sharesSold": "0.00",
  "sharesRemaining": "100.00",
  "status": "draft",
  "validFrom": "2024-01-01T00:00:00Z",
  "validUntil": "2024-12-31T23:59:59Z",
  "minPurchaseQuantity": "1.00",
  "maxPurchaseQuantity": "50.00",
  "settings": {...},
  "metadata": {...},
  "createdAt": "2024-01-01T12:00:00Z",
  "updatedAt": "2024-01-01T12:00:00Z",
  "createdBy": "uuid-string",
  "updatedBy": "uuid-string"
}
```

#### `GET /shares/offers`
Lists all share offers with summary statistics.

**Response (200):**
```json
{
  "offers": [...],
  "total": 23,
  "active": 15
}
```

### Share Subscription (Purchase)

#### `POST /shares/subscribe`
Subscribes to shares (maps to internal purchase logic).

**Request Body:**
```json
{
  "shareOfferId": "uuid-string",
  "ownerId": "uuid-string", 
  "ownerType": "member",
  "quantity": "5.00",
  "purchasedBy": "uuid-string"
}
```

**Response (200):**
```json
{
  "transactions": [
    {
      "id": "uuid-string",
      "sharesId": "uuid-string",
      "ownerId": "uuid-string",
      "ownerType": "member",
      "quantity": "5.00",
      "totalValue": "5000.00",
      "transactionType": "purchase",
      "createdAt": "2024-01-01T12:00:00Z",
      "createdBy": "uuid-string"
    }
  ],
  "pagination": {
    "page": 0,
    "size": 1,
    "total": 1
  },
  "summary": {
    "totalShares": "5.00",
    "totalValue": "5000.00"
  }
}
```

### Share Management

#### `POST /shares/transfer`
Transfers shares between owners.

**Request Body:**
```json
{
  "shareId": "uuid-string",
  "newOwnerId": "uuid-string",
  "newOwnerType": "member",
  "quantityToTransfer": "2.00",
  "transferredBy": "uuid-string",
  "reason": "Transfer reason"
}
```

#### `POST /shares/update`
Updates share records (basic implementation).

**Request Body:**
```json
{
  "shareId": "uuid-string",
  "quantity": "10.00",
  "totalValue": "10000.00",
  "updatedBy": "uuid-string",
  "reason": "Update reason"
}
```

### Transaction History

#### `GET /shares/transactions`
Lists all share transactions with pagination.

**Query Parameters:**
- `page` (optional): Page number (default: 0)
- `size` (optional): Page size (default: 100)

#### `GET /shares/transactions/:userId`
Gets transactions for a specific user.

**Query Parameters:**
- `page` (optional): Page number (default: 0)
- `size` (optional): Page size (default: 100)

#### `GET /shares/transactions/find/:sharesId`
Finds a specific transaction by shares ID.

## Data Format Compatibility

### Key Differences from Internal API

1. **Field Naming**: Uses camelCase (e.g., `shareOfferId`) instead of snake_case
2. **Data Types**: All UUIDs and decimals are represented as strings for exact compatibility
3. **Response Structure**: Matches NestJS response patterns exactly
4. **Pagination**: Uses `page`/`size` parameters instead of `limit`/`offset`

### Field Mapping

| NestJS Field | Internal Field | Notes |
|-------------|----------------|-------|
| `quantity` | `share_quantity` | Share amount |
| `sharesId` | `id` | Transaction ID mapping |
| `purchasedBy` | `created_by` | User who created the record |
| `pricePerShare` | `price_per_share` | Price per individual share |

## Error Handling

All endpoints return NestJS-compatible error responses:

```json
{
  "error": "error_type",
  "message": "Human-readable error message"
}
```

Common error types:
- `invalid_uuid`: Invalid UUID format
- `invalid_decimal`: Invalid decimal format  
- `invalid_owner_type`: Invalid owner type (must be 'member' or 'group')
- `creation_failed`: Failed to create resource
- `fetch_failed`: Failed to retrieve resource
- `not_found`: Resource not found

## Testing

A Python test script is provided at `test_shares_compat.py` to verify basic endpoint functionality:

```bash
python3 test_shares_compat.py
```

## Implementation Notes

1. **Compatibility Layer**: This is a translation layer that maintains the existing API contract while using the new Rust implementation internally.

2. **Semantic Mapping**: The `/shares/subscribe` endpoint maps NestJS "subscribe" semantics to our internal "purchase" logic transparently.

3. **Transaction Model**: Since our current model uses share records directly, transaction history endpoints map from share records rather than separate transaction tables.

4. **Status Mapping**: Share offer statuses are converted from Rust enums to lowercase strings for compatibility.

5. **Date Formatting**: All timestamps are formatted in ISO 8601 format for consistency.

## Future Considerations

1. **Deprecation Path**: Once migration is complete, these compatibility endpoints can be deprecated in favor of the new `/api/shares/*` endpoints.

2. **Performance**: Consider adding caching layers for frequently accessed data like share offers and transaction summaries.

3. **Audit Trail**: The compatibility layer preserves audit information (created_by, updated_by) for compliance.

4. **Validation**: All input validation matches NestJS patterns to ensure consistent behavior.

## Migration Timeline

- **Phase 1**: Compatibility layer (current) - 100% backward compatibility
- **Phase 2**: Client migration - Gradual migration of clients to new API
- **Phase 3**: Deprecation - Remove compatibility endpoints after full migration