use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ::entity::{
    fedimint_operations,
    sea_orm_active_enums::{
        FedimintOperationType, ReserveType, TransactionStatus, TransactionType, WalletStatus,
    },
    wallet_transactions, wallets,
};
use sea_orm::Set;

use super::fedimint::{
    FedimintClientService, FedimintError, FedimintOperation, FedimintOperationResult,
};
use crate::repositories::{Repositories, RepositoryError};

/// Error types for wallet operations
#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Fedimint error: {0}")]
    Fedimint(#[from] FedimintError),
    #[error("Wallet not found: {0}")]
    WalletNotFound(Uuid),
    #[error("Insufficient balance: required {required} msat, available {available} msat")]
    InsufficientBalance { required: i64, available: i64 },
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Wallet is {0}, operation not allowed")]
    WalletNotActive(String),
    #[error("Transaction not found: {0}")]
    TransactionNotFound(Uuid),
    #[error("Operation not allowed: {0}")]
    OperationNotAllowed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("External service error: {0}")]
    ExternalService(String),
}

pub type WalletResult<T> = Result<T, WalletError>;

/// Request to create a new wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub owner_id: Uuid,
    pub owner_type: String,
    pub name: String,
    pub description: Option<String>,
    pub federation_id: String,
}

/// Request to deposit funds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositRequest {
    pub wallet_id: Uuid,
    pub amount_sats: Option<u64>, // None for address generation only
    pub description: Option<String>,
    pub owner_id: Uuid,
}

/// Request to withdraw funds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawRequest {
    pub wallet_id: Uuid,
    pub amount_sats: u64,
    pub address: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
}

/// Request to generate Lightning invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceRequest {
    pub wallet_id: Uuid,
    pub amount_msats: u64,
    pub description: String,
    pub expiry_secs: Option<u64>,
    pub owner_id: Uuid,
}

/// Request to pay Lightning invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayInvoiceRequest {
    pub wallet_id: Uuid,
    pub invoice: String,
    pub max_amount_msats: Option<u64>,
    pub owner_id: Uuid,
}

/// Wallet balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub wallet_id: Uuid,
    pub total_balance_msat: i64,
    pub available_balance_msat: i64,
    pub pending_in_msat: i64,
    pub pending_out_msat: i64,
    pub reserved_available_msat: i64,
    pub reserved_pending_msat: i64,
    pub reserved_locked_msat: i64,
    pub reserved_emergency_msat: i64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Deposit response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositResponse {
    pub transaction_id: Uuid,
    pub fedimint_operation_id: Uuid,
    pub deposit_address: String,
    pub amount_sats: Option<u64>,
}

/// Withdrawal response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawResponse {
    pub transaction_id: Uuid,
    pub fedimint_operation_id: Uuid,
    pub amount_sats: u64,
    pub address: String,
    pub estimated_fee_sats: Option<u64>,
}

/// Invoice generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceResponse {
    pub transaction_id: Uuid,
    pub fedimint_operation_id: Uuid,
    pub invoice: String,
    pub payment_hash: String,
    pub amount_msats: u64,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Payment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub transaction_id: Uuid,
    pub fedimint_operation_id: Uuid,
    pub payment_hash: String,
    pub amount_msats: u64,
    pub preimage: Option<String>,
}

/// Main wallet service
#[derive(Clone)]
pub struct WalletService {
    repositories: Repositories,
    fedimint: FedimintClientService,
}

impl WalletService {
    pub fn new(repositories: Repositories, fedimint: FedimintClientService) -> Self {
        Self {
            repositories,
            fedimint,
        }
    }

