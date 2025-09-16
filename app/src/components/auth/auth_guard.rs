use crate::middleware::auth_compat::SSRAuthContext;
use http::request::Parts;
use leptos::prelude::*;

/// SSR-compatible authentication guard that works with middleware-based authentication
/// This component assumes authentication has been checked by SSR middleware and
/// extracts the validated user context from the request
#[component]
pub fn AuthGuard(children: Children) -> impl IntoView {
    tracing::info!("🔥 SSRAuthGuard: Component created - middleware-based authentication");

    // During SSR, try to get auth context from request extensions
    // During hydration/client-side, this will be None and we fall back to client auth
    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing::info!(
            "🔥 SSRAuthGuard: Running on server side - checking middleware auth context"
        );

        // Try to get HTTP request parts
        match use_context::<Parts>() {
            Some(parts) => {
                tracing::info!("🔥 SSRAuthGuard: Found HTTP request context");

                if let Some(ssr_auth_context) = parts.extensions.get::<SSRAuthContext>() {
                    match ssr_auth_context {
                        SSRAuthContext::Authenticated(user_info) => {
                            tracing::info!("🔥 SSRAuthGuard: User authenticated via middleware: {} with roles: {:?}",
                                user_info.id, user_info.roles);
                            return children().into_any();
                        }
                        SSRAuthContext::Unauthenticated => {
                            tracing::warn!("🔥 SSRAuthGuard: User not authenticated - this should have been caught by middleware");
                            return create_access_denied_view("Authentication required").into_any();
                        }
                        SSRAuthContext::InvalidToken(error) => {
                            tracing::warn!("🔥 SSRAuthGuard: Invalid token: {} - this should have been caught by middleware", error);
                            return create_access_denied_view("Invalid authentication token")
                                .into_any();
                        }
                    }
                } else {
                    tracing::warn!("🔥 SSRAuthGuard: No SSRAuthContext found in request extensions - middleware may not have run");
                    return create_access_denied_view("Authentication context not found")
                        .into_any();
                }
            }
            None => {
                tracing::warn!("🔥 SSRAuthGuard: Could not access HTTP request context");
                return create_access_denied_view("Request context not available").into_any();
            }
        }
    }

    // Client-side fallback - use reactive auth context
    #[cfg(target_arch = "wasm32")]
    {
        tracing::info!("🔥 SSRAuthGuard: Running on client side - checking reactive auth context");

        use crate::contexts::auth::use_auth;
        let auth = use_auth();

        if auth.is_loading.get() {
            return view! {
                <div class="min-h-screen flex items-center justify-center bg-gray-50">
                    <div class="text-center">
                        <div class="mx-auto h-12 w-12 flex items-center justify-center rounded-full bg-blue-100">
                            <div class="animate-spin rounded-full h-6 w-6 border-2 border-blue-600 border-t-transparent"></div>
                        </div>
                        <h2 class="mt-6 text-xl font-semibold text-gray-900">
                            "Loading..."
                        </h2>
                        <p class="mt-2 text-sm text-gray-600">
                            "Verifying authentication..."
                        </p>
                    </div>
                </div>
            }.into_any();
        }

        if auth.is_authenticated.get() {
            if let Some(user) = auth.user.get() {
                tracing::info!(
                    "🔥 SSRAuthGuard: Client-side user authenticated: {} with roles: {:?}",
                    user.id,
                    user.roles
                );
                return children().into_any();
            }
        }

        tracing::warn!("🔥 SSRAuthGuard: Client-side authentication failed");
        return create_access_denied_view("Please log in to access this page").into_any();
    }
}

/// Create a consistent access denied view
fn create_access_denied_view(message: &str) -> impl IntoView {
    let message = message.to_string();
    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50">
            <div class="text-center max-w-md mx-auto p-8">
                <div class="mx-auto h-12 w-12 flex items-center justify-center rounded-full bg-red-100 mb-4">
                    <svg class="h-6 w-6 text-red-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
                    </svg>
                </div>
                <h2 class="text-2xl font-semibold text-gray-900 mb-2">
                    "Access Denied"
                </h2>
                <p class="text-gray-600 mb-6">
                    {message}
                </p>
                <a href="/login" class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors">
                    "Go to Login"
                </a>
            </div>
        </div>
    }
}

/// Role-based auth guard that works with SSR middleware
#[component]
pub fn SSRRoleGuard(
    children: Children,
    required_roles: Vec<String>,
    #[prop(optional)] fallback_message: Option<String>,
) -> impl IntoView {
    tracing::info!("🔥 SSRRoleGuard: Checking roles: {:?}", required_roles);

    // First check basic authentication
    #[cfg(not(target_arch = "wasm32"))]
    {
        match use_context::<Parts>() {
            Some(parts) => {
                if let Some(ssr_auth_context) = parts.extensions.get::<SSRAuthContext>() {
                    match ssr_auth_context {
                        SSRAuthContext::Authenticated(user_info) => {
                            // Check if user has any of the required roles
                            let has_required_role = required_roles
                                .iter()
                                .any(|role| user_info.roles.contains(role));

                            if has_required_role {
                                tracing::info!(
                                    "🔥 SSRRoleGuard: User has required roles, granting access"
                                );
                                return children().into_any();
                            } else {
                                tracing::warn!("🔥 SSRRoleGuard: User lacks required roles. Has: {:?}, Needs: {:?}",
                                    user_info.roles, required_roles);
                                let message = fallback_message.unwrap_or_else(|| {
                                    format!(
                                        "You need one of these roles: {}",
                                        required_roles.join(", ")
                                    )
                                });
                                return create_access_denied_view(&message).into_any();
                            }
                        }
                        _ => {
                            return create_access_denied_view("Authentication required").into_any();
                        }
                    }
                } else {
                    return create_access_denied_view("Authentication required").into_any();
                }
            }
            None => {
                return create_access_denied_view("Authentication required").into_any();
            }
        }
    }

    // Client-side fallback
    #[cfg(target_arch = "wasm32")]
    {
        use crate::contexts::auth::use_auth;
        let auth = use_auth();

        if let Some(user) = auth.user.get() {
            let has_required_role = required_roles.iter().any(|role| user.roles.contains(role));

            if has_required_role {
                return children().into_any();
            } else {
                let message = fallback_message.unwrap_or_else(|| {
                    format!("You need one of these roles: {}", required_roles.join(", "))
                });
                return create_access_denied_view(&message).into_any();
            }
        }
    }

    // This should never be reached due to the cfg attributes above,
    // but we need this for compilation when neither cfg is active
    #[allow(unreachable_code)]
    create_access_denied_view("Access denied").into_any()
}

/// Admin-only guard (requires admin or superadmin role)
#[component]
pub fn SSRAdminGuard(children: Children) -> impl IntoView {
    view! {
        <SSRRoleGuard required_roles=vec!["admin".to_string(), "superadmin".to_string()]>
            {children()}
        </SSRRoleGuard>
    }
}

/// Superadmin-only guard
#[component]
pub fn SSRSuperAdminGuard(children: Children) -> impl IntoView {
    view! {
        <SSRRoleGuard required_roles=vec!["superadmin".to_string()]>
            {children()}
        </SSRRoleGuard>
    }
}
