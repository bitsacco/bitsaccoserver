use crate::repositories::{Repositories, RepositoryError};
use ::entity::{share_offers, shares};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct AnalyticsService {
    repositories: Repositories,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareholderSummary {
    pub total_shareholders: u64,
    pub member_shareholders: u64,
    pub group_shareholders: u64,
    pub active_shareholders: u64,
    pub top_shareholders: Vec<TopShareholderInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopShareholderInfo {
    pub owner_id: Uuid,
    pub owner_type: shares::OwnerType,
    pub total_shares: Decimal,
    pub total_value: Decimal,
    pub share_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareOfferAnalytics {
    pub total_offers: u64,
    pub active_offers: u64,
    pub completed_offers: u64,
    pub total_shares_offered: Decimal,
    pub total_shares_sold: Decimal,
    pub average_completion_rate: f64,
    pub offer_performance: Vec<OfferPerformanceInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfferPerformanceInfo {
    pub offer_id: Uuid,
    pub offer_name: String,
    pub total_shares: Decimal,
    pub shares_sold: Decimal,
    pub completion_rate: f64,
    pub total_value: Decimal,
    pub status: share_offers::ShareOfferStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketAnalytics {
    pub total_market_value: Decimal,
    pub total_shares_in_circulation: Decimal,
    pub average_share_price: Decimal,
    pub price_distribution: Vec<PricePoint>,
    pub ownership_distribution: OwnershipDistribution,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PricePoint {
    pub price: Decimal,
    pub share_count: Decimal,
    pub offer_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OwnershipDistribution {
    pub member_ownership_percentage: f64,
    pub group_ownership_percentage: f64,
    pub concentration_metrics: ConcentrationMetrics,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ConcentrationMetrics {
    pub top_10_percent_ownership: f64,
    pub top_20_percent_ownership: f64,
    pub gini_coefficient: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionAnalytics {
    pub total_transactions: u64,
    pub purchase_transactions: u64,
    pub transfer_transactions: u64,
    pub total_transaction_value: Decimal,
    pub average_transaction_size: Decimal,
    pub transaction_trends: Vec<TransactionTrend>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionTrend {
    pub period: String,
    pub transaction_count: u64,
    pub total_value: Decimal,
    pub average_size: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComprehensiveReport {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub report_period: ReportPeriod,
    pub shareholder_summary: ShareholderSummary,
    pub offer_analytics: ShareOfferAnalytics,
    pub market_analytics: MarketAnalytics,
    pub transaction_analytics: TransactionAnalytics,
    pub key_insights: Vec<KeyInsight>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportPeriod {
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyInsight {
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub metric_value: Option<Decimal>,
    pub trend: Option<TrendDirection>,
    pub severity: InsightSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    Growth,
    Risk,
    Opportunity,
    Performance,
    Alert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightSeverity {
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Calculation error: {0}")]
    Calculation(String),
    #[error("Data inconsistency: {0}")]
    DataInconsistency(String),
}

pub type AnalyticsServiceResult<T> = Result<T, AnalyticsError>;

impl AnalyticsService {
    pub fn new(repositories: Repositories) -> Self {
        Self { repositories }
    }

    /// Generate comprehensive analytics report
    pub async fn generate_comprehensive_report(
        &self,
        period: Option<ReportPeriod>,
    ) -> AnalyticsServiceResult<ComprehensiveReport> {
        let report_period = period.unwrap_or_else(|| ReportPeriod {
            start_date: None,
            end_date: None,
            description: "All-time data".to_string(),
        });

        let shareholder_summary = self.get_shareholder_summary().await?;
        let offer_analytics = self.get_offer_analytics().await?;
        let market_analytics = self.get_market_analytics().await?;
        let transaction_analytics = self.get_transaction_analytics().await?;

        let key_insights = self
            .generate_key_insights(
                &shareholder_summary,
                &offer_analytics,
                &market_analytics,
                &transaction_analytics,
            )
            .await?;

        Ok(ComprehensiveReport {
            generated_at: chrono::Utc::now(),
            report_period,
            shareholder_summary,
            offer_analytics,
            market_analytics,
            transaction_analytics,
            key_insights,
        })
    }

    /// Get shareholder analytics summary
    pub async fn get_shareholder_summary(&self) -> AnalyticsServiceResult<ShareholderSummary> {
        // Use safe fallbacks for potentially failing queries
        let member_count = self
            .repositories
            .shares
            .count_by_owner_type(shares::OwnerType::Member)
            .await
            .map_err(AnalyticsError::Repository)?;
        let group_count = self
            .repositories
            .shares
            .count_by_owner_type(shares::OwnerType::Group)
            .await
            .map_err(AnalyticsError::Repository)?;
        let total_shareholders = member_count + group_count;

        // Get top shareholders by value - use safe fallback for empty database
        let top_shareholders = self.get_top_shareholders(10).await.unwrap_or_default();

        // For now, assume all shareholders are active (this could be enhanced with activity tracking)
        let active_shareholders = total_shareholders;

        Ok(ShareholderSummary {
            total_shareholders,
            member_shareholders: member_count,
            group_shareholders: group_count,
            active_shareholders,
            top_shareholders,
        })
    }

    /// Get share offer analytics
    pub async fn get_offer_analytics(&self) -> AnalyticsServiceResult<ShareOfferAnalytics> {
        // Use safe fallbacks for potentially failing queries
        let total_offers = self
            .repositories
            .share_offers
            .count()
            .await
            .map_err(AnalyticsError::Repository)?;
        let active_offers = self
            .repositories
            .share_offers
            .count_by_status(share_offers::ShareOfferStatus::Active)
            .await
            .map_err(AnalyticsError::Repository)?;
        let completed_offers = self
            .repositories
            .share_offers
            .count_by_status(share_offers::ShareOfferStatus::Completed)
            .await
            .map_err(AnalyticsError::Repository)?;

        let total_shares_offered = self
            .repositories
            .share_offers
            .total_shares_offered()
            .await
            .map_err(AnalyticsError::Repository)?;
        let total_shares_sold = self
            .repositories
            .share_offers
            .total_shares_sold()
            .await
            .map_err(AnalyticsError::Repository)?;

        let average_completion_rate = if total_shares_offered > Decimal::ZERO {
            (total_shares_sold / total_shares_offered)
                .to_f64()
                .unwrap_or(0.0)
                * 100.0
        } else {
            0.0
        };

        let offer_performance = self.get_offer_performance().await.unwrap_or_default();

        Ok(ShareOfferAnalytics {
            total_offers,
            active_offers,
            completed_offers,
            total_shares_offered,
            total_shares_sold,
            average_completion_rate,
            offer_performance,
        })
    }

    /// Get market analytics
    pub async fn get_market_analytics(&self) -> AnalyticsServiceResult<MarketAnalytics> {
        // Use safe fallback for potentially failing queries
        let all_shares = self
            .repositories
            .shares
            .find_all()
            .await
            .map_err(AnalyticsError::Repository)?;

        let total_market_value: Decimal = all_shares.iter().map(|s| s.total_value).sum();
        let total_shares_in_circulation: Decimal =
            all_shares.iter().map(|s| s.share_quantity).sum();

        let average_share_price = if total_shares_in_circulation > Decimal::ZERO {
            total_market_value / total_shares_in_circulation
        } else {
            Decimal::ZERO
        };

        let price_distribution = self
            .calculate_price_distribution(&all_shares)
            .await
            .unwrap_or_default();
        let ownership_distribution = self
            .calculate_ownership_distribution(&all_shares)
            .await
            .unwrap_or_default();

        Ok(MarketAnalytics {
            total_market_value,
            total_shares_in_circulation,
            average_share_price,
            price_distribution,
            ownership_distribution,
        })
    }

    /// Get transaction analytics
    pub async fn get_transaction_analytics(&self) -> AnalyticsServiceResult<TransactionAnalytics> {
        // For now, we'll use shares as a proxy for transactions
        // In a real implementation, we'd have a separate transactions table
        let all_shares = self
            .repositories
            .shares
            .find_all()
            .await
            .map_err(AnalyticsError::Repository)?;

        let total_transactions = all_shares.len() as u64;
        let purchase_transactions = total_transactions; // All shares represent purchases for now
        let transfer_transactions = 0; // Would need separate tracking

        let total_transaction_value: Decimal = all_shares.iter().map(|s| s.total_value).sum();
        let average_transaction_size = if total_transactions > 0 {
            total_transaction_value / Decimal::from(total_transactions)
        } else {
            Decimal::ZERO
        };

        // Generate monthly trends (simplified)
        let transaction_trends = self
            .calculate_transaction_trends(&all_shares)
            .await
            .unwrap_or_default();

        Ok(TransactionAnalytics {
            total_transactions,
            purchase_transactions,
            transfer_transactions,
            total_transaction_value,
            average_transaction_size,
            transaction_trends,
        })
    }

    /// Get top shareholders
    async fn get_top_shareholders(
        &self,
        limit: u64,
    ) -> AnalyticsServiceResult<Vec<TopShareholderInfo>> {
        // Get all shares and group by owner
        let all_shares = self.repositories.shares.find_all().await?;
        let mut owner_holdings: HashMap<(Uuid, shares::OwnerType), (Decimal, Decimal, u64)> =
            HashMap::new();

        for share in all_shares {
            let key = (share.owner_id, share.owner_type);
            let entry = owner_holdings
                .entry(key)
                .or_insert((Decimal::ZERO, Decimal::ZERO, 0));
            entry.0 += share.share_quantity;
            entry.1 += share.total_value;
            entry.2 += 1;
        }

        let mut top_shareholders: Vec<TopShareholderInfo> = owner_holdings
            .into_iter()
            .map(
                |((owner_id, owner_type), (total_shares, total_value, share_count))| {
                    TopShareholderInfo {
                        owner_id,
                        owner_type,
                        total_shares,
                        total_value,
                        share_count,
                    }
                },
            )
            .collect();

        top_shareholders.sort_by(|a, b| b.total_value.cmp(&a.total_value));
        top_shareholders.truncate(limit as usize);

        Ok(top_shareholders)
    }

    /// Get offer performance details
    async fn get_offer_performance(&self) -> AnalyticsServiceResult<Vec<OfferPerformanceInfo>> {
        let all_offers = self.repositories.share_offers.find_all().await?;

        let performance: Vec<OfferPerformanceInfo> = all_offers
            .into_iter()
            .map(|offer| {
                let completion_rate = if offer.total_shares_available > Decimal::ZERO {
                    (offer.shares_sold / offer.total_shares_available)
                        .to_f64()
                        .unwrap_or(0.0)
                        * 100.0
                } else {
                    0.0
                };

                let total_value = offer.shares_sold * offer.price_per_share;

                OfferPerformanceInfo {
                    offer_id: offer.id,
                    offer_name: offer.name,
                    total_shares: offer.total_shares_available,
                    shares_sold: offer.shares_sold,
                    completion_rate,
                    total_value,
                    status: offer.status,
                }
            })
            .collect();

        Ok(performance)
    }

    /// Calculate price distribution
    async fn calculate_price_distribution(
        &self,
        shares: &[shares::Model],
    ) -> AnalyticsServiceResult<Vec<PricePoint>> {
        let mut price_map: HashMap<Decimal, (Decimal, u64)> = HashMap::new();

        for share in shares {
            let entry = price_map
                .entry(share.share_value)
                .or_insert((Decimal::ZERO, 0));
            entry.0 += share.share_quantity;
            entry.1 += 1;
        }

        let mut price_distribution: Vec<PricePoint> = price_map
            .into_iter()
            .map(|(price, (share_count, offer_count))| PricePoint {
                price,
                share_count,
                offer_count,
            })
            .collect();

        price_distribution.sort_by(|a, b| a.price.cmp(&b.price));
        Ok(price_distribution)
    }

    /// Calculate ownership distribution
    async fn calculate_ownership_distribution(
        &self,
        shares: &[shares::Model],
    ) -> AnalyticsServiceResult<OwnershipDistribution> {
        let total_value: Decimal = shares.iter().map(|s| s.total_value).sum();

        let member_value: Decimal = shares
            .iter()
            .filter(|s| s.owner_type == shares::OwnerType::Member)
            .map(|s| s.total_value)
            .sum();

        let group_value: Decimal = shares
            .iter()
            .filter(|s| s.owner_type == shares::OwnerType::Group)
            .map(|s| s.total_value)
            .sum();

        let member_ownership_percentage = if total_value > Decimal::ZERO {
            (member_value / total_value).to_f64().unwrap_or(0.0) * 100.0
        } else {
            0.0
        };

        let group_ownership_percentage = if total_value > Decimal::ZERO {
            (group_value / total_value).to_f64().unwrap_or(0.0) * 100.0
        } else {
            0.0
        };

        // Calculate concentration metrics (simplified)
        let concentration_metrics = self.calculate_concentration_metrics(shares).await?;

        Ok(OwnershipDistribution {
            member_ownership_percentage,
            group_ownership_percentage,
            concentration_metrics,
        })
    }

    /// Calculate concentration metrics
    async fn calculate_concentration_metrics(
        &self,
        shares: &[shares::Model],
    ) -> AnalyticsServiceResult<ConcentrationMetrics> {
        // Group by owner and calculate total holdings
        let mut owner_holdings: HashMap<(Uuid, shares::OwnerType), Decimal> = HashMap::new();

        for share in shares {
            let key = (share.owner_id, share.owner_type);
            *owner_holdings.entry(key).or_insert(Decimal::ZERO) += share.total_value;
        }

        let mut holdings: Vec<Decimal> = owner_holdings.values().cloned().collect();
        holdings.sort_by(|a, b| b.cmp(a)); // Sort descending

        let total_value: Decimal = holdings.iter().sum();
        let total_owners = holdings.len();

        // Calculate top percentiles
        let top_10_count = (total_owners as f64 * 0.1).ceil() as usize;
        let top_20_count = (total_owners as f64 * 0.2).ceil() as usize;

        let top_10_value: Decimal = holdings.iter().take(top_10_count).sum();
        let top_20_value: Decimal = holdings.iter().take(top_20_count).sum();

        let top_10_percent_ownership = if total_value > Decimal::ZERO {
            (top_10_value / total_value).to_f64().unwrap_or(0.0) * 100.0
        } else {
            0.0
        };

        let top_20_percent_ownership = if total_value > Decimal::ZERO {
            (top_20_value / total_value).to_f64().unwrap_or(0.0) * 100.0
        } else {
            0.0
        };

        // Simplified Gini coefficient calculation
        let gini_coefficient = self.calculate_gini_coefficient(&holdings).unwrap_or(0.0);

        Ok(ConcentrationMetrics {
            top_10_percent_ownership,
            top_20_percent_ownership,
            gini_coefficient,
        })
    }

    /// Calculate Gini coefficient for wealth distribution
    fn calculate_gini_coefficient(&self, holdings: &[Decimal]) -> Option<f64> {
        if holdings.is_empty() {
            return Some(0.0);
        }

        let n = holdings.len();
        let mut sorted_holdings: Vec<f64> =
            holdings.iter().map(|d| d.to_f64().unwrap_or(0.0)).collect();
        sorted_holdings.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let sum: f64 = sorted_holdings.iter().sum();
        if sum == 0.0 {
            return Some(0.0);
        }

        let mut weighted_sum = 0.0;
        for (i, value) in sorted_holdings.iter().enumerate() {
            weighted_sum += (2.0 * (i + 1) as f64 - n as f64 - 1.0) * value;
        }

        Some(weighted_sum / (n as f64 * sum))
    }

    /// Calculate transaction trends
    async fn calculate_transaction_trends(
        &self,
        shares: &[shares::Model],
    ) -> AnalyticsServiceResult<Vec<TransactionTrend>> {
        // Group transactions by month
        let mut monthly_data: HashMap<String, (u64, Decimal)> = HashMap::new();

        for share in shares {
            let month_key = share.created_at.format("%Y-%m").to_string();
            let entry = monthly_data.entry(month_key).or_insert((0, Decimal::ZERO));
            entry.0 += 1;
            entry.1 += share.total_value;
        }

        let mut trends: Vec<TransactionTrend> = monthly_data
            .into_iter()
            .map(|(period, (count, total_value))| {
                let average_size = if count > 0 {
                    total_value / Decimal::from(count)
                } else {
                    Decimal::ZERO
                };

                TransactionTrend {
                    period,
                    transaction_count: count,
                    total_value,
                    average_size,
                }
            })
            .collect();

        trends.sort_by(|a, b| a.period.cmp(&b.period));
        Ok(trends)
    }

    /// Generate key insights based on analytics data
    async fn generate_key_insights(
        &self,
        shareholder_summary: &ShareholderSummary,
        offer_analytics: &ShareOfferAnalytics,
        market_analytics: &MarketAnalytics,
        _transaction_analytics: &TransactionAnalytics,
    ) -> AnalyticsServiceResult<Vec<KeyInsight>> {
        let mut insights = Vec::new();

        // Market growth insight
        if market_analytics.total_market_value > Decimal::from(100000) {
            insights.push(KeyInsight {
                insight_type: InsightType::Growth,
                title: "Strong Market Performance".to_string(),
                description: "Total market value indicates healthy growth in share ownership"
                    .to_string(),
                metric_value: Some(market_analytics.total_market_value),
                trend: Some(TrendDirection::Up),
                severity: InsightSeverity::Info,
            });
        }

        // Offer completion insight
        if offer_analytics.average_completion_rate < 50.0 {
            insights.push(KeyInsight {
                insight_type: InsightType::Risk,
                title: "Low Offer Completion Rate".to_string(),
                description: "Share offers are not completing efficiently, consider reviewing pricing or terms".to_string(),
                metric_value: Some(Decimal::from_f64(offer_analytics.average_completion_rate).unwrap_or_default()),
                trend: Some(TrendDirection::Down),
                severity: InsightSeverity::Medium,
            });
        }

        // Concentration risk
        if market_analytics
            .ownership_distribution
            .concentration_metrics
            .top_10_percent_ownership
            > 80.0
        {
            insights.push(KeyInsight {
                insight_type: InsightType::Risk,
                title: "High Ownership Concentration".to_string(),
                description: "Top 10% of shareholders own majority of shares, consider diversification initiatives".to_string(),
                metric_value: Some(Decimal::from_f64(market_analytics.ownership_distribution.concentration_metrics.top_10_percent_ownership).unwrap_or_default()),
                trend: None,
                severity: InsightSeverity::High,
            });
        }

        // Growth opportunity
        if shareholder_summary.member_shareholders > shareholder_summary.group_shareholders * 3 {
            insights.push(KeyInsight {
                insight_type: InsightType::Opportunity,
                title: "Group Participation Opportunity".to_string(),
                description: "Individual members significantly outnumber group shareholders, potential for group engagement".to_string(),
                metric_value: Some(Decimal::from(shareholder_summary.member_shareholders)),
                trend: None,
                severity: InsightSeverity::Low,
            });
        }

        Ok(insights)
    }
}
