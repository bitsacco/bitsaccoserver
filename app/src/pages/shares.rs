use leptos::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareOfferResponse {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub total_quantity: Decimal,
    pub available_quantity: Decimal,
    pub price_per_share: Decimal,
    pub total_value: Decimal,
    pub status: String,
    pub expires_at: String,
    pub progress: f64,
}

#[component]
pub fn SharesPage() -> impl IntoView {
    // Create SSR-compatible resource for shares data
    let shares_resource = Resource::new(|| (), |_| crate::api::get_shares(None, None, None));
    view! {
        <div class="space-y-6">
            <div class="sm:flex sm:items-center">
                <div class="sm:flex-auto">
                    <h1 class="text-xl font-semibold text-gray-900">"Shares"</h1>
                    <p class="mt-2 text-sm text-gray-700">
                        "Manage share offers, purchases, and transfers within your SACCO."
                    </p>
                </div>
                <div class="mt-4 sm:mt-0 sm:ml-16 sm:flex-none">
                    <button
                        type="button"
                        class="inline-flex items-center justify-center rounded-md border border-transparent bg-blue-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 sm:w-auto"
                    >
                        "Create Offer"
                    </button>
                </div>
            </div>

            // Share Offers Overview
            <div class="grid grid-cols-1 gap-5 sm:grid-cols-4">
                <div class="bg-white overflow-hidden shadow rounded-lg">
                    <div class="p-5">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <div class="text-sm font-medium text-gray-500">"Active Offers"</div>
                            </div>
                        </div>
                        <div class="mt-1 flex items-baseline">
                            <div class="text-2xl font-semibold text-gray-900">"23"</div>
                        </div>
                    </div>
                </div>

                <div class="bg-white overflow-hidden shadow rounded-lg">
                    <div class="p-5">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <div class="text-sm font-medium text-gray-500">"Total Available"</div>
                            </div>
                        </div>
                        <div class="mt-1 flex items-baseline">
                            <div class="text-2xl font-semibold text-gray-900">"45,780"</div>
                        </div>
                    </div>
                </div>

                <div class="bg-white overflow-hidden shadow rounded-lg">
                    <div class="p-5">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <div class="text-sm font-medium text-gray-500">"Total Value"</div>
                            </div>
                        </div>
                        <div class="mt-1 flex items-baseline">
                            <div class="text-2xl font-semibold text-gray-900">"$2.3M"</div>
                        </div>
                    </div>
                </div>

                <div class="bg-white overflow-hidden shadow rounded-lg">
                    <div class="p-5">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <div class="text-sm font-medium text-gray-500">"Avg. Price"</div>
                            </div>
                        </div>
                        <div class="mt-1 flex items-baseline">
                            <div class="text-2xl font-semibold text-gray-900">"$50.25"</div>
                        </div>
                    </div>
                </div>
            </div>

            // Tabs
            <div class="bg-white shadow rounded-lg">
                <div class="border-b border-gray-200">
                    <nav class="-mb-px flex space-x-8 px-6" aria-label="Tabs">
                        <button class="border-blue-500 text-blue-600 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm">
                            "Share Offers"
                        </button>
                        <button class="border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm">
                            "Transactions"
                        </button>
                        <button class="border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm">
                            "Analytics"
                        </button>
                    </nav>
                </div>

                <div class="p-6">
                    <ShareOffersTable shares_resource=shares_resource />
                </div>
            </div>
        </div>
    }
}

