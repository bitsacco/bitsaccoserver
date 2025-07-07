use crate::repositories::{Repositories, RepositoryError, RepositoryResult};
use ::entity::{share_offers, shares};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct SharePurchaseService {
    repositories: Repositories,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SharePurchaseRequest {
    pub share_offer_id: Uuid,
    pub owner_id: Uuid,
    pub owner_type: shares::OwnerType,
    pub quantity: rust_decimal::Decimal,
    pub purchased_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SharePurchaseResult {
    pub share_record: shares::Model,
    pub updated_offer: share_offers::Model,
    pub transaction_summary: TransactionSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub quantity_purchased: rust_decimal::Decimal,
    pub price_per_share: rust_decimal::Decimal,
    pub total_cost: rust_decimal::Decimal,
    pub offer_name: String,
    pub shares_remaining_in_offer: rust_decimal::Decimal,
    pub offer_completed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareTransferRequest {
    pub share_id: Uuid,
    pub new_owner_id: Uuid,
    pub new_owner_type: shares::OwnerType,
    pub quantity_to_transfer: Option<rust_decimal::Decimal>, // None = transfer all
    pub transferred_by: Option<Uuid>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareTransferResult {
    pub original_share: Option<shares::Model>, // None if fully transferred
    pub new_share: shares::Model,
    pub quantity_transferred: rust_decimal::Decimal,
}

#[derive(Debug, thiserror::Error)]
pub enum SharePurchaseError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Business rule violation: {0}")]
    BusinessRule(String),
    #[error("Insufficient funds or shares: {0}")]
    InsufficientResources(String),
}

pub type SharePurchaseServiceResult<T> = Result<T, SharePurchaseError>;

impl SharePurchaseService {
    pub fn new(repositories: Repositories) -> Self {
        Self { repositories }
    }

    /// Purchase shares from a specific share offer
    pub async fn purchase_shares(
        &self,
        request: SharePurchaseRequest,
    ) -> SharePurchaseServiceResult<SharePurchaseResult> {
        // Validate the purchase request
        self.validate_purchase_request(&request).await?;

        // Get the share offer and validate it's available for purchase
        let offer = self
            .repositories
            .share_offers
            .find_by_id_required(request.share_offer_id)
            .await?;

        // Validate offer availability
        self.validate_offer_for_purchase(&offer, request.quantity)
            .await?;

        // Validate owner exists
        self.validate_owner_exists(request.owner_id, request.owner_type)
            .await?;

        // Calculate transaction details
        let price_per_share = offer.price_per_share;
        let total_cost = request.quantity * price_per_share;

        // Create the share record
        let share_record = self.create_share_record(&request, &offer).await?;

        // Update the share offer with the new sales
        let updated_offer = self
            .repositories
            .share_offers
            .update_shares_sold(
                request.share_offer_id,
                request.quantity,
                request.purchased_by,
            )
            .await?;

        let transaction_summary = TransactionSummary {
            quantity_purchased: request.quantity,
            price_per_share,
            total_cost,
            offer_name: offer.name.clone(),
            shares_remaining_in_offer: updated_offer.shares_remaining,
            offer_completed: updated_offer.status == share_offers::ShareOfferStatus::Completed,
        };

        Ok(SharePurchaseResult {
            share_record,
            updated_offer,
            transaction_summary,
        })
    }

    /// Transfer shares between owners
    pub async fn transfer_shares(
        &self,
        request: ShareTransferRequest,
    ) -> SharePurchaseServiceResult<ShareTransferResult> {
        // Get the original share record
        let original_share = self
            .repositories
            .shares
            .find_by_id_required(request.share_id)
            .await?;

        // Validate transfer request
        self.validate_transfer_request(&request, &original_share)
            .await?;

        // Validate new owner exists
        self.validate_owner_exists(request.new_owner_id, request.new_owner_type)
            .await?;

        // Determine quantity to transfer
        let quantity_to_transfer = request
            .quantity_to_transfer
            .unwrap_or(original_share.share_quantity);

        if quantity_to_transfer > original_share.share_quantity {
            return Err(SharePurchaseError::Validation(
                "Cannot transfer more shares than owned".to_string(),
            ));
        }

        // Create new share record for the recipient
        let new_share = self
            .create_transferred_share_record(&original_share, &request, quantity_to_transfer)
            .await?;

        // Update or delete the original share record
        let updated_original = if quantity_to_transfer == original_share.share_quantity {
            // Full transfer - delete original record
            self.repositories.shares.delete(request.share_id).await?;
            None
        } else {
            // Partial transfer - update original record
            let new_quantity = original_share.share_quantity - quantity_to_transfer;
            let updated = self
                .repositories
                .shares
                .update_share_quantity(request.share_id, new_quantity, request.transferred_by)
                .await?;
            Some(updated)
        };

        Ok(ShareTransferResult {
            original_share: updated_original,
            new_share,
            quantity_transferred: quantity_to_transfer,
        })
    }

    /// Get aggregated share ownership for an entity
    pub async fn get_ownership_summary(
        &self,
        owner_id: Uuid,
        owner_type: shares::OwnerType,
    ) -> RepositoryResult<OwnershipSummary> {
        let shares = self
            .repositories
            .shares
            .find_by_owner(owner_id, owner_type)
            .await?;

        let total_quantity: rust_decimal::Decimal = shares.iter().map(|s| s.share_quantity).sum();
        let total_value: rust_decimal::Decimal = shares.iter().map(|s| s.total_value).sum();

        // Group by share offer for detailed breakdown
        let mut offer_breakdown = std::collections::HashMap::new();
        for share in &shares {
            let entry = offer_breakdown
                .entry(share.share_offer_id)
                .or_insert(OfferBreakdown {
                    offer_id: share.share_offer_id,
                    quantity: rust_decimal::Decimal::ZERO,
                    value: rust_decimal::Decimal::ZERO,
                    share_value: share.share_value,
                    purchase_count: 0,
                });
            entry.quantity += share.share_quantity;
            entry.value += share.total_value;
            entry.purchase_count += 1;
        }

        Ok(OwnershipSummary {
            owner_id,
            owner_type,
            total_shares: total_quantity,
            total_value,
            offers_breakdown: offer_breakdown.into_values().collect(),
            individual_purchases: shares,
        })
    }

    /// Validate purchase request
    async fn validate_purchase_request(
        &self,
        request: &SharePurchaseRequest,
    ) -> SharePurchaseServiceResult<()> {
        if request.quantity <= rust_decimal::Decimal::ZERO {
            return Err(SharePurchaseError::Validation(
                "Purchase quantity must be greater than zero".to_string(),
            ));
        }

        // Check if the specific offer is valid for this purchase
        let is_valid = self
            .repositories
            .share_offers
            .check_purchase_validity(request.share_offer_id, request.quantity)
            .await?;

        if !is_valid {
            return Err(SharePurchaseError::BusinessRule(
                "Share offer is not available for this purchase quantity".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate offer for purchase
    async fn validate_offer_for_purchase(
        &self,
        offer: &share_offers::Model,
        quantity: rust_decimal::Decimal,
    ) -> SharePurchaseServiceResult<()> {
        if offer.status != share_offers::ShareOfferStatus::Active {
            return Err(SharePurchaseError::BusinessRule(
                "Share offer is not active".to_string(),
            ));
        }

        if quantity > offer.shares_remaining {
            return Err(SharePurchaseError::InsufficientResources(format!(
                "Only {} shares remaining in offer",
                offer.shares_remaining
            )));
        }

        // Check validity dates
        let now = chrono::Utc::now();
        if let Some(valid_from) = offer.valid_from {
            if now < valid_from {
                return Err(SharePurchaseError::BusinessRule(
                    "Share offer is not yet valid".to_string(),
                ));
            }
        }

        if let Some(valid_until) = offer.valid_until {
            if now > valid_until {
                return Err(SharePurchaseError::BusinessRule(
                    "Share offer has expired".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate that owner exists
    async fn validate_owner_exists(
        &self,
        owner_id: Uuid,
        owner_type: shares::OwnerType,
    ) -> SharePurchaseServiceResult<()> {
        match owner_type {
            shares::OwnerType::Member => {
                let member = self.repositories.members.find_by_id(owner_id).await?;
                if member.is_none() {
                    return Err(SharePurchaseError::Validation(
                        "Member not found".to_string(),
                    ));
                }
            }
            shares::OwnerType::Group => {
                let group = self.repositories.groups.find_by_id(owner_id).await?;
                if group.is_none() {
                    return Err(SharePurchaseError::Validation(
                        "Group not found".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Create a new share record
    async fn create_share_record(
        &self,
        request: &SharePurchaseRequest,
        offer: &share_offers::Model,
    ) -> SharePurchaseServiceResult<shares::Model> {
        let share = shares::ActiveModel {
            id: Set(Uuid::new_v4()),
            owner_id: Set(request.owner_id),
            owner_type: Set(request.owner_type.clone()),
            share_offer_id: Set(request.share_offer_id),
            share_quantity: Set(request.quantity),
            share_value: Set(offer.price_per_share),
            total_value: Set(request.quantity * offer.price_per_share),
            last_transaction_at: Set(Some(chrono::Utc::now().into())),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(request.purchased_by),
            updated_by: Set(request.purchased_by),
        };

        let result = self.repositories.shares.create(share).await?;
        Ok(result)
    }

    /// Create transferred share record
    async fn create_transferred_share_record(
        &self,
        original: &shares::Model,
        request: &ShareTransferRequest,
        quantity: rust_decimal::Decimal,
    ) -> SharePurchaseServiceResult<shares::Model> {
        let new_total_value = quantity * original.share_value;

        let share = shares::ActiveModel {
            id: Set(Uuid::new_v4()),
            owner_id: Set(request.new_owner_id),
            owner_type: Set(request.new_owner_type.clone()),
            share_offer_id: Set(original.share_offer_id),
            share_quantity: Set(quantity),
            share_value: Set(original.share_value),
            total_value: Set(new_total_value),
            last_transaction_at: Set(Some(chrono::Utc::now().into())),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            created_by: Set(request.transferred_by),
            updated_by: Set(request.transferred_by),
        };

        let result = self.repositories.shares.create(share).await?;
        Ok(result)
    }

    /// Validate transfer request
    async fn validate_transfer_request(
        &self,
        request: &ShareTransferRequest,
        original_share: &shares::Model,
    ) -> SharePurchaseServiceResult<()> {
        if let Some(quantity) = request.quantity_to_transfer {
            if quantity <= rust_decimal::Decimal::ZERO {
                return Err(SharePurchaseError::Validation(
                    "Transfer quantity must be greater than zero".to_string(),
                ));
            }

            if quantity > original_share.share_quantity {
                return Err(SharePurchaseError::Validation(
                    "Cannot transfer more shares than owned".to_string(),
                ));
            }
        }

        // Prevent transfer to same owner
        if request.new_owner_id == original_share.owner_id
            && request.new_owner_type == original_share.owner_type
        {
            return Err(SharePurchaseError::Validation(
                "Cannot transfer shares to the same owner".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OwnershipSummary {
    pub owner_id: Uuid,
    pub owner_type: shares::OwnerType,
    pub total_shares: rust_decimal::Decimal,
    pub total_value: rust_decimal::Decimal,
    pub offers_breakdown: Vec<OfferBreakdown>,
    pub individual_purchases: Vec<shares::Model>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfferBreakdown {
    pub offer_id: Uuid,
    pub quantity: rust_decimal::Decimal,
    pub value: rust_decimal::Decimal,
    pub share_value: rust_decimal::Decimal,
    pub purchase_count: u32,
}
