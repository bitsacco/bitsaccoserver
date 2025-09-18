// PLACEHOLDER: Members page simplified for frontend-only mode
// CRUD operations should be implemented through API adapter pattern

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberResponse {
    pub id: uuid::Uuid,
    pub member_number: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: String,
    pub shares_count: Option<u32>,
    pub total_shares_value: Option<rust_decimal::Decimal>,
    pub groups: Option<Vec<String>>,
}

#[component]
pub fn MembersPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-semibold text-gray-900">"Members"</h1>
                    <p class="mt-1 text-sm text-gray-500">"Member management and registration"</p>
                </div>
                <button
                    class="bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-2 px-4 rounded-lg"
                    disabled
                >
                    "Add New Member"
                </button>
            </div>

            <div class="bg-white shadow rounded-lg p-8">
                <div class="text-center">
                    <div class="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-yellow-100">
                        <svg class="h-6 w-6 text-yellow-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                        </svg>
                    </div>
                    <h3 class="mt-2 text-sm font-medium text-gray-900">"Members functionality pending"</h3>
                    <p class="mt-1 text-sm text-gray-500">
                        "Members CRUD operations will be available once the API adapter pattern is implemented."
                    </p>
                    <p class="mt-2 text-xs text-gray-400">
                        "Configure API_BACKEND=nestjs to use the NestJS backend for member management."
                    </p>
                </div>
            </div>
        </div>
    }
}