    /// Create a new wallet for a user
    pub async fn create_wallet(
        &self,
        request: CreateWalletRequest,
    ) -> WalletResult<wallets::Model> {
        // Check if user already has an active wallet for this federation
        let existing_wallets = self
            .repositories
            .wallets
            .find_by_owner_and_federation(request.owner_id, &request.federation_id)
            .await?;

        for wallet in existing_wallets {
            if wallet.status == WalletStatus::Active {
                return Err(WalletError::OperationNotAllowed(
                    "User already has an active wallet for this federation".to_string(),
                ));
            }
        }

        let wallet = wallets::ActiveModel {
            id: Set(Uuid::new_v4()),
            owner_id: Set(request.owner_id),
            owner_type: Set(request.owner_type),
            name: Set(request.name),
            description: Set(request.description.clone()),
            status: Set(WalletStatus::Active),
            balance_msat: Set(0),
            pending_in_msat: Set(0),
            pending_out_msat: Set(0),
            federation_id: Set(Some(request.federation_id)),
            client_config: Set(None),
            metadata: Set(None),
            last_sync_at: Set(None),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(Some(request.owner_id)),
            updated_by: Set(Some(request.owner_id)),
        };

        let created_wallet = self.repositories.wallets.create(wallet).await?;

        // Initialize available reserves for the new wallet
        self.repositories
            .wallet_reserves
            .upsert_reserve(
                created_wallet.id,
                ReserveType::Available,
                0,
                Some("initial_balance".to_string()),
                None,
                Some(request.owner_id),
            )
            .await?;

        Ok(created_wallet)
    }

    /// Get wallet balance with reserves breakdown
    pub async fn get_balance(
        &self,
        wallet_id: Uuid,
        owner_id: Uuid,
    ) -> WalletResult<WalletBalance> {
        let wallet = self.get_wallet_for_owner(wallet_id, owner_id).await?;
        let reserve_summary = self
            .repositories
            .wallet_reserves
            .get_wallet_summary(wallet_id)
            .await?;

        Ok(WalletBalance {
            wallet_id,
            total_balance_msat: wallet.balance_msat,
            available_balance_msat: wallet.balance_msat - wallet.pending_out_msat,
            pending_in_msat: wallet.pending_in_msat,
            pending_out_msat: wallet.pending_out_msat,
            reserved_available_msat: reserve_summary.available_msat,
            reserved_pending_msat: reserve_summary.pending_msat,
            reserved_locked_msat: reserve_summary.locked_msat,
            reserved_emergency_msat: reserve_summary.emergency_msat,
            last_updated: wallet.updated_at.into(),
        })
    }

    /// Initiate a Bitcoin deposit
    pub async fn deposit(&self, request: DepositRequest) -> WalletResult<DepositResponse> {
        let wallet = self
            .get_active_wallet_for_owner(request.wallet_id, request.owner_id)
            .await?;

        // Create transaction record
        let transaction = wallet_transactions::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_id: Set(wallet.id),
            transaction_type: Set(TransactionType::Deposit),
            status: Set(TransactionStatus::Pending),
            amount_msat: Set(request.amount_sats.map(|s| s as i64 * 1000).unwrap_or(0)),
            fee_msat: Set(0),
            description: Set(request.description.clone()),
            external_id: Set(None),
            counterparty_id: Set(None),
            fedimint_operation_id: Set(None),
            metadata: Set(None),
            processed_at: Set(None),
            expires_at: Set(None),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(Some(request.owner_id)),
            updated_by: Set(Some(request.owner_id)),
        };

        let created_transaction = self
            .repositories
            .wallet_transactions
            .create(transaction)
            .await?;

