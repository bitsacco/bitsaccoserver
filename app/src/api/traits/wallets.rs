use async_trait::async_trait;
use uuid::Uuid;

use crate::api::{
    errors::ApiResult,
    types::{PaginatedResponse, PaginationQuery},
};

// Placeholder wallet types - will be expanded based on actual requirements
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub balance: u64, // Satoshis
    pub wallet_type: WalletType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WalletType {
    #[serde(rename = "fedimint")]
    Fedimint,
    #[serde(rename = "lightning")]
    Lightning,
    #[serde(rename = "onchain")]
    OnChain,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateWalletRequest {
    pub user_id: Uuid,
    pub name: String,
    pub wallet_type: WalletType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalletTransaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub amount: i64, // Satoshis (negative for outgoing)
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "transfer")]
    Transfer,
    #[serde(rename = "payment")]
    Payment,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TransactionStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "confirmed")]
    Confirmed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

#[async_trait]
pub trait WalletsApi: Send + Sync {
    /// Get a wallet by ID
    async fn get_wallet(&self, wallet_id: Uuid) -> ApiResult<Wallet>;

    /// Get all wallets for a user
    async fn get_user_wallets(&self, user_id: Uuid) -> ApiResult<Vec<Wallet>>;

    /// Get all wallets with pagination
    async fn get_wallets(
        &self,
        pagination: PaginationQuery,
    ) -> ApiResult<PaginatedResponse<Wallet>>;

    /// Create a new wallet
    async fn create_wallet(&self, request: CreateWalletRequest) -> ApiResult<Wallet>;

    /// Delete a wallet
    async fn delete_wallet(&self, wallet_id: Uuid) -> ApiResult<()>;

    /// Get wallet transactions
    async fn get_wallet_transactions(
        &self,
        wallet_id: Uuid,
        pagination: PaginationQuery,
    ) -> ApiResult<PaginatedResponse<WalletTransaction>>;

    /// Get wallet balance
    async fn get_wallet_balance(&self, wallet_id: Uuid) -> ApiResult<u64>;
}
