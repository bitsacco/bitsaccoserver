// PLACEHOLDER: Groups page simplified for frontend-only mode
// CRUD operations should be implemented through API adapter pattern

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub group_type: String,
    pub status: String,
    pub member_count: Option<u64>,
    pub children_count: Option<u64>,
}

#[component]
pub fn GroupsPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-semibold text-gray-900">"Groups"</h1>
                    <p class="mt-1 text-sm text-gray-500">"Group management and organization"</p>
                </div>
                <button
                    class="bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-2 px-4 rounded-lg"
                    disabled
                >
                    "Add New Group"
                </button>
            </div>

            <div class="bg-white shadow rounded-lg p-8">
                <div class="text-center">
                    <div class="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-yellow-100">
                        <svg class="h-6 w-6 text-yellow-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
                        </svg>
                    </div>
                    <h3 class="mt-2 text-sm font-medium text-gray-900">"Groups functionality pending"</h3>
                    <p class="mt-1 text-sm text-gray-500">
                        "Groups CRUD operations will be available once the API adapter pattern is implemented."
                    </p>
                    <p class="mt-2 text-xs text-gray-400">
                        "Configure API_BACKEND=nestjs to use the NestJS backend for groups management."
                    </p>
                </div>
            </div>
        </div>
    }
}