        // Create Fedimint operation
        let fedimint_operation = fedimint_operations::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_id: Set(wallet.id),
            operation_type: Set(FedimintOperationType::Deposit),
            status: Set(TransactionStatus::Pending),
            fedimint_operation_id: Set(Some(format!("deposit_{}", created_transaction.id))),
            amount_msat: Set(request.amount_sats.map(|s| s as i64 * 1000)),
            fee_msat: Set(None),
            request: Set(serde_json::to_value(&request).ok()),
            response: Set(None),
            error_details: Set(None),
            retry_count: Set(0),
            last_retry_at: Set(None),
            processed_at: Set(None),
            expires_at: Set(Some(
                (chrono::Utc::now() + chrono::Duration::hours(24)).into(),
            )),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(Some(request.owner_id)),
            updated_by: Set(Some(request.owner_id)),
        };

        let fedimint_op = self
            .repositories
            .fedimint_operations
            .create(fedimint_operation)
            .await?;

        // Update transaction with Fedimint operation ID
        let mut transaction: wallet_transactions::ActiveModel = created_transaction.into();
        transaction.fedimint_operation_id = Set(Some(fedimint_op.id));
        let updated_transaction = self
            .repositories
            .wallet_transactions
            .update(transaction)
            .await?;

        // Get deposit address from Fedimint
        let deposit_operation = FedimintOperation::Deposit {
            amount_sats: request.amount_sats.unwrap_or(0),
            address: None,
        };

        let fedimint_result = self
            .fedimint
            .execute_operation(
                wallet
                    .federation_id
                    .as_ref()
                    .unwrap_or(&"default".to_string()),
                deposit_operation,
            )
            .await?;

        let deposit_address = match &fedimint_result {
            FedimintOperationResult::DepositAddress { address, .. } => address.clone(),
            _ => {
                return Err(WalletError::ExternalService(
                    "Unexpected response from Fedimint for deposit".to_string(),
                ))
            }
        };

        // Update Fedimint operation with response
        self.repositories
            .fedimint_operations
            .update_response(
                fedimint_op.id,
                serde_json::to_value(&fedimint_result).unwrap(),
            )
            .await?;

        Ok(DepositResponse {
            transaction_id: updated_transaction.id,
            fedimint_operation_id: fedimint_op.id,
            deposit_address,
            amount_sats: request.amount_sats,
        })
    }

    /// Initiate a Bitcoin withdrawal
    pub async fn withdraw(&self, request: WithdrawRequest) -> WalletResult<WithdrawResponse> {
        let wallet = self
            .get_active_wallet_for_owner(request.wallet_id, request.owner_id)
            .await?;
        let amount_msat = request.amount_sats as i64 * 1000;

        // Check if sufficient balance is available
        let balance = self
            .get_balance(request.wallet_id, request.owner_id)
            .await?;
        if balance.available_balance_msat < amount_msat {
            return Err(WalletError::InsufficientBalance {
                required: amount_msat,
                available: balance.available_balance_msat,
            });
        }

        // Create transaction record
        let transaction = wallet_transactions::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_id: Set(wallet.id),
            transaction_type: Set(TransactionType::Withdraw),
            status: Set(TransactionStatus::Pending),
            amount_msat: Set(amount_msat),
            fee_msat: Set(0), // Will be updated when withdrawal is processed
            description: Set(request.description.clone()),
            external_id: Set(None),
            counterparty_id: Set(None),
            fedimint_operation_id: Set(None),
            metadata: Set(None),
            processed_at: Set(None),
            expires_at: Set(None),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(Some(request.owner_id)),
            updated_by: Set(Some(request.owner_id)),
        };

        let created_transaction = self
            .repositories
            .wallet_transactions
            .create(transaction)
            .await?;

        // Reserve the funds by transferring from available to locked
        self.repositories
            .wallet_reserves
            .transfer_reserves(
                wallet.id,
                ReserveType::Available,
                ReserveType::Locked,
                amount_msat,
                Some(format!("withdraw_{}", created_transaction.id)),
                Some(request.owner_id),
            )
            .await?;

        // Update wallet pending out amount
        self.repositories
            .wallets
            .update_balance(
                wallet.id,
                wallet.balance_msat,
                wallet.pending_in_msat,
                wallet.pending_out_msat + amount_msat,
            )
            .await?;

        // Create Fedimint operation
        let fedimint_operation = fedimint_operations::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_id: Set(wallet.id),
            operation_type: Set(FedimintOperationType::Withdraw),
            status: Set(TransactionStatus::Pending),
            fedimint_operation_id: Set(Some(format!("withdraw_{}", created_transaction.id))),
            amount_msat: Set(Some(amount_msat)),
            fee_msat: Set(None),
            request: Set(serde_json::to_value(&request).ok()),
            response: Set(None),
            error_details: Set(None),
            retry_count: Set(0),
            last_retry_at: Set(None),
            processed_at: Set(None),
            expires_at: Set(Some(
                (chrono::Utc::now() + chrono::Duration::hours(1)).into(),
            )),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(Some(request.owner_id)),
            updated_by: Set(Some(request.owner_id)),
        };

        let fedimint_op = self
            .repositories
            .fedimint_operations
            .create(fedimint_operation)
            .await?;

        // Update transaction with Fedimint operation ID
        let mut transaction: wallet_transactions::ActiveModel = created_transaction.into();
        transaction.fedimint_operation_id = Set(Some(fedimint_op.id));
        let updated_transaction = self
            .repositories
            .wallet_transactions
            .update(transaction)
            .await?;

        // Initiate withdrawal with Fedimint
        let withdraw_operation = FedimintOperation::Withdraw {
            amount_sats: request.amount_sats,
            address: request.address.clone(),
        };

        let fedimint_result = self
            .fedimint
            .execute_operation(
                wallet
                    .federation_id
                    .as_ref()
                    .unwrap_or(&"default".to_string()),
                withdraw_operation,
            )
            .await?;

        // Update Fedimint operation with response
        self.repositories
            .fedimint_operations
            .update_response(
                fedimint_op.id,
                serde_json::to_value(&fedimint_result).unwrap(),
            )
            .await?;

        Ok(WithdrawResponse {
            transaction_id: updated_transaction.id,
            fedimint_operation_id: fedimint_op.id,
            amount_sats: request.amount_sats,
            address: request.address,
            estimated_fee_sats: None, // TODO: Estimate fees
        })
    }

    /// Generate a Lightning invoice
    pub async fn generate_invoice(&self, request: InvoiceRequest) -> WalletResult<InvoiceResponse> {
        let wallet = self
            .get_active_wallet_for_owner(request.wallet_id, request.owner_id)
            .await?;

        if request.amount_msats == 0 {
            return Err(WalletError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Create transaction record
        let transaction = wallet_transactions::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_id: Set(wallet.id),
            transaction_type: Set(TransactionType::Deposit),
            status: Set(TransactionStatus::Pending),
            amount_msat: Set(request.amount_msats as i64),
            fee_msat: Set(0),
            description: Set(Some(request.description.clone())),
            external_id: Set(None),
            counterparty_id: Set(None),
            fedimint_operation_id: Set(None),
            metadata: Set(None),
            processed_at: Set(None),
            expires_at: Set(None),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(Some(request.owner_id)),
            updated_by: Set(Some(request.owner_id)),
        };

        let created_transaction = self
            .repositories
            .wallet_transactions
            .create(transaction)
            .await?;

        // Create Fedimint operation
        let fedimint_operation = fedimint_operations::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_id: Set(wallet.id),
            operation_type: Set(FedimintOperationType::Lightning),
            status: Set(TransactionStatus::Pending),
            fedimint_operation_id: Set(Some(format!("invoice_{}", created_transaction.id))),
            amount_msat: Set(Some(request.amount_msats as i64)),
            fee_msat: Set(None),
            request: Set(serde_json::to_value(&request).ok()),
            response: Set(None),
            error_details: Set(None),
            retry_count: Set(0),
            last_retry_at: Set(None),
            processed_at: Set(None),
            expires_at: Set(Some(
                (chrono::Utc::now()
                    + chrono::Duration::seconds(request.expiry_secs.unwrap_or(3600) as i64))
                .into(),
            )),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(Some(request.owner_id)),
            updated_by: Set(Some(request.owner_id)),
        };

        let fedimint_op = self
            .repositories
            .fedimint_operations
            .create(fedimint_operation)
            .await?;

        // Update transaction with Fedimint operation ID
        let mut transaction: wallet_transactions::ActiveModel = created_transaction.into();
        transaction.fedimint_operation_id = Set(Some(fedimint_op.id));
        let updated_transaction = self
            .repositories
            .wallet_transactions
            .update(transaction)
            .await?;

        // Generate invoice with Fedimint
        let invoice_operation = FedimintOperation::GenerateInvoice {
            amount_msats: request.amount_msats,
            description: request.description.clone(),
            expiry_secs: request.expiry_secs,
        };

        let fedimint_result = self
            .fedimint
            .execute_operation(
                wallet
                    .federation_id
                    .as_ref()
                    .unwrap_or(&"default".to_string()),
                invoice_operation,
            )
            .await?;

        let (invoice, payment_hash) = match &fedimint_result {
            FedimintOperationResult::InvoiceGenerated {
                invoice,
                payment_hash,
                ..
            } => (invoice.clone(), payment_hash.clone()),
            _ => {
                return Err(WalletError::ExternalService(
                    "Unexpected response from Fedimint for invoice generation".to_string(),
                ))
            }
        };

        // Update transaction with payment hash as external ID
        let mut transaction: wallet_transactions::ActiveModel = updated_transaction.into();
        transaction.external_id = Set(Some(payment_hash.clone()));
        let final_transaction = self
            .repositories
            .wallet_transactions
            .update(transaction)
            .await?;

        // Update Fedimint operation with response
        self.repositories
            .fedimint_operations
            .update_response(
                fedimint_op.id,
                serde_json::to_value(&fedimint_result).unwrap(),
            )
            .await?;

        Ok(InvoiceResponse {
            transaction_id: final_transaction.id,
            fedimint_operation_id: fedimint_op.id,
            invoice,
            payment_hash,
            amount_msats: request.amount_msats,
            expires_at: chrono::Utc::now()
                + chrono::Duration::seconds(request.expiry_secs.unwrap_or(3600) as i64),
        })
    }

    /// Pay a Lightning invoice
    pub async fn pay_invoice(&self, request: PayInvoiceRequest) -> WalletResult<PaymentResponse> {
        let wallet = self
            .get_active_wallet_for_owner(request.wallet_id, request.owner_id)
            .await?;

        // TODO: Parse invoice to get amount if max_amount_msats is not provided
        let amount_msat = request.max_amount_msats.unwrap_or(0) as i64;

        if amount_msat > 0 {
            // Check if sufficient balance is available
            let balance = self
                .get_balance(request.wallet_id, request.owner_id)
                .await?;
            if balance.available_balance_msat < amount_msat {
                return Err(WalletError::InsufficientBalance {
                    required: amount_msat,
                    available: balance.available_balance_msat,
                });
            }
        }

        // Create transaction record
        let transaction = wallet_transactions::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_id: Set(wallet.id),
            transaction_type: Set(TransactionType::Payment),
            status: Set(TransactionStatus::Pending),
            amount_msat: Set(amount_msat),
            fee_msat: Set(0),
            description: Set(Some("Lightning payment".to_string())),
            external_id: Set(None),
            counterparty_id: Set(None),
            fedimint_operation_id: Set(None),
            metadata: Set(None),
            processed_at: Set(None),
            expires_at: Set(None),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(Some(request.owner_id)),
            updated_by: Set(Some(request.owner_id)),
        };

        let created_transaction = self
            .repositories
            .wallet_transactions
            .create(transaction)
            .await?;

        // Reserve funds if amount is known
        if amount_msat > 0 {
            self.repositories
                .wallet_reserves
                .transfer_reserves(
                    wallet.id,
                    ReserveType::Available,
                    ReserveType::Locked,
                    amount_msat,
                    Some(format!("payment_{}", created_transaction.id)),
                    Some(request.owner_id),
                )
                .await?;

            // Update wallet pending out amount
            self.repositories
                .wallets
                .update_balance(
                    wallet.id,
                    wallet.balance_msat,
                    wallet.pending_in_msat,
                    wallet.pending_out_msat + amount_msat,
                )
                .await?;
        }

        // Create Fedimint operation
        let fedimint_operation = fedimint_operations::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_id: Set(wallet.id),
            operation_type: Set(FedimintOperationType::Lightning),
            status: Set(TransactionStatus::Pending),
            fedimint_operation_id: Set(Some(format!("payment_{}", created_transaction.id))),
            amount_msat: Set(if amount_msat > 0 {
                Some(amount_msat)
            } else {
                None
            }),
            fee_msat: Set(None),
            request: Set(serde_json::to_value(&request).ok()),
            response: Set(None),
            error_details: Set(None),
            retry_count: Set(0),
            last_retry_at: Set(None),
            processed_at: Set(None),
            expires_at: Set(Some(
                (chrono::Utc::now() + chrono::Duration::minutes(5)).into(),
            )),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(Some(request.owner_id)),
            updated_by: Set(Some(request.owner_id)),
        };

        let fedimint_op = self
            .repositories
            .fedimint_operations
            .create(fedimint_operation)
            .await?;

        // Update transaction with Fedimint operation ID
        let mut transaction: wallet_transactions::ActiveModel = created_transaction.into();
        transaction.fedimint_operation_id = Set(Some(fedimint_op.id));
        let updated_transaction = self
            .repositories
            .wallet_transactions
            .update(transaction)
            .await?;

        // Pay invoice with Fedimint
        let payment_operation = FedimintOperation::PayInvoice {
            invoice: request.invoice,
            max_amount_msats: request.max_amount_msats,
        };

        let fedimint_result = self
            .fedimint
            .execute_operation(
                wallet
                    .federation_id
                    .as_ref()
                    .unwrap_or(&"default".to_string()),
                payment_operation,
            )
            .await?;

        let (payment_hash, _operation_id) = match &fedimint_result {
            FedimintOperationResult::PaymentSent {
                payment_hash,
                operation_id,
                ..
            } => (payment_hash.clone(), operation_id.clone()),
            _ => {
                return Err(WalletError::ExternalService(
                    "Unexpected response from Fedimint for payment".to_string(),
                ))
            }
        };

        // Update transaction with payment hash as external ID
        let mut transaction: wallet_transactions::ActiveModel = updated_transaction.into();
        transaction.external_id = Set(Some(payment_hash.clone()));
        let final_transaction = self
            .repositories
            .wallet_transactions
            .update(transaction)
            .await?;

        // Update Fedimint operation with response
        self.repositories
            .fedimint_operations
            .update_response(
                fedimint_op.id,
                serde_json::to_value(&fedimint_result).unwrap(),
            )
            .await?;

        Ok(PaymentResponse {
            transaction_id: final_transaction.id,
            fedimint_operation_id: fedimint_op.id,
            payment_hash,
            amount_msats: amount_msat as u64,
            preimage: None,
        })
    }

    /// Get wallet transaction history
    pub async fn get_transactions(
        &self,
        wallet_id: Uuid,
        owner_id: Uuid,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> WalletResult<Vec<wallet_transactions::Model>> {
        self.get_wallet_for_owner(wallet_id, owner_id).await?;

        let transactions = self
            .repositories
            .wallet_transactions
            .find_by_wallet(wallet_id, limit, offset)
            .await?;

        Ok(transactions)
    }

    /// Get transaction by ID
    pub async fn get_transaction(
        &self,
        transaction_id: Uuid,
        owner_id: Uuid,
    ) -> WalletResult<wallet_transactions::Model> {
        let transaction = self
            .repositories
            .wallet_transactions
            .find_by_id(transaction_id)
            .await?
            .ok_or(WalletError::TransactionNotFound(transaction_id))?;

        // Verify user owns the wallet
        self.get_wallet_for_owner(transaction.wallet_id, owner_id)
            .await?;

        Ok(transaction)
    }

    /// Sync wallet with Fedimint federation
    pub async fn sync_wallet(&self, wallet_id: Uuid, owner_id: Uuid) -> WalletResult<()> {
        let wallet = self
            .get_active_wallet_for_owner(wallet_id, owner_id)
            .await?;

        let sync_operation = FedimintOperation::SyncWallet;
        let _result = self
            .fedimint
            .execute_operation(
                wallet
                    .federation_id
                    .as_ref()
                    .unwrap_or(&"default".to_string()),
                sync_operation,
            )
            .await?;

        // Update last sync timestamp
        self.repositories
            .wallets
            .update_last_sync(wallet_id)
            .await?;

        // TODO: Process any new transactions discovered during sync

        Ok(())
    }

    /// Get wallet by ID and verify owner ownership
    async fn get_wallet_for_owner(
        &self,
        wallet_id: Uuid,
        owner_id: Uuid,
    ) -> WalletResult<wallets::Model> {
        let wallet = self
            .repositories
            .wallets
            .find_by_id(wallet_id)
            .await?
            .ok_or(WalletError::WalletNotFound(wallet_id))?;

        if wallet.owner_id != owner_id {
            return Err(WalletError::WalletNotFound(wallet_id));
        }

        Ok(wallet)
    }

    /// Get active wallet by ID and verify owner ownership
    async fn get_active_wallet_for_owner(
        &self,
        wallet_id: Uuid,
        owner_id: Uuid,
    ) -> WalletResult<wallets::Model> {
        let wallet = self.get_wallet_for_owner(wallet_id, owner_id).await?;

        if wallet.status != WalletStatus::Active {
            return Err(WalletError::WalletNotActive(format!("{:?}", wallet.status)));
        }

        Ok(wallet)
    }
}

#[cfg(test)]
mod tests {

    // TODO: Add comprehensive tests for wallet service
    // Tests should cover:
    // - Wallet creation
    // - Balance operations
    // - Deposit/withdraw flows
    // - Lightning invoice generation/payment
    // - Error conditions
    // - User authorization checks
}
