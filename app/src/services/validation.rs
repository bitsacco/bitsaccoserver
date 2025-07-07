use crate::repositories::{Repositories, RepositoryError};
use ::entity::{share_offers, shares};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct ValidationService {
    repositories: Repositories,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SharePurchaseValidationRequest {
    pub offer_id: Uuid,
    pub owner_id: Uuid,
    pub owner_type: shares::OwnerType,
    pub quantity: Decimal,
    pub requested_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareTransferValidationRequest {
    pub from_owner_id: Uuid,
    pub from_owner_type: shares::OwnerType,
    pub to_owner_id: Uuid,
    pub to_owner_type: shares::OwnerType,
    pub share_offer_id: Uuid,
    pub quantity: Decimal,
    pub requested_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub violations: Vec<ValidationViolation>,
    pub warnings: Vec<ValidationWarning>,
    pub recommendations: Vec<ValidationRecommendation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationViolation {
    pub code: String,
    pub message: String,
    pub severity: ViolationSeverity,
    pub field: Option<String>,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationRecommendation {
    pub code: String,
    pub message: String,
    pub action: String,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Critical, // Blocks the transaction
    High,     // Should block but may be overridden
    Medium,   // Warning that should be addressed
    Low,      // Informational
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OwnerValidationInfo {
    pub exists: bool,
    pub is_active: bool,
    pub current_holdings: Vec<shares::Model>,
    pub total_shares: Decimal,
    pub total_value: Decimal,
    pub first_purchase_date: Option<chrono::DateTime<chrono::Utc>>,
    pub last_transaction_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfferValidationInfo {
    pub exists: bool,
    pub is_active: bool,
    pub is_available: bool,
    pub shares_remaining: Decimal,
    pub within_validity_period: bool,
    pub within_purchase_limits: bool,
    pub price_per_share: Decimal,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Validation configuration error: {0}")]
    Configuration(String),
    #[error("Invalid validation request: {0}")]
    InvalidRequest(String),
}

pub type ValidationServiceResult<T> = Result<T, ValidationError>;

impl ValidationService {
    pub fn new(repositories: Repositories) -> Self {
        Self { repositories }
    }

    /// Comprehensive validation for share purchase requests
    pub async fn validate_share_purchase(
        &self,
        request: &SharePurchaseValidationRequest,
    ) -> ValidationServiceResult<ValidationResult> {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Basic input validation
        self.validate_purchase_inputs(request, &mut violations)
            .await?;

        // Validate offer existence and availability
        let offer_info = self
            .validate_offer_for_purchase(request, &mut violations, &mut warnings)
            .await?;

        // Validate owner existence and eligibility
        let owner_info = self
            .validate_owner_for_purchase(request, &mut violations, &mut warnings)
            .await?;

        // Business rule validations
        self.validate_purchase_business_rules(
            request,
            &offer_info,
            &owner_info,
            &mut violations,
            &mut warnings,
        )
        .await?;

        // Generate recommendations
        self.generate_purchase_recommendations(
            request,
            &offer_info,
            &owner_info,
            &mut recommendations,
        )
        .await?;

        let is_valid = violations.iter().all(|v| {
            !matches!(
                v.severity,
                ViolationSeverity::Critical | ViolationSeverity::High
            )
        });

        Ok(ValidationResult {
            is_valid,
            violations,
            warnings,
            recommendations,
        })
    }

    /// Comprehensive validation for share transfer requests
    pub async fn validate_share_transfer(
        &self,
        request: &ShareTransferValidationRequest,
    ) -> ValidationServiceResult<ValidationResult> {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Basic input validation
        self.validate_transfer_inputs(request, &mut violations)
            .await?;

        // Validate from owner has sufficient shares
        self.validate_from_owner_holdings(request, &mut violations, &mut warnings)
            .await?;

        // Validate to owner eligibility
        self.validate_to_owner_eligibility(request, &mut violations, &mut warnings)
            .await?;

        // Business rule validations
        self.validate_transfer_business_rules(request, &mut violations, &mut warnings)
            .await?;

        // Generate recommendations
        self.generate_transfer_recommendations(request, &mut recommendations)
            .await?;

        let is_valid = violations.iter().all(|v| {
            !matches!(
                v.severity,
                ViolationSeverity::Critical | ViolationSeverity::High
            )
        });

        Ok(ValidationResult {
            is_valid,
            violations,
            warnings,
            recommendations,
        })
    }

    /// Get detailed owner validation information
    pub async fn get_owner_validation_info(
        &self,
        owner_id: Uuid,
        owner_type: shares::OwnerType,
    ) -> ValidationServiceResult<OwnerValidationInfo> {
        let holdings = self
            .repositories
            .shares
            .find_by_owner(owner_id, owner_type)
            .await?;

        let total_shares: Decimal = holdings.iter().map(|s| s.share_quantity).sum();
        let total_value: Decimal = holdings.iter().map(|s| s.total_value).sum();

        let first_purchase_date = holdings
            .iter()
            .map(|s| s.created_at.naive_utc().and_utc())
            .min();

        let last_transaction_date = holdings
            .iter()
            .filter_map(|s| s.last_transaction_at.map(|t| t.naive_utc().and_utc()))
            .max();

        // Check if owner exists and is active
        let (exists, is_active) = match owner_type {
            shares::OwnerType::Member => {
                let member = self.repositories.members.find_by_id(owner_id).await?;
                (member.is_some(), member.map_or(false, |m| m.is_active()))
            }
            shares::OwnerType::Group => {
                let group = self.repositories.groups.find_by_id(owner_id).await?;
                (group.is_some(), group.map_or(false, |g| g.is_active()))
            }
        };

        Ok(OwnerValidationInfo {
            exists,
            is_active,
            current_holdings: holdings,
            total_shares,
            total_value,
            first_purchase_date,
            last_transaction_date,
        })
    }

    /// Get detailed offer validation information
    pub async fn get_offer_validation_info(
        &self,
        offer_id: Uuid,
        quantity: Decimal,
    ) -> ValidationServiceResult<OfferValidationInfo> {
        let offer = self.repositories.share_offers.find_by_id(offer_id).await?;

        match offer {
            Some(offer) => {
                let is_active = offer.status == share_offers::ShareOfferStatus::Active;
                let is_available = offer.shares_remaining >= quantity;

                let now = chrono::Utc::now();
                let within_validity_period = {
                    let after_start = offer.valid_from.map_or(true, |start| now >= start);
                    let before_end = offer.valid_until.map_or(true, |end| now <= end);
                    after_start && before_end
                };

                let within_purchase_limits = {
                    let above_min = offer
                        .min_purchase_quantity
                        .map_or(true, |min| quantity >= min);
                    let below_max = offer
                        .max_purchase_quantity
                        .map_or(true, |max| quantity <= max);
                    above_min && below_max
                };

                Ok(OfferValidationInfo {
                    exists: true,
                    is_active,
                    is_available,
                    shares_remaining: offer.shares_remaining,
                    within_validity_period,
                    within_purchase_limits,
                    price_per_share: offer.price_per_share,
                })
            }
            None => Ok(OfferValidationInfo {
                exists: false,
                is_active: false,
                is_available: false,
                shares_remaining: Decimal::ZERO,
                within_validity_period: false,
                within_purchase_limits: false,
                price_per_share: Decimal::ZERO,
            }),
        }
    }

    // Private validation helper methods

    async fn validate_purchase_inputs(
        &self,
        request: &SharePurchaseValidationRequest,
        violations: &mut Vec<ValidationViolation>,
    ) -> ValidationServiceResult<()> {
        if request.quantity <= Decimal::ZERO {
            violations.push(ValidationViolation {
                code: "INVALID_QUANTITY".to_string(),
                message: "Purchase quantity must be greater than zero".to_string(),
                severity: ViolationSeverity::Critical,
                field: Some("quantity".to_string()),
                context: Some(serde_json::json!({"provided_quantity": request.quantity})),
            });
        }

        if request.quantity.scale() > 8 {
            violations.push(ValidationViolation {
                code: "EXCESSIVE_PRECISION".to_string(),
                message: "Quantity precision cannot exceed 8 decimal places".to_string(),
                severity: ViolationSeverity::High,
                field: Some("quantity".to_string()),
                context: Some(serde_json::json!({"provided_scale": request.quantity.scale()})),
            });
        }

        Ok(())
    }

    async fn validate_offer_for_purchase(
        &self,
        request: &SharePurchaseValidationRequest,
        violations: &mut Vec<ValidationViolation>,
        warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationServiceResult<OfferValidationInfo> {
        let offer_info = self
            .get_offer_validation_info(request.offer_id, request.quantity)
            .await?;

        if !offer_info.exists {
            violations.push(ValidationViolation {
                code: "OFFER_NOT_FOUND".to_string(),
                message: "Share offer does not exist".to_string(),
                severity: ViolationSeverity::Critical,
                field: Some("offer_id".to_string()),
                context: Some(serde_json::json!({"offer_id": request.offer_id})),
            });
        } else {
            if !offer_info.is_active {
                violations.push(ValidationViolation {
                    code: "OFFER_INACTIVE".to_string(),
                    message: "Share offer is not active".to_string(),
                    severity: ViolationSeverity::Critical,
                    field: Some("offer_id".to_string()),
                    context: None,
                });
            }

            if !offer_info.is_available {
                violations.push(ValidationViolation {
                    code: "INSUFFICIENT_SHARES".to_string(),
                    message: format!(
                        "Only {} shares remaining in offer",
                        offer_info.shares_remaining
                    ),
                    severity: ViolationSeverity::Critical,
                    field: Some("quantity".to_string()),
                    context: Some(serde_json::json!({
                        "requested": request.quantity,
                        "available": offer_info.shares_remaining
                    })),
                });
            }

            if !offer_info.within_validity_period {
                violations.push(ValidationViolation {
                    code: "OFFER_EXPIRED".to_string(),
                    message: "Share offer is outside its validity period".to_string(),
                    severity: ViolationSeverity::Critical,
                    field: Some("offer_id".to_string()),
                    context: None,
                });
            }

            if !offer_info.within_purchase_limits {
                violations.push(ValidationViolation {
                    code: "PURCHASE_LIMITS_EXCEEDED".to_string(),
                    message: "Purchase quantity is outside the allowed limits for this offer"
                        .to_string(),
                    severity: ViolationSeverity::High,
                    field: Some("quantity".to_string()),
                    context: Some(serde_json::json!({"requested": request.quantity})),
                });
            }

            // Warning for low remaining shares
            if offer_info.shares_remaining < request.quantity * Decimal::from(10) {
                warnings.push(ValidationWarning {
                    code: "LOW_REMAINING_SHARES".to_string(),
                    message: "This purchase will significantly deplete the remaining shares in this offer".to_string(),
                    field: Some("quantity".to_string()),
                    context: Some(serde_json::json!({
                        "remaining_after_purchase": offer_info.shares_remaining - request.quantity
                    })),
                });
            }
        }

        Ok(offer_info)
    }

    async fn validate_owner_for_purchase(
        &self,
        request: &SharePurchaseValidationRequest,
        violations: &mut Vec<ValidationViolation>,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationServiceResult<OwnerValidationInfo> {
        let owner_info = self
            .get_owner_validation_info(request.owner_id, request.owner_type)
            .await?;

        if !owner_info.exists {
            violations.push(ValidationViolation {
                code: "OWNER_NOT_FOUND".to_string(),
                message: format!("{:?} not found", request.owner_type),
                severity: ViolationSeverity::Critical,
                field: Some("owner_id".to_string()),
                context: Some(serde_json::json!({
                    "owner_id": request.owner_id,
                    "owner_type": request.owner_type
                })),
            });
        } else if !owner_info.is_active {
            violations.push(ValidationViolation {
                code: "OWNER_INACTIVE".to_string(),
                message: format!("{:?} is not active", request.owner_type),
                severity: ViolationSeverity::High,
                field: Some("owner_id".to_string()),
                context: None,
            });
        }

        Ok(owner_info)
    }

    async fn validate_purchase_business_rules(
        &self,
        request: &SharePurchaseValidationRequest,
        offer_info: &OfferValidationInfo,
        owner_info: &OwnerValidationInfo,
        violations: &mut Vec<ValidationViolation>,
        warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationServiceResult<()> {
        // Check for duplicate purchases (same owner, same offer)
        let existing_holdings = owner_info
            .current_holdings
            .iter()
            .any(|h| h.share_offer_id == request.offer_id);

        if existing_holdings {
            warnings.push(ValidationWarning {
                code: "DUPLICATE_OFFER_PURCHASE".to_string(),
                message: "Owner already has shares from this offer".to_string(),
                field: None,
                context: Some(serde_json::json!({
                    "offer_id": request.offer_id,
                    "owner_id": request.owner_id
                })),
            });
        }

        // Check for large single purchases
        let purchase_value = request.quantity * offer_info.price_per_share;
        let large_purchase_threshold = Decimal::from(100000); // 100,000 threshold

        if purchase_value > large_purchase_threshold {
            violations.push(ValidationViolation {
                code: "LARGE_PURCHASE_WARNING".to_string(),
                message: "Large purchase requires additional approval".to_string(),
                severity: ViolationSeverity::Medium,
                field: Some("quantity".to_string()),
                context: Some(serde_json::json!({
                    "purchase_value": purchase_value,
                    "threshold": large_purchase_threshold
                })),
            });
        }

        Ok(())
    }

    async fn generate_purchase_recommendations(
        &self,
        request: &SharePurchaseValidationRequest,
        offer_info: &OfferValidationInfo,
        owner_info: &OwnerValidationInfo,
        recommendations: &mut Vec<ValidationRecommendation>,
    ) -> ValidationServiceResult<()> {
        // Recommend optimal purchase quantities
        if offer_info.shares_remaining < request.quantity * Decimal::from(2) {
            recommendations.push(ValidationRecommendation {
                code: "CONSIDER_SMALLER_QUANTITY".to_string(),
                message: "Consider purchasing a smaller quantity to leave shares for other members"
                    .to_string(),
                action: "Reduce purchase quantity".to_string(),
                context: Some(serde_json::json!({
                    "suggested_max": offer_info.shares_remaining * Decimal::new(8, 1)
                })),
            });
        }

        // Recommend diversification if this is a large investment
        let total_after_purchase =
            owner_info.total_value + (request.quantity * offer_info.price_per_share);
        if total_after_purchase > Decimal::from(50000) && owner_info.current_holdings.len() < 3 {
            recommendations.push(ValidationRecommendation {
                code: "CONSIDER_DIVERSIFICATION".to_string(),
                message: "Consider diversifying across multiple share offers".to_string(),
                action: "Explore other available offers".to_string(),
                context: Some(serde_json::json!({
                    "current_holdings_count": owner_info.current_holdings.len(),
                    "total_value_after_purchase": total_after_purchase
                })),
            });
        }

        Ok(())
    }

    // Transfer validation methods (simplified implementations)

    async fn validate_transfer_inputs(
        &self,
        request: &ShareTransferValidationRequest,
        violations: &mut Vec<ValidationViolation>,
    ) -> ValidationServiceResult<()> {
        if request.quantity <= Decimal::ZERO {
            violations.push(ValidationViolation {
                code: "INVALID_TRANSFER_QUANTITY".to_string(),
                message: "Transfer quantity must be greater than zero".to_string(),
                severity: ViolationSeverity::Critical,
                field: Some("quantity".to_string()),
                context: None,
            });
        }

        if request.from_owner_id == request.to_owner_id
            && request.from_owner_type == request.to_owner_type
        {
            violations.push(ValidationViolation {
                code: "SELF_TRANSFER".to_string(),
                message: "Cannot transfer shares to the same owner".to_string(),
                severity: ViolationSeverity::Critical,
                field: Some("to_owner_id".to_string()),
                context: None,
            });
        }

        Ok(())
    }

    async fn validate_from_owner_holdings(
        &self,
        request: &ShareTransferValidationRequest,
        violations: &mut Vec<ValidationViolation>,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationServiceResult<()> {
        let holdings = self
            .repositories
            .shares
            .find_by_owner(request.from_owner_id, request.from_owner_type)
            .await?;

        let offer_holdings: Decimal = holdings
            .iter()
            .filter(|h| h.share_offer_id == request.share_offer_id)
            .map(|h| h.share_quantity)
            .sum();

        if offer_holdings < request.quantity {
            violations.push(ValidationViolation {
                code: "INSUFFICIENT_SHARES_FOR_TRANSFER".to_string(),
                message: "Insufficient shares for transfer".to_string(),
                severity: ViolationSeverity::Critical,
                field: Some("quantity".to_string()),
                context: Some(serde_json::json!({
                    "available": offer_holdings,
                    "requested": request.quantity
                })),
            });
        }

        Ok(())
    }

    async fn validate_to_owner_eligibility(
        &self,
        request: &ShareTransferValidationRequest,
        violations: &mut Vec<ValidationViolation>,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationServiceResult<()> {
        let owner_info = self
            .get_owner_validation_info(request.to_owner_id, request.to_owner_type)
            .await?;

        if !owner_info.exists {
            violations.push(ValidationViolation {
                code: "TRANSFER_TARGET_NOT_FOUND".to_string(),
                message: "Transfer target does not exist".to_string(),
                severity: ViolationSeverity::Critical,
                field: Some("to_owner_id".to_string()),
                context: None,
            });
        } else if !owner_info.is_active {
            violations.push(ValidationViolation {
                code: "TRANSFER_TARGET_INACTIVE".to_string(),
                message: "Transfer target is not active".to_string(),
                severity: ViolationSeverity::High,
                field: Some("to_owner_id".to_string()),
                context: None,
            });
        }

        Ok(())
    }

    async fn validate_transfer_business_rules(
        &self,
        _request: &ShareTransferValidationRequest,
        _violations: &mut Vec<ValidationViolation>,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationServiceResult<()> {
        // Additional business rules for transfers can be added here
        Ok(())
    }

    async fn generate_transfer_recommendations(
        &self,
        _request: &ShareTransferValidationRequest,
        _recommendations: &mut Vec<ValidationRecommendation>,
    ) -> ValidationServiceResult<()> {
        // Transfer-specific recommendations can be added here
        Ok(())
    }
}
