use crate::contexts::auth::{decode_jwt_token, UserInfo};
use http::request::Parts;
use leptos::prelude::*;

/// Server-side authentication check that works during SSR
/// This bypasses client-side reactive signals entirely - pure server-side admin app
pub fn check_server_auth() -> Option<UserInfo> {
    tracing::info!("🔥 ServerAuthGuard: Starting server-side auth check");

    // Server-side cookie reading using HTTP request context
    let req_parts = expect_context::<Parts>();
    tracing::info!("🔥 ServerAuthGuard: Found HTTP request context");

    if let Some(cookie_header) = req_parts.headers.get("cookie") {
        if let Ok(cookie_string) = cookie_header.to_str() {
            tracing::info!("🔥 ServerAuthGuard: Server-side cookies: {}", cookie_string);

            let token = parse_cookie_value(cookie_string, "auth_token");
            if let Some(token) = token {
                tracing::info!(
                    "🔥 ServerAuthGuard: Found auth_token: {}...",
                    &token[..token.len().min(20)]
                );

                // Decode the JWT token
                match decode_jwt_token(&token) {
                    Ok(user_info) => {
                        tracing::info!(
                            "🔥 ServerAuthGuard: JWT decoded successfully: {:?}",
                            user_info
                        );
                        return Some(user_info);
                    }
                    Err(e) => {
                        tracing::warn!("🔥 ServerAuthGuard: JWT decode failed: {}", e);
                        return None;
                    }
                }
            } else {
                tracing::warn!("🔥 ServerAuthGuard: No auth_token found in cookies");
            }
        } else {
            tracing::warn!("🔥 ServerAuthGuard: Could not parse cookie header");
        }
    } else {
        tracing::warn!("🔥 ServerAuthGuard: No cookie header found");
    }

    tracing::warn!("🔥 ServerAuthGuard: Authentication failed - no valid token found");
    None
}

fn parse_cookie_value(cookie_string: &str, name: &str) -> Option<String> {
    cookie_string.split(';').find_map(|cookie| {
        let mut parts = cookie.trim().splitn(2, '=');
        match (parts.next(), parts.next()) {
            (Some(key), Some(value)) if key == name => Some(value.to_string()),
            _ => None,
        }
    })
}

/// Simplified SSR-compatible auth guard that works with middleware
/// Since middleware handles authentication, this component just provides context access
#[component]
pub fn ServerAuthGuard(children: Children) -> impl IntoView {
    tracing::info!("🔥 ServerAuthGuard: Component created - using middleware-based auth");

    // At this point, if the component is rendered, it means the middleware
    // has already validated authentication. We just render the children.
    children()
}

/// Legacy server-side auth guard component (kept for reference)
#[component]
pub fn LegacyServerAuthGuard(children: Children) -> impl IntoView {
    tracing::info!("🔥 LegacyServerAuthGuard: Component created");

    // Only run on server side - check if we're running in SSR mode
    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing::info!("🔥 LegacyServerAuthGuard: Running on server side (not WASM)");

        // Check authentication on the server side
        let user_info = check_server_auth();

        match user_info {
            Some(user) => {
                tracing::info!("🔥 LegacyServerAuthGuard: User authenticated: {:?}", user);
                return children().into_any();
            }
            None => {
                tracing::info!(
                    "🔥 LegacyServerAuthGuard: User not authenticated, showing access denied"
                );
                return view! {
                    <div class="min-h-screen flex items-center justify-center bg-gray-50">
                        <div class="text-center">
                            <h2 class="text-2xl font-semibold text-gray-900 mb-2">
                                "Access Denied"
                            </h2>
                            <p class="text-gray-600 mb-4">
                                "You don't have permission to access this page."
                            </p>
                            <a href="/login" class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">
                                "Login"
                            </a>
                        </div>
                    </div>
                }.into_any();
            }
        }
    }

    // On client side (WASM), just render children as fallback
    #[cfg(target_arch = "wasm32")]
    {
        tracing::info!("🔥 LegacyServerAuthGuard: Running on client side (WASM) - allowing access");
        children().into_any()
    }
}