#[component]
fn ShareOffersTable(
    shares_resource: Resource<
        Result<
            crate::api::ApiResponse<crate::api::PaginatedResponse<ShareOfferResponse>>,
            ServerFnError,
        >,
    >,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            // Filters
            <div class="grid grid-cols-1 gap-4 sm:grid-cols-4">
                <div>
                    <label for="offer-search" class="block text-sm font-medium text-gray-700">
                        "Search Offers"
                    </label>
                    <input
                        type="text"
                        name="offer-search"
                        id="offer-search"
                        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
                        placeholder="Title or ID..."
                    />
                </div>
                <div>
                    <label for="offer-status" class="block text-sm font-medium text-gray-700">
                        "Status"
                    </label>
                    <select
                        id="offer-status"
                        name="offer-status"
                        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
                    >
                        <option>"All Statuses"</option>
                        <option>"Active"</option>
                        <option>"Paused"</option>
                        <option>"Completed"</option>
                        <option>"Cancelled"</option>
                    </select>
                </div>
                <div>
                    <label for="price-range" class="block text-sm font-medium text-gray-700">
                        "Price Range"
                    </label>
                    <select
                        id="price-range"
                        name="price-range"
                        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
                    >
                        <option>"All Prices"</option>
                        <option>"$0 - $25"</option>
                        <option>"$25 - $50"</option>
                        <option>"$50 - $100"</option>
                        <option>"$100+"</option>
                    </select>
                </div>
                <div>
                    <label for="issuer" class="block text-sm font-medium text-gray-700">
                        "Issuer"
                    </label>
                    <select
                        id="issuer"
                        name="issuer"
                        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
                    >
                        <option>"All Issuers"</option>
                        <option>"SACCO Board"</option>
                        <option>"Member Transfers"</option>
                        <option>"Group Offers"</option>
                    </select>
                </div>
            </div>

            // Table
            <div class="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg">
                <table class="min-w-full divide-y divide-gray-300">
                    <thead class="bg-gray-50">
                        <tr>
                            <th class="px-6 py-3.5 text-left text-sm font-semibold text-gray-900">
                                "Offer"
                            </th>
                            <th class="px-6 py-3.5 text-left text-sm font-semibold text-gray-900">
                                "Quantity & Price"
                            </th>
                            <th class="px-6 py-3.5 text-left text-sm font-semibold text-gray-900">
                                "Progress"
                            </th>
                            <th class="px-6 py-3.5 text-left text-sm font-semibold text-gray-900">
                                "Status"
                            </th>
                            <th class="px-6 py-3.5 text-left text-sm font-semibold text-gray-900">
                                "Expires"
                            </th>
                            <th class="relative px-6 py-3.5">
                                <span class="sr-only">"Actions"</span>
                            </th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-200 bg-white">
                        <Suspense fallback=|| view! {
                            <tr>
                                <td colspan="6" class="px-6 py-12 text-center">
                                    <div class="animate-pulse">
                                        <div class="h-4 bg-gray-200 rounded w-3/4 mx-auto mb-2"></div>
                                        <div class="h-3 bg-gray-200 rounded w-1/2 mx-auto"></div>
                                    </div>
                                </td>
                            </tr>
                        }>
                            {move || {
                                match shares_resource.get() {
                                    Some(Ok(response)) => {
                                        if let Some(paginated_data) = response.data {
                                            paginated_data.data.into_iter().map(|share| {
                                                view! {
                                                    <ShareOfferRow share=share />
                                                }
                                            }).collect::<Vec<_>>().into_any()
                                        } else {
                                            view! {
                                                <tr>
                                                    <td colspan="6" class="px-6 py-4 text-center text-gray-500">
                                                        "No share offers available"
                                                    </td>
                                                </tr>
                                            }.into_any()
                                        }
                                    }
                                    Some(Err(err)) => {
                                        view! {
                                            <tr>
                                                <td colspan="6" class="px-6 py-4 text-center text-red-600">
                                                    "Error loading shares: " {err.to_string()}
                                                </td>
                                            </tr>
                                        }.into_any()
                                    }
                                    None => view! {
                                        <tr>
                                            <td colspan="6" class="px-6 py-4 text-center text-gray-500">
                                                "Loading..."
                                            </td>
                                        </tr>
                                    }.into_any()
                                }
                            }}
                        </Suspense>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn ShareOfferRow(share: ShareOfferResponse) -> impl IntoView {
    let status_class = match share.status.as_str() {
        "Active" => "bg-green-100 text-green-800",
        "Paused" => "bg-yellow-100 text-yellow-800",
        "Completed" => "bg-blue-100 text-blue-800",
        "Cancelled" => "bg-red-100 text-red-800",
        _ => "bg-gray-100 text-gray-800",
    };

    let progress_color = if share.progress > 75.0 {
        "bg-green-600"
    } else if share.progress > 50.0 {
        "bg-yellow-500"
    } else if share.progress > 25.0 {
        "bg-blue-600"
    } else {
        "bg-gray-400"
    };

    view! {
        <tr>
            <td class="px-6 py-4">
                <div class="flex items-start">
                    <div class="flex-shrink-0 h-10 w-10">
                        <div class="h-10 w-10 rounded bg-blue-100 flex items-center justify-center">
                            <span class="text-sm font-medium text-blue-600">
                                {share.title.chars().take(2).collect::<String>()}
                            </span>
                        </div>
                    </div>
                    <div class="ml-4">
                        <div class="text-sm font-medium text-gray-900">{share.title.clone()}</div>
                        <div class="text-sm text-gray-500">{share.description.clone().unwrap_or_default()}</div>
                    </div>
                </div>
            </td>
            <td class="px-6 py-4">
                <div class="text-sm text-gray-900">
                    {format!("{} / {} shares", share.available_quantity, share.total_quantity)}
                </div>
                <div class="text-sm text-gray-500">
                    {format!("${:.2} per share (${:.2} total)", share.price_per_share, share.total_value)}
                </div>
            </td>
            <td class="px-6 py-4">
                <div class="text-sm text-gray-900 mb-1">
                    {format!("{:.1}% sold", share.progress)}
                </div>
                <div class="w-full bg-gray-200 rounded-full h-2">
                    <div
                        class={format!("h-2 rounded-full {}", progress_color)}
                        style={format!("width: {}%", share.progress)}
                    ></div>
                </div>
            </td>
            <td class="px-6 py-4">
                <span class={format!("inline-flex px-2 py-1 text-xs font-semibold rounded-full {}", status_class)}>
                    {share.status.clone()}
                </span>
            </td>
            <td class="px-6 py-4 text-sm text-gray-500">
                {share.expires_at.clone()}
            </td>
            <td class="px-6 py-4 text-right text-sm font-medium space-x-2">
                <button class="text-blue-600 hover:text-blue-900">"View"</button>
                <button class="text-blue-600 hover:text-blue-900">"Edit"</button>
                <button class="text-red-600 hover:text-red-900">"Pause"</button>
            </td>
        </tr>
    }
}
