use crate::api::get_dashboard_metrics;
use crate::components::ui::Button;
use leptos::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// Dashboard data structures matching backend analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub shareholders: ShareholderSummary,
    pub market: MarketAnalytics,
    pub offers: ShareOfferAnalytics,
    pub transactions: TransactionAnalytics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareholderSummary {
    pub total_shareholders: u64,
    pub member_shareholders: u64,
    pub group_shareholders: u64,
    pub active_shareholders: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnalytics {
    pub total_market_value_kes: Decimal, // Total value in KES
    pub total_shares_in_circulation: Decimal,
    pub share_price_kes: Decimal, // Fixed at 1000 KES per share
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareOfferAnalytics {
    pub total_offers: u64,
    pub active_offers: u64,
    pub completed_offers: u64,
    pub average_completion_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAnalytics {
    pub total_transactions: u64,
    pub total_transaction_value_kes: Decimal, // Total value in KES
    pub average_transaction_size_kes: Decimal, // Average size in KES
}

// Enhanced data structures for dashboard display
#[derive(Debug, Clone)]
pub struct MetricCardData {
    pub title: String,
    pub value: String,
    pub change: Option<ChangeIndicator>,
    pub icon: String,
    pub description: Option<String>,
    pub color_scheme: ColorScheme,
}

#[derive(Debug, Clone)]
pub struct ChangeIndicator {
    pub percentage: f64,
    pub direction: TrendDirection,
    pub period: String,
}

#[derive(Debug, Clone, Copy)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

#[derive(Debug, Clone, Copy)]
pub enum ColorScheme {
    Blue,
    Green,
    Red,
    Yellow,
    Purple,
    Gray,
}

#[component]
pub fn DashboardContent() -> impl IntoView {
    // Create a Resource that calls our server function (which now calls NestJS API)
    let dashboard_resource = Resource::new(
        || (), // No dependencies, fetch immediately
        |_| async move {
            match get_dashboard_metrics().await {
                Ok(response) => {
                    if response.success {
                        Ok(response.data.unwrap_or_else(|| {
                            leptos::logging::warn!(
                                "Dashboard: No data in response, using fallback"
                            );
                            create_fallback_metrics()
                        }))
                    } else {
                        leptos::logging::error!(
                            "Dashboard: Server function returned error: {:?}",
                            response.errors
                        );
                        Err("Server function returned error".to_string())
                    }
                }
                Err(e) => {
                    leptos::logging::error!("Dashboard: Server function failed: {:?}", e);
                    Err(format!("Server function error: {}", e))
                }
            }
        },
    );

    view! {
        <div class="space-y-8">
            <Suspense fallback=move || view! { <DashboardSkeleton /> }>
                {move || {
                    dashboard_resource.get().map(|result| {
                        match result {
                            Ok(metrics) => {
                                // Merged dashboard content - simplified single component
                                view! {
                                    <div class="space-y-8">
                                        // Key Performance Indicators
                                        <KeyMetricsSection metrics=metrics.clone() />

                                        // Financial Overview
                                        <FinancialOverviewSection metrics=metrics.clone() />
                                    </div>
                                }.into_any()
                            },
                            Err(error_msg) => {
                                leptos::logging::error!("Dashboard error: {}", error_msg);
                                view! { <ErrorState message=error_msg /> }.into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

// Helper function to create fallback metrics if API fails
fn create_fallback_metrics() -> DashboardMetrics {
    DashboardMetrics {
        shareholders: ShareholderSummary {
            total_shareholders: 150,
            member_shareholders: 120,
            group_shareholders: 30,
            active_shareholders: 145,
        },
        market: MarketAnalytics {
            total_market_value_kes: rust_decimal::Decimal::from(1500000),
            total_shares_in_circulation: rust_decimal::Decimal::from(75000),
            share_price_kes: rust_decimal::Decimal::from(1000),
        },
        offers: ShareOfferAnalytics {
            total_offers: 25,
            active_offers: 8,
            completed_offers: 17,
            average_completion_rate: 68.0,
        },
        transactions: TransactionAnalytics {
            total_transactions: 1250,
            total_transaction_value_kes: rust_decimal::Decimal::from(750000),
            average_transaction_size_kes: rust_decimal::Decimal::from(600),
        },
    }
}

// Key metrics overview with large cards
#[component]
fn KeyMetricsSection(metrics: DashboardMetrics) -> impl IntoView {
    let key_metrics = vec![
        MetricCardData {
            title: "Total Members".to_string(),
            value: metrics.shareholders.total_shareholders.to_string(),
            change: Some(ChangeIndicator {
                percentage: 4.75,
                direction: TrendDirection::Up,
                period: "this month".to_string(),
            }),
            icon: "👥".to_string(),
            description: Some("Active SACCO members".to_string()),
            color_scheme: ColorScheme::Blue,
        },
        MetricCardData {
            title: "Market Value".to_string(),
            value: format!(
                "{:.0} KES",
                metrics
                    .market
                    .total_market_value_kes
                    .to_f64()
                    .unwrap_or(0.0)
            ),
            change: Some(ChangeIndicator {
                percentage: 12.3,
                direction: TrendDirection::Up,
                period: "vs last quarter".to_string(),
            }),
            icon: "💰".to_string(),
            description: Some("Total portfolio value in KES".to_string()),
            color_scheme: ColorScheme::Green,
        },
        MetricCardData {
            title: "Active Offers".to_string(),
            value: metrics.offers.active_offers.to_string(),
            change: Some(ChangeIndicator {
                percentage: 2.1,
                direction: TrendDirection::Down,
                period: "this week".to_string(),
            }),
            icon: "📊".to_string(),
            description: Some("Available share offers".to_string()),
            color_scheme: ColorScheme::Purple,
        },
        MetricCardData {
            title: "Share Price".to_string(),
            value: format!(
                "{} KES",
                metrics.market.share_price_kes.to_u64().unwrap_or(1000)
            ),
            change: None, // Share price is fixed, no change indicator
            icon: "📈".to_string(),
            description: Some("Fixed share price".to_string()),
            color_scheme: ColorScheme::Yellow,
        },
    ];

    view! {
        <section aria-labelledby="kpi-heading">
            <h2 id="kpi-heading" class="text-lg font-semibold text-gray-900 mb-4">"Key Performance Indicators"</h2>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
                {key_metrics.into_iter().map(|metric| view! {
                    <MetricCard data=metric />
                }).collect_view()}
            </div>
        </section>
    }
}

// Reusable metric card component
#[component]
fn MetricCard(data: MetricCardData) -> impl IntoView {
    let (bg_class, icon_bg_class, _text_class) = match data.color_scheme {
        ColorScheme::Blue => (
            "bg-blue-50 border-blue-200",
            "bg-blue-100 text-blue-600",
            "text-blue-600",
        ),
        ColorScheme::Green => (
            "bg-green-50 border-green-200",
            "bg-green-100 text-green-600",
            "text-green-600",
        ),
        ColorScheme::Red => (
            "bg-red-50 border-red-200",
            "bg-red-100 text-red-600",
            "text-red-600",
        ),
        ColorScheme::Yellow => (
            "bg-yellow-50 border-yellow-200",
            "bg-yellow-100 text-yellow-600",
            "text-yellow-600",
        ),
        ColorScheme::Purple => (
            "bg-purple-50 border-purple-200",
            "bg-purple-100 text-purple-600",
            "text-purple-600",
        ),
        ColorScheme::Gray => (
            "bg-gray-50 border-gray-200",
            "bg-gray-100 text-gray-600",
            "text-gray-600",
        ),
    };

    let aria_label = format!("{}: {}", data.title, data.value);
    let title = data.title.clone();
    let value = data.value.clone();
    let icon = data.icon.clone();
    let description = data.description.clone();
    let change = data.change.clone();

    view! {
        <div class=format!("bg-white rounded-xl shadow-sm border hover:shadow-md transition-shadow p-6 {}", bg_class)
             role="article"
             aria-label=aria_label>
            <div class="flex items-center justify-between mb-4">
                <div class=format!("p-2 rounded-lg {}", icon_bg_class) aria-hidden="true">
                    <span class="text-xl">{icon}</span>
                </div>
                {change.as_ref().map(|change| {
                    let (arrow, color_class) = match change.direction {
                        TrendDirection::Up => ("↗", "text-green-600 bg-green-50"),
                        TrendDirection::Down => ("↘", "text-red-600 bg-red-50"),
                        TrendDirection::Stable => ("→", "text-gray-600 bg-gray-50"),
                    };
                    view! {
                        <div class=format!("flex items-center px-2 py-1 rounded-full text-xs font-medium {}", color_class)
                             role="img"
                             aria-label=format!("{:.1}% change", change.percentage)>
                            <span class="mr-1" aria-hidden="true">{arrow}</span>
                            <span>{format!("{:.1}%", change.percentage)}</span>
                        </div>
                    }.into_any()
                }).unwrap_or_else(|| view! { <div></div> }.into_any())}
            </div>

            <div class="space-y-1">
                <h3 class="text-sm font-medium text-gray-500">{title}</h3>
                <p class="text-3xl font-bold text-gray-900">{value}</p>
                {description.map(|desc| view! {
                    <p class="text-sm text-gray-600">{desc}</p>
                }.into_any()).unwrap_or_else(|| view! { <div></div> }.into_any())}
                {change.map(|change| view! {
                    <p class="text-xs text-gray-500 mt-2">{change.period}</p>
                }.into_any()).unwrap_or_else(|| view! { <div></div> }.into_any())}
            </div>
        </div>
    }
}

// Financial Overview Section
#[component]
fn FinancialOverviewSection(metrics: DashboardMetrics) -> impl IntoView {
    let market_value = metrics
        .market
        .total_market_value_kes
        .to_f64()
        .unwrap_or(0.0);
    let total_transactions = metrics
        .transactions
        .total_transaction_value_kes
        .to_f64()
        .unwrap_or(0.0);
    let avg_transaction = metrics
        .transactions
        .average_transaction_size_kes
        .to_f64()
        .unwrap_or(0.0);

    view! {
        <section aria-labelledby="financial-heading">
            <div class="bg-white rounded-xl shadow-sm border p-6">
                <div class="flex items-center justify-between mb-6">
                    <h3 id="financial-heading" class="text-lg font-semibold text-gray-900">"Financial Overview"</h3>
                    <div class="flex items-center text-sm text-gray-500">
                        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6"></path>
                        </svg>
                        "Updated 5 min ago"
                    </div>
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-6">
                    <div class="text-center p-4 bg-gray-50 rounded-lg">
                        <div class="text-2xl font-bold text-gray-900">{format!("{:.0} KES", market_value)}</div>
                        <div class="text-sm text-gray-600">"Total Assets"</div>
                    </div>
                    <div class="text-center p-4 bg-gray-50 rounded-lg">
                        <div class="text-2xl font-bold text-gray-900">{format!("{:.0} KES", total_transactions)}</div>
                        <div class="text-sm text-gray-600">"Transaction Volume"</div>
                    </div>
                    <div class="text-center p-4 bg-gray-50 rounded-lg">
                        <div class="text-2xl font-bold text-gray-900">{format!("{:.0} KES", avg_transaction)}</div>
                        <div class="text-sm text-gray-600">"Avg. Transaction"</div>
                    </div>
                </div>

                <div class="space-y-3">
                    <div class="flex justify-between items-center">
                        <span class="text-sm font-medium text-gray-600">"Portfolio Growth"</span>
                        <span class="text-sm font-semibold text-green-600">"+12.3%"</span>
                    </div>
                    <div class="w-full bg-gray-200 rounded-full h-2" role="progressbar" aria-valuenow="75" aria-valuemin="0" aria-valuemax="100">
                        <div class="bg-green-500 h-2 rounded-full" style="width: 75%"></div>
                    </div>
                    <div class="flex justify-between text-xs text-gray-500">
                        <span>"Target: $2M"</span>
                        <span>"Current: $1.5M"</span>
                    </div>
                </div>
            </div>
        </section>
    }
}

// Loading skeleton component
#[component]
fn DashboardSkeleton() -> impl IntoView {
    view! {
        <div class="space-y-8" role="status" aria-label="Loading dashboard">
            // Key metrics skeleton
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
                {(0..4).map(|_| view! {
                    <div class="bg-white rounded-xl shadow-sm border p-6 animate-pulse">
                        <div class="flex items-center justify-between mb-4">
                            <div class="w-10 h-10 bg-gray-200 rounded-lg"></div>
                            <div class="w-16 h-6 bg-gray-200 rounded-full"></div>
                        </div>
                        <div class="space-y-2">
                            <div class="h-4 bg-gray-200 rounded w-1/2"></div>
                            <div class="h-8 bg-gray-200 rounded w-3/4"></div>
                            <div class="h-3 bg-gray-200 rounded w-1/3"></div>
                        </div>
                    </div>
                }).collect_view()}
            </div>

            // Content sections skeleton
            <div class="grid grid-cols-1 xl:grid-cols-2 gap-8">
                {(0..2).map(|_| view! {
                    <div class="bg-white rounded-xl shadow-sm border p-6 animate-pulse">
                        <div class="h-6 bg-gray-200 rounded w-1/3 mb-6"></div>
                        <div class="space-y-3">
                            {(0..4).map(|_| view! {
                                <div class="h-4 bg-gray-200 rounded"></div>
                            }).collect_view()}
                        </div>
                    </div>
                }).collect_view()}
            </div>
            <span class="sr-only">"Loading dashboard data..."</span>
        </div>
    }
}

// Error state component
#[component]
fn ErrorState(message: String) -> impl IntoView {
    view! {
        <div class="bg-red-50 border border-red-200 rounded-xl p-8 text-center" role="alert">
            <div class="w-12 h-12 bg-red-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg class="w-6 h-6 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16c-.77.833.192 2.5 1.732 2.5z"></path>
                </svg>
            </div>
            <h3 class="text-lg font-semibold text-red-800 mb-2">"Something went wrong"</h3>
            <p class="text-red-600 mb-4">{message}</p>
            <Button>
                "Retry"
            </Button>
        </div>
    }
}
