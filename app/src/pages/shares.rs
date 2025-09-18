// PLACEHOLDER: Shares page simplified for frontend-only mode
// CRUD operations should be implemented through API adapter pattern

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareOfferResponse {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub total_quantity: rust_decimal::Decimal,
    pub available_quantity: rust_decimal::Decimal,
    pub price_per_share: rust_decimal::Decimal,
    pub total_value: rust_decimal::Decimal,
    pub status: String,
    pub expires_at: String,
    pub progress: f64,
}

#[component]
pub fn SharesPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-semibold text-gray-900">"Shares"</h1>
                    <p class="mt-1 text-sm text-gray-500">"Manage share offers, purchases, and transfers within your SACCO"</p>
                </div>
                <button
                    class="bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-2 px-4 rounded-lg"
                    disabled
                >
                    "Create Share Offer"
                </button>
            </div>

            <div class="bg-white shadow rounded-lg p-8">
                <div class="text-center">
                    <div class="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-yellow-100">
                        <svg class="h-6 w-6 text-yellow-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                        </svg>
                    </div>
                    <h3 class="mt-2 text-sm font-medium text-gray-900">"Shares functionality pending"</h3>
                    <p class="mt-1 text-sm text-gray-500">
                        "Share offers and trading operations will be available once the API adapter pattern is implemented."
                    </p>
                    <p class="mt-2 text-xs text-gray-400">
                        "Configure API_BACKEND=nestjs to use the NestJS backend for shares management."
                    </p>
                </div>
            </div>
        </div>
    }
}
