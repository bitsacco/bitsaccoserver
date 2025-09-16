use crate::contexts::auth::use_auth;
use leptos::prelude::*;

/// SSR-compatible protected route wrapper that works with middleware authentication
/// This component assumes authentication has already been checked by the SSR middleware
/// and simply renders the children if we reach this point
#[component]
pub fn SSRProtectedRoute(children: Children) -> impl IntoView {
    tracing::info!("🔥 SSRProtectedRoute: Component created");

    // Since we're using middleware-based authentication, if this component
    // is being rendered, it means the user has already been authenticated
    // by the ssr_auth_middleware in main.rs

    let auth = use_auth();

    // Show loading state while auth context initializes
    if auth.is_loading.get() {
        tracing::info!("🔥 SSRProtectedRoute: Auth context is loading");
        return view! {
            <div class="min-h-screen flex items-center justify-center bg-gray-50">
                <div class="text-center">
                    <div class="mx-auto h-12 w-12 flex items-center justify-center rounded-full bg-blue-100">
                        <div class="animate-spin rounded-full h-6 w-6 border-2 border-blue-600 border-t-transparent"></div>
                    </div>
                    <h2 class="mt-6 text-3xl font-extrabold text-gray-900">
                        "Loading..."
                    </h2>
                    <p class="mt-2 text-sm text-gray-600">
                        "Initializing authentication state"
                    </p>
                </div>
            </div>
        }.into_any();
    }

    // If there's an error, show it
    if let Some(error_msg) = auth.error.get() {
        tracing::error!("🔥 SSRProtectedRoute: Auth error: {}", error_msg);
        return view! {
            <div class="min-h-screen flex items-center justify-center bg-gray-50">
                <div class="text-center">
                    <h2 class="text-2xl font-semibold text-gray-900 mb-2">
                        "Authentication Error"
                    </h2>
                    <p class="text-gray-600 mb-4">
                        {error_msg}
                    </p>
                    <a href="/login" class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">
                        "Go to Login"
                    </a>
                </div>
            </div>
        }.into_any();
    }

    // Check if user is authenticated in the context
    if auth.is_authenticated.get() {
        if let Some(user) = auth.user.get() {
            tracing::info!(
                "🔥 SSRProtectedRoute: User authenticated: {} with roles: {:?}",
                user.id,
                user.roles
            );
            children().into_any()
        } else {
            tracing::warn!("🔥 SSRProtectedRoute: is_authenticated is true but no user found");
            view! {
                <div class="min-h-screen flex items-center justify-center bg-gray-50">
                    <div class="text-center">
                        <h2 class="text-2xl font-semibold text-gray-900 mb-2">
                            "Authentication State Error"
                        </h2>
                        <p class="text-gray-600 mb-4">
                            "Authentication state is inconsistent. Please try logging in again."
                        </p>
                        <a href="/login" class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">
                            "Login"
                        </a>
                    </div>
                </div>
            }.into_any()
        }
    } else {
        tracing::info!("🔥 SSRProtectedRoute: User not authenticated in context");
        // This case should not happen if middleware is working correctly,
        // but we handle it gracefully
        view! {
            <div class="min-h-screen flex items-center justify-center bg-gray-50">
                <div class="text-center">
                    <h2 class="text-2xl font-semibold text-gray-900 mb-2">
                        "Access Denied"
                    </h2>
                    <p class="text-gray-600 mb-4">
                        "You need to be logged in to access this page."
                    </p>
                    <a href="/login" class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">
                        "Login"
                    </a>
                </div>
            </div>
        }.into_any()
    }
}

/// Simplified auth wrapper that just renders children
/// Use this when middleware-based auth is handling the protection
#[component]
pub fn SimpleAuthWrapper(children: Children) -> impl IntoView {
    tracing::info!("🔥 SimpleAuthWrapper: Rendering children (auth handled by middleware)");
    children()
}
