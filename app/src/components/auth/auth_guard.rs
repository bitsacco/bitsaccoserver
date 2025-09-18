use leptos::prelude::*;

/// Simplified authentication guard for frontend-only mode
/// In frontend-only mode, we assume authentication is handled by the NestJS backend
/// This component just passes through to the protected content
#[component]
pub fn AuthGuard(children: Children) -> impl IntoView {
    tracing::info!("AuthGuard: Frontend-only mode - delegating auth to backend");

    // In frontend-only mode, we simply render the protected content
    // Authentication is handled by the NestJS backend and API calls
    // will fail with appropriate HTTP status codes if not authenticated
    view! {
        <div class="auth-guard-wrapper">
            {children()}
        </div>
    }
}
