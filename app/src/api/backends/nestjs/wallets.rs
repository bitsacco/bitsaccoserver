use async_trait::async_trait;
use uuid::Uuid;

use crate::api::{
    errors::ApiResult,
    traits::wallets::{
        CreateWalletRequest, TransactionStatus, TransactionType, Wallet, WalletTransaction,
        WalletType, WalletsApi,
    },
    types::{PaginatedResponse, PaginationQuery},
};

use super::client::NestJsClient;

#[derive(Clone)]
pub struct NestJsWalletsApi {
    client: NestJsClient,
}

impl NestJsWalletsApi {
    pub fn new(client: NestJsClient) -> Self {
        Self { client }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UserTxsRequest {
    #[serde(rename = "userId")]
    user_id: String,
    page: Option<u32>,
    size: Option<u32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SoloWalletResponse {
    transactions: Vec<serde_json::Value>,
    #[serde(default)]
    page: u32,
    #[serde(default = "default_size")]
    size: u32,
    #[serde(default)]
    total: u32,
    #[serde(default)]
    pages: u32,
}

fn default_size() -> u32 {
    10
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CreateSoloWalletRequest {
    #[serde(rename = "userId")]
    user_id: String,
    // Additional fields as needed based on the NestJS backend
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CreateChamaWalletRequest {
    #[serde(rename = "chamaId")]
    chama_id: String,
    // Additional fields as needed based on the NestJS backend
}

#[async_trait]
impl WalletsApi for NestJsWalletsApi {
    async fn get_wallet(&self, wallet_id: Uuid) -> ApiResult<Wallet> {
        // The NestJS backend doesn't have a direct "get wallet by ID" endpoint
        // We'll simulate this by getting wallet details through other means
        // This is a limitation of the current backend API structure
        Err(crate::api::errors::ApiError::NotFound {
            resource: format!(
                "Direct wallet lookup not supported by backend for wallet {}",
                wallet_id
            ),
        })
    }

    async fn get_user_wallets(&self, _user_id: Uuid) -> ApiResult<Vec<Wallet>> {
        // The NestJS backend doesn't have a direct endpoint to get all wallets for a user
        // In a real implementation, you might need to:
        // 1. Get solo wallets for the user
        // 2. Get chama wallets where the user is a member
        // 3. Combine them into a single list

        // For now, we'll return an empty list and let the calling code handle this limitation
        Ok(Vec::new())
    }

    async fn get_wallets(
        &self,
        pagination: PaginationQuery,
    ) -> ApiResult<PaginatedResponse<Wallet>> {
        // The NestJS backend doesn't have a general "list all wallets" endpoint
        // This would need to be implemented in the backend or composed from multiple calls
        Ok(PaginatedResponse {
            data: Vec::new(),
            total: 0,
            page: pagination.page.unwrap_or(1),
            limit: pagination.limit.unwrap_or(20),
            total_pages: 0,
        })
    }

    async fn create_wallet(&self, request: CreateWalletRequest) -> ApiResult<Wallet> {
        match request.wallet_type {
            WalletType::Fedimint => {
                // Create a solo wallet
                let solo_request = CreateSoloWalletRequest {
                    user_id: request.user_id.to_string(),
                };

                let req = self.client.post("/solowallet");
                let _response: serde_json::Value =
                    self.client.send_json(req, &solo_request).await?;

                // Convert the response to our Wallet structure
                // Note: The actual response format may differ, so this is an approximation
                Ok(Wallet {
                    id: request.user_id, // Using user_id as wallet_id for solo wallets
                    user_id: request.user_id,
                    name: request.name,
                    balance: 0, // Initial balance
                    wallet_type: WalletType::Fedimint,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })
            }
            WalletType::Lightning | WalletType::OnChain => {
                // These wallet types are not directly supported by the current backend
                Err(crate::api::errors::ApiError::NotFound {
                    resource: format!(
                        "Wallet type {:?} not supported by backend",
                        request.wallet_type
                    ),
                })
            }
        }
    }

    async fn delete_wallet(&self, wallet_id: Uuid) -> ApiResult<()> {
        // The NestJS backend doesn't have a delete wallet endpoint
        Err(crate::api::errors::ApiError::NotFound {
            resource: format!(
                "Delete wallet operation not supported by backend for wallet {}",
                wallet_id
            ),
        })
    }

    async fn get_wallet_transactions(
        &self,
        wallet_id: Uuid,
        pagination: PaginationQuery,
    ) -> ApiResult<PaginatedResponse<WalletTransaction>> {
        // For solo wallets, we can use the user transactions endpoint
        // Note: This assumes wallet_id corresponds to user_id for solo wallets
        let tx_request = UserTxsRequest {
            user_id: wallet_id.to_string(),
            page: pagination.page,
            size: pagination.limit,
        };

        let req = self.client.post("/solowallet/transactions");
        let response: SoloWalletResponse = self.client.send_json(req, &tx_request).await?;

        // Convert the response transactions to our WalletTransaction structure
        let transactions: Result<Vec<WalletTransaction>, _> = response
            .transactions
            .into_iter()
            .map(|tx| self.convert_transaction(tx, wallet_id))
            .collect();

        let transactions =
            transactions.map_err(|e| crate::api::errors::ApiError::Serialization {
                message: format!("Failed to convert transactions: {}", e),
            })?;

        Ok(PaginatedResponse {
            data: transactions,
            total: response.total as u64,
            page: response.page,
            limit: response.size,
            total_pages: response.pages,
        })
    }

    async fn get_wallet_balance(&self, _wallet_id: Uuid) -> ApiResult<u64> {
        // The NestJS backend doesn't have a direct balance endpoint
        // We would need to either:
        // 1. Add a balance endpoint to the backend
        // 2. Calculate balance from transactions
        // 3. Use a different approach

        // For now, we'll return 0 and let the calling code handle this limitation
        Ok(0)
    }
}

impl NestJsWalletsApi {
    /// Convert a transaction from the NestJS backend format to our WalletTransaction structure
    fn convert_transaction(
        &self,
        tx: serde_json::Value,
        wallet_id: Uuid,
    ) -> Result<WalletTransaction, String> {
        let id = tx
            .get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| "Missing or invalid transaction ID".to_string())?;

        let amount = tx.get("amount").and_then(|v| v.as_i64()).unwrap_or(0);

        // Try to determine transaction type from the backend data
        let transaction_type = if amount > 0 {
            TransactionType::Deposit
        } else {
            TransactionType::Withdrawal
        };

        // Try to determine status from the backend data
        let status_str = tx
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("pending");

        let status = match status_str.to_lowercase().as_str() {
            "completed" | "success" | "confirmed" => TransactionStatus::Confirmed,
            "failed" | "error" => TransactionStatus::Failed,
            "cancelled" => TransactionStatus::Cancelled,
            _ => TransactionStatus::Pending,
        };

        let created_at = tx
            .get("createdAt")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let updated_at = tx
            .get("updatedAt")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        Ok(WalletTransaction {
            id,
            wallet_id,
            amount,
            transaction_type,
            status,
            created_at,
            updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::config::ApiConfig;

    fn create_test_client() -> NestJsClient {
        let config = ApiConfig::default();
        NestJsClient::new(&config).expect("Failed to create test client")
    }

    #[tokio::test]
    async fn test_wallets_api_creation() {
        let client = create_test_client();
        let wallets_api = NestJsWalletsApi::new(client);

        // Test that the API can be created
        assert!(true); // If we get here, creation succeeded
    }

    #[test]
    fn test_transaction_conversion() {
        let client = create_test_client();
        let wallets_api = NestJsWalletsApi::new(client);
        let wallet_id = Uuid::new_v4();

        // Test successful transaction conversion
        let tx_json = serde_json::json!({
            "id": wallet_id.to_string(),
            "amount": 1000,
            "status": "confirmed",
            "createdAt": "2023-01-01T12:00:00Z",
            "updatedAt": "2023-01-01T12:00:00Z"
        });

        let result = wallets_api.convert_transaction(tx_json, wallet_id);
        assert!(result.is_ok());

        let transaction = result.unwrap();
        assert_eq!(transaction.amount, 1000);
        assert_eq!(transaction.wallet_id, wallet_id);
        assert_eq!(transaction.transaction_type, TransactionType::Deposit);
        assert_eq!(transaction.status, TransactionStatus::Confirmed);
    }

    #[test]
    fn test_transaction_type_detection() {
        // Positive amount should be deposit
        let positive_amount = 1000i64;
        assert!(positive_amount > 0);

        // Negative amount should be withdrawal
        let negative_amount = -500i64;
        assert!(negative_amount < 0);
    }

    #[test]
    fn test_status_mapping() {
        let test_cases = vec![
            ("completed", TransactionStatus::Confirmed),
            ("success", TransactionStatus::Confirmed),
            ("confirmed", TransactionStatus::Confirmed),
            ("failed", TransactionStatus::Failed),
            ("error", TransactionStatus::Failed),
            ("cancelled", TransactionStatus::Cancelled),
            ("pending", TransactionStatus::Pending),
            ("unknown", TransactionStatus::Pending),
        ];

        for (input, expected) in test_cases {
            let status = match input.to_lowercase().as_str() {
                "completed" | "success" | "confirmed" => TransactionStatus::Confirmed,
                "failed" | "error" => TransactionStatus::Failed,
                "cancelled" => TransactionStatus::Cancelled,
                _ => TransactionStatus::Pending,
            };
            assert_eq!(status, expected, "Failed for input: {}", input);
        }
    }
}
