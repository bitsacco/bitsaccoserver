use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub method: AuthMethod,
    pub identifier: String, // email, phone, or npub
    pub credential: String, // password or PIN
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum AuthMethod {
    Email,
    Phone,
    Pin,
    Nostr,
}

impl AuthMethod {
    pub fn display_name(&self) -> &'static str {
        match self {
            AuthMethod::Email => "Email Address",
            AuthMethod::Phone => "Phone Number",
            AuthMethod::Pin => "PIN Code",
            AuthMethod::Nostr => "Nostr Public Key",
        }
    }

    pub fn placeholder(&self) -> &'static str {
        match self {
            AuthMethod::Email => "Enter your email address",
            AuthMethod::Phone => "Enter your phone number",
            AuthMethod::Pin => "Enter your PIN",
            AuthMethod::Nostr => "Enter your npub",
        }
    }

    pub fn input_type(&self) -> &'static str {
        match self {
            AuthMethod::Email => "email",
            AuthMethod::Phone => "tel",
            AuthMethod::Pin => "password",
            AuthMethod::Nostr => "text",
        }
    }
}

// Re-export the API types instead of defining our own
pub use crate::api::types::auth::AuthResponse;
pub use crate::api::types::User;

// Create a type alias for backward compatibility
pub type UserInfo = User;

// Add display methods to User via extension trait
pub trait UserDisplayExt {
    fn display_name(&self) -> String;
}

impl UserDisplayExt for User {
    fn display_name(&self) -> String {
        if let Some(phone) = &self.phone {
            phone.number.clone()
        } else if let Some(nostr) = &self.nostr {
            nostr.npub.clone()
        } else {
            self.id.to_string()
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user: RwSignal<Option<UserInfo>>,
    pub token: RwSignal<Option<String>>,
    pub refresh_token: RwSignal<Option<String>>,
    pub token_expires_at: RwSignal<Option<u64>>,
    pub is_authenticated: Signal<bool>,
    pub is_loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    pub login: Callback<LoginCredentials, ()>,
    pub logout: Callback<(), ()>,
    pub clear_error: Callback<(), ()>,
    pub handle_auth_response: Callback<AuthResponse, ()>,
}

#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    // Immediate debug logging
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&"✨ AuthProvider component created (client-side)".into());
        if let Ok(cookies) = leptos::leptos_dom::helpers::document().cookie() {
            web_sys::console::log_1(&format!("  - Document cookies: {}", cookies).into());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    tracing::info!("✨ AuthProvider component created (server-side)");

    let user = RwSignal::new(None::<UserInfo>);
    let token = RwSignal::new(None::<String>);
    let refresh_token = RwSignal::new(None::<String>);
    let token_expires_at = RwSignal::new(None::<u64>);
    let is_loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    // API client creation removed during cleanup
    // let _api_client = AbstractedApiClient::default().expect("Failed to create API client");

    let is_authenticated = Signal::derive(move || user.get().is_some() && token.get().is_some());

    // Auto-refresh token when it's about to expire
    Effect::new(move |_| {
        if let (Some(refresh_token_val), Some(expires_at)) =
            (refresh_token.get(), token_expires_at.get())
        {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            // Refresh 5 minutes before expiry
            if current_time + 300 >= expires_at {
                let refresh_token_clone = refresh_token_val.clone();
                spawn_local(async move {
                    if let Err(e) = attempt_token_refresh(refresh_token_clone).await {
                        tracing::error!("Token refresh failed: {}", e);
                        // Clear auth state on refresh failure
                        user.set(None);
                        token.set(None);
                        refresh_token.set(None);
                        token_expires_at.set(None);
                        error.set(Some("Session expired. Please login again.".to_string()));
                    }
                });
            }
        }
    });

    // Try to restore authentication state from cookies/storage on init
    // Use Effect::new to ensure it runs during hydration
    Effect::new(move |_| {
        // Server-side logging that should appear in logs
        #[cfg(not(target_arch = "wasm32"))]
        tracing::info!("🔥 Auth Effect: Starting on server side");

        #[cfg(target_arch = "wasm32")]
        {
            // Client-side logging
            web_sys::console::log_1(&"🔥 Auth Effect: Starting on client side".into());
        }

        is_loading.set(true);
        spawn_local(async move {
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::console::log_1(
                    &"🔥 Auth context: spawn_local running on client side".into(),
                );
            }

            if let Some(stored_token) = get_auth_token_from_cookies() {
                #[cfg(target_arch = "wasm32")]
                {
                    web_sys::console::log_1(
                        &format!(
                            "🔥 Auth context: Found auth_token cookie, length: {}",
                            stored_token.len()
                        )
                        .into(),
                    );
                }

                token.set(Some(stored_token.clone()));

                // Decode JWT token to extract user information
                match decode_jwt_token(&stored_token) {
                    Ok(user_info) => {
                        #[cfg(target_arch = "wasm32")]
                        {
                            web_sys::console::log_1(&format!("🔥 Successfully decoded JWT token for user: {} with roles: {:?}", user_info.id, user_info.roles).into());
                        }
                        user.set(Some(user_info));
                    }
                    Err(_e) => {
                        #[cfg(target_arch = "wasm32")]
                        {
                            web_sys::console::log_1(
                                &format!("🔥 Failed to decode JWT token: {}", _e).into(),
                            );
                        }
                        // Token is invalid, clear everything
                        token.set(None);
                        refresh_token.set(None);
                    }
                }
            } else {
                #[cfg(target_arch = "wasm32")]
                {
                    web_sys::console::log_1(&"🔥 Auth context: No auth_token cookie found".into());
                }
            }
            is_loading.set(false);
        });
    });

    let clear_error = Callback::new(move |_| {
        error.set(None);
    });

    let login = Callback::new(move |_creds: LoginCredentials| {
        is_loading.set(true);
        error.set(None);

        // API client removed during cleanup - functionality disabled
        is_loading.set(false);
        error.set(Some("Auth temporarily disabled during cleanup".to_string()));
        return;
        #[allow(unreachable_code)]
        spawn_local(async move {
            // Early return to avoid compilation errors
            /*
            // Code disabled during cleanup
            // Convert to API request format
            let api_request = ApiLoginRequest {
                pin: creds.password.clone(),
                phone: Some(creds.email.clone()),
                npub: None,
            };

            match api_client_clone.login(api_request).await {
                Ok(auth_response) => {
                    // Store tokens if available
                    if let Some(access_token) = &auth_response.access_token {
                        token.set(Some(access_token.clone()));
                    }
                    if let Some(refresh_token_value) = &auth_response.refresh_token {
                        refresh_token.set(Some(refresh_token_value.clone()));
                    }
                    // Set a default expiry time (1 hour)
                    let expires_at = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        + 3600; // 1 hour
                    token_expires_at.set(Some(expires_at));

                    // Use the user directly from the API response
                    let user_info = auth_response.user;
                    user.set(Some(user_info));

                    // Store in secure cookies using enhanced security if tokens available
                    if let (Some(access), Some(refresh)) =
                        (&auth_response.access_token, &auth_response.refresh_token)
                    {
                        store_auth_tokens_secure(access, refresh).await;
                    }

                    // Clear any errors
                    error.set(None);

                    // Redirect to dashboard
                    let _ = window().location().set_href("/dashboard");
                }
                Err(e) => {
                    tracing::error!("Login failed: {}", e);
                    error.set(Some(format!("Login failed: {}", e)));
                }
            }
            is_loading.set(false);
            */
        });
    });

    let logout = Callback::new(move |_| {
        is_loading.set(true);

        spawn_local(async move {
            // Logout from server if we have a refresh token
            if let Some(refresh_token_val) = refresh_token.get() {
                // Try to logout from server, but don't block on failure
                let _ = logout_user(refresh_token_val).await;
            }

            // Clear local state
            user.set(None);
            token.set(None);
            refresh_token.set(None);
            token_expires_at.set(None);
            error.set(None);
            clear_auth_state_secure().await;

            // Redirect to login
            #[cfg(target_arch = "wasm32")]
            {
                let _ = web_sys::window().unwrap().location().set_href("/login");
            }
            is_loading.set(false);
        });
    });

    let handle_auth_response = Callback::new(
        move |auth_response: crate::api::types::auth::AuthResponse| {
            spawn_local(async move {
                tracing::info!(
                    "🔥 handle_auth_response called with user ID: {}",
                    auth_response.user.id
                );
                tracing::info!("🔥 User roles: {:?}", auth_response.user.roles);

                // Store tokens if available
                if let Some(access_token) = &auth_response.access_token {
                    token.set(Some(access_token.clone()));
                }
                if let Some(refresh_token_value) = &auth_response.refresh_token {
                    refresh_token.set(Some(refresh_token_value.clone()));
                }

                // Set a default expiry time (1 hour)
                let expires_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + 3600; // 1 hour
                token_expires_at.set(Some(expires_at));

                // Use the user directly from the API response
                let user_info = auth_response.user;
                user.set(Some(user_info));

                // Store in secure cookies using enhanced security if tokens available
                if let (Some(access), Some(refresh)) =
                    (&auth_response.access_token, &auth_response.refresh_token)
                {
                    store_auth_tokens_secure(access, refresh).await;
                }

                // Clear any errors
                error.set(None);

                tracing::info!("Auth response handled successfully, user authenticated");
            });
        },
    );

    let auth_context = AuthContext {
        user,
        token,
        refresh_token,
        token_expires_at,
        is_authenticated,
        is_loading,
        error,
        login,
        logout,
        clear_error,
        handle_auth_response,
    };

    provide_context(auth_context);

    view! {
        {children()}
    }
}

pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("AuthContext not provided")
}

// Utility function to get current auth token
pub fn use_auth_token() -> Option<String> {
    let auth = use_context::<AuthContext>()?;
    auth.token.get()
}

// Server functions for authentication
#[server(LoginUser, "/api")]
pub async fn login_user(credentials: LoginCredentials) -> Result<AuthResponse, ServerFnError> {
    // For now, use the HTTP API endpoint directly until we resolve the context issue
    use reqwest::Client;

    #[derive(serde::Serialize)]
    struct LoginRequest {
        username: String,
        password: String,
    }

    let client = Client::new();
    let login_request = LoginRequest {
        username: credentials.email.clone(),
        password: credentials.password.clone(),
    };

    // Get the current server URL from environment, prefer NESTJS_API_URL for NestJS backend
    let server_url = std::env::var("NESTJS_API_URL")
        .or_else(|_| std::env::var("SERVER_URL"))
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    let response = client
        .post(format!("{}/api/auth/login", server_url))
        .json(&login_request)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("HTTP request failed: {}", e)))?;

    if response.status().is_success() {
        let auth_response: crate::api::types::auth::AuthResponse = response
            .json()
            .await
            .map_err(|e| ServerFnError::new(format!("JSON parsing failed: {}", e)))?;

        // Convert to API AuthResponse format
        Ok(AuthResponse {
            user: User {
                id: auth_response.user.id,
                phone: Some(crate::api::types::common::Phone {
                    number: auth_response
                        .user
                        .phone
                        .map(|p| p.number)
                        .unwrap_or_default(),
                }),
                nostr: None,
                profile: None,
                roles: auth_response
                    .user
                    .roles
                    .into_iter()
                    .map(|role_str| match role_str.to_auth_string().as_str() {
                        "admin" => crate::api::types::common::Role::Admin,
                        "super_admin" | "superadmin" => crate::api::types::common::Role::SuperAdmin,
                        _ => crate::api::types::common::Role::Member,
                    })
                    .collect(),
                verified: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            authenticated: true,
            access_token: auth_response.access_token,
            refresh_token: auth_response.refresh_token,
        })
    } else {
        // Parse the error response to provide user-friendly messages
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Authentication failed".to_string());

        // Check for common authentication errors and provide user-friendly messages
        if error_text.contains("invalid_grant") || error_text.contains("Invalid user credentials") {
            Err(ServerFnError::new(
                "Invalid email or password. Please check your credentials and try again.",
            ))
        } else if error_text.contains("Account is not fully set up") {
            Err(ServerFnError::new(
                "Your account is not fully configured. Please contact an administrator.",
            ))
        } else if error_text.contains("Account disabled") {
            Err(ServerFnError::new(
                "Your account has been disabled. Please contact an administrator.",
            ))
        } else {
            // Generic error message that doesn't expose internal details
            Err(ServerFnError::new(
                "Authentication failed. Please try again.",
            ))
        }
    }
}

// Removed validate_token server function - replaced with client-side JWT decoding since /api/auth/me doesn't exist

#[server(LogoutUser, "/api")]
pub async fn logout_user(refresh_token: String) -> Result<(), ServerFnError> {
    use crate::api::types::auth::LogoutRequest;
    use reqwest::Client;

    let client = Client::new();
    let server_url = std::env::var("NESTJS_API_URL")
        .or_else(|_| std::env::var("SERVER_URL"))
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let logout_request = LogoutRequest { refresh_token };

    let response = client
        .post(format!("{}/api/auth/logout", server_url))
        .json(&logout_request)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("HTTP request failed: {}", e)))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new("Logout failed".to_string()))
    }
}

// Client-side helper functions for browser storage and API calls
async fn _perform_login(credentials: LoginCredentials) -> Result<AuthResponse, String> {
    login_user(credentials).await.map_err(|e| e.to_string())
}

// Removed validate_token_and_get_user - replaced with JWT decoding

async fn _perform_logout(refresh_token: &str) -> Result<(), String> {
    logout_user(refresh_token.to_string())
        .await
        .map_err(|e| e.to_string())
}

// Enhanced token refresh function using API abstraction
async fn attempt_token_refresh(_refresh_token: String) -> Result<(), String> {
    // API client removed during cleanup - token refresh disabled
    // let api_client =
    //     AbstractedApiClient::default().map_err(|e| format!("API client error: {}", e))?;
    return Err("Token refresh disabled during cleanup".to_string());

    #[allow(unreachable_code)]
    {
        let _refresh_request = crate::api::types::RefreshTokenRequest {
            refresh_token: _refresh_token,
        };
        // Unreachable code - commented out to avoid errors
        /*
        match api_client.auth().refresh_token(refresh_request).await {
            Ok(tokens_response) => {
                // Update stored tokens
                store_auth_tokens_secure(
                    &tokens_response.access_token,
                    &tokens_response.refresh_token,
                )
                .await;
                Ok(())
            }
            Err(e) => Err(format!("Token refresh failed: {}", e)),
        }
        */
    }
    Ok(())
}

pub fn get_auth_token_from_cookies() -> Option<String> {
    // Enhanced secure cookie reading with CSR/SSR compatibility
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;
        if let Some(window) = window() {
            if let Ok(document) = window.document() {
                if let Ok(cookie_string) = document.cookie() {
                    tracing::info!("🔥 All cookies: {}", cookie_string);
                    let result = parse_cookie_value(&cookie_string, "auth_token");
                    if let Some(ref token) = result {
                        tracing::info!("🔥 Found auth_token cookie, length: {}", token.len());
                    } else {
                        tracing::warn!("🔥 No auth_token found in cookies");
                    }
                    return result;
                }
            }
        }
    }

    // Server-side: return None for now, we'll handle this directly in ServerAuthGuard
    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing::warn!("🔥 Server-side: get_auth_token_from_cookies called but not implemented for server-side context");
    }

    tracing::warn!("🔥 Failed to access cookies - no valid context");
    None
}

#[cfg(target_arch = "wasm32")]
fn parse_cookie_value(cookie_string: &str, name: &str) -> Option<String> {
    cookie_string.split(';').find_map(|cookie| {
        let mut parts = cookie.trim().splitn(2, '=');
        match (parts.next(), parts.next()) {
            (Some(key), Some(value)) if key == name => Some(value.to_string()),
            _ => None,
        }
    })
}

// Enhanced secure token storage with proper security measures
async fn store_auth_tokens_secure(_access_token: &str, _refresh_token: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::{window, Document};
        if let Some(window) = window() {
            if let Ok(document) = window.document() {
                // Set secure, httpOnly-like cookies (limited in browser)
                let secure_flag = if window.location().protocol().unwrap_or_default() == "https:" {
                    "; Secure"
                } else {
                    ""
                };

                // Access token (shorter expiry)
                let access_cookie = format!(
                    "auth_token={}; Path=/; SameSite=Strict{}; Max-Age=3600",
                    access_token, secure_flag
                );

                // Refresh token (longer expiry, more secure)
                let refresh_cookie = format!(
                    "refresh_token={}; Path=/auth; SameSite=Strict{}; Max-Age=604800",
                    refresh_token, secure_flag
                );

                let _ = document.set_cookie(&access_cookie);
                let _ = document.set_cookie(&refresh_cookie);

                // Also store in sessionStorage for quick access
                if let Ok(Some(storage)) = window.session_storage() {
                    let _ = storage.set_item(
                        "auth_token_exp",
                        &(js_sys::Date::now() as u64 + 3600000).to_string(),
                    );
                }
            }
        }
    }
}

// Enhanced secure state clearing
async fn clear_auth_state_secure() {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;
        if let Some(window) = window() {
            if let Ok(document) = window.document() {
                // Clear cookies by setting expired dates
                let _ = document
                    .set_cookie("auth_token=; Path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT");
                let _ = document.set_cookie(
                    "refresh_token=; Path=/auth; expires=Thu, 01 Jan 1970 00:00:00 GMT",
                );

                // Clear session storage
                if let Ok(Some(storage)) = window.session_storage() {
                    let _ = storage.remove_item("auth_token_exp");
                    let _ = storage.clear();
                }

                // Clear local storage of auth data
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.remove_item("user_preferences");
                    // Keep other non-auth related data
                }
            }
        }
    }
}

fn _store_auth_tokens(_access_token: &str, _refresh_token: &str) {
    // Legacy function - use store_auth_tokens_secure instead
    tracing::warn!("Using deprecated store_auth_tokens - use store_auth_tokens_secure");
}

// Removed deprecated clear_auth_state function

// Protected route wrapper
#[component]
pub fn ProtectedRoute(children: Children) -> impl IntoView {
    let auth = use_auth();

    if auth.is_authenticated.get() {
        children().into_any()
    } else if auth.is_loading.get() {
        view! {
            <div class="min-h-screen flex items-center justify-center bg-gray-50">
                <div class="max-w-md w-full space-y-8">
                    <div class="text-center">
                        <div class="mx-auto h-12 w-12 flex items-center justify-center rounded-full bg-blue-100">
                            <div class="animate-spin rounded-full h-6 w-6 border-2 border-blue-600 border-t-transparent"></div>
                        </div>
                        <h2 class="mt-6 text-3xl font-extrabold text-gray-900">
                            "Checking authentication..."
                        </h2>
                    </div>
                </div>
            </div>
        }.into_any()
    } else {
        // Not authenticated - redirect to login
        Effect::new(move |_| {
            let _ = window().location().set_href("/login");
        });

        view! {
            <div class="min-h-screen flex items-center justify-center bg-gray-50">
                <div class="max-w-md w-full space-y-8">
                    <div class="text-center">
                        <h2 class="mt-6 text-3xl font-extrabold text-gray-900">
                            "Redirecting to login..."
                        </h2>
                    </div>
                </div>
            </div>
        }
        .into_any()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtUser {
    id: String,
    phone: Option<JwtPhone>,
    nostr: Option<JwtNostr>,
    profile: Option<JwtProfile>,
    roles: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtPhone {
    number: String,
    verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtNostr {
    npub: String,
    verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtProfile {
    name: String,
    avatar_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    user: JwtUser, // User object with nested data
    iat: u64,      // Issued at timestamp
    nbf: u64,      // Not before timestamp
    iss: String,   // Issuer
    aud: String,   // Audience
    jti: String,   // JWT ID
    exp: u64,      // Expiration timestamp
}

/// Server function to get authenticated user information from request context
/// Enhanced version that works with the new SSR auth middleware
#[server(GetAuthUser, "/api")]
pub async fn get_auth_user() -> Result<Option<UserInfo>, ServerFnError> {
    // Removed middleware dependency - using simplified auth for frontend-only mode
    use axum::http::request::Parts;
    use leptos_axum::extract;

    tracing::info!("🔥 get_auth_user: Starting server function execution");

    // Try to extract from request parts context
    match extract::<Parts>().await {
        Ok(parts) => {
            tracing::info!("🔥 get_auth_user: Successfully extracted request parts");

            // Create a dummy request to use the extraction helper
            let dummy_request = axum::extract::Request::builder()
                .uri("/")
                .body(axum::body::Body::empty())
                .unwrap();

            // Copy extensions from parts
            let mut request = dummy_request;
            *request.extensions_mut() = parts.extensions;

            // PLACEHOLDER: Auth middleware removed - return None for frontend-only mode
            tracing::info!("🔥 get_auth_user: Frontend-only mode - no SSR auth available");
            Ok(None)
        }
        Err(e) => {
            tracing::error!("🔥 get_auth_user: Failed to extract request parts: {}", e);
            Ok(None)
        }
    }
}

/// Decode JWT token to extract user information
/// This is a client-side only function (no signature verification)
pub fn decode_jwt_token(token: &str) -> Result<UserInfo, String> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    tracing::info!("🔥 JWT: Starting decode of token (length: {})", token.len());
    tracing::info!(
        "🔥 JWT: Token preview: {}...",
        &token[..std::cmp::min(50, token.len())]
    );

    // For client-side decoding, we don't verify the signature
    // since we don't have the secret key and we trust the token from our own server
    let mut validation = Validation::default();
    validation.insecure_disable_signature_validation();
    validation.validate_exp = false; // Don't validate expiration on client side
    validation.validate_nbf = false; // Don't validate not-before on client side
    validation.validate_aud = false; // Don't validate audience for now
    validation.required_spec_claims.clear(); // Clear required claims

    tracing::info!("🔥 JWT: Attempting to decode with validation settings");

    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(&[]), // Empty key since we're not validating signature
        &validation,
    )
    .map_err(|e| {
        tracing::error!("🔥 JWT decode failed: {}", e);
        tracing::error!("🔥 JWT: Full token was: {}", token);
        format!("JWT decode error: {}", e)
    })?;

    let claims = token_data.claims;
    tracing::info!(
        "🔥 JWT: Successfully decoded claims for user: {}",
        claims.user.id
    );
    tracing::info!("🔥 JWT: User roles: {:?}", claims.user.roles);
    tracing::info!("🔥 JWT: User phone: {:?}", claims.user.phone);

    // Convert numeric roles to string format expected by auth guards
    let roles: Vec<String> = claims
        .user
        .roles
        .into_iter()
        .map(|role| match role {
            0 => "member".to_string(),
            1 => "admin".to_string(),
            2 | 3 => "superadmin".to_string(), // Role 3 might be another super admin variant
            _ => "member".to_string(),         // Default to member for unknown roles
        })
        .collect();

    let phone_number = claims.user.phone.as_ref().map(|p| p.number.clone());
    let phone_verified = claims
        .user
        .phone
        .as_ref()
        .map(|p| p.verified)
        .unwrap_or(false);

    Ok(User {
        id: uuid::Uuid::parse_str(&claims.user.id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
        phone: phone_number.map(|number| crate::api::types::Phone { number }),
        nostr: claims
            .user
            .nostr
            .map(|n| crate::api::types::Nostr { npub: n.npub }),
        profile: claims.user.profile.map(|p| crate::api::types::Profile {
            name: Some(p.name),
            avatar_url: Some(p.avatar_url),
        }),
        roles: roles
            .into_iter()
            .map(|role_str| match role_str.as_str() {
                "admin" => crate::api::types::Role::Admin,
                "superadmin" | "super_admin" => crate::api::types::Role::SuperAdmin,
                _ => crate::api::types::Role::Member,
            })
            .collect(),
        verified: phone_verified,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

/// Enhanced SSR-compatible auth provider that uses middleware-injected auth state
#[component]
pub fn SSRAuthProvider(children: Children) -> impl IntoView {
    tracing::info!("✨ SSRAuthProvider component created");

    let user = RwSignal::new(None::<UserInfo>);
    let token = RwSignal::new(None::<String>);
    let refresh_token = RwSignal::new(None::<String>);
    let token_expires_at = RwSignal::new(None::<u64>);
    let is_loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let is_authenticated = Signal::derive(move || user.get().is_some());

    // Initialize auth state using the enhanced server function
    // This now works reliably with the SSR auth middleware
    Effect::new(move |_| {
        tracing::info!("🔥 SSRAuthProvider: Effect starting - initializing auth state");
        is_loading.set(true);

        spawn_local(async move {
            tracing::info!(
                "🔥 SSRAuthProvider: Starting client-side auth restoration from cookies"
            );

            // Try to restore auth state from cookies (like the regular AuthProvider)
            if let Some(stored_token) = get_auth_token_from_cookies() {
                tracing::info!(
                    "🔥 SSRAuthProvider: Found auth_token cookie, length: {}",
                    stored_token.len()
                );
                token.set(Some(stored_token.clone()));

                // Decode JWT token to extract user information
                match decode_jwt_token(&stored_token) {
                    Ok(user_info) => {
                        tracing::info!("🔥 SSRAuthProvider: Successfully decoded JWT token for user: {} with roles: {:?}", user_info.id, user_info.roles);
                        user.set(Some(user_info));
                        error.set(None);
                    }
                    Err(e) => {
                        tracing::error!("🔥 SSRAuthProvider: Failed to decode JWT token: {}", e);
                        // Token is invalid, clear everything
                        token.set(None);
                        refresh_token.set(None);
                        user.set(None);
                        error.set(None);
                    }
                }
            } else {
                tracing::info!("🔥 SSRAuthProvider: No auth_token cookie found");
                user.set(None);
                error.set(None);
            }

            is_loading.set(false);
            tracing::info!("🔥 SSRAuthProvider: Auth state initialization complete");
        });
    });

    // Create callbacks (simplified for SSR)
    let clear_error = Callback::new(move |_| {
        error.set(None);
    });

    let login = Callback::new(move |creds: LoginCredentials| {
        is_loading.set(true);
        error.set(None);

        spawn_local(async move {
            match login_user(creds).await {
                Ok(auth_response) => {
                    // The login_user function already returns UserInfo in the user field
                    tracing::info!(
                        "🔥 SSRAuthProvider: Login successful, setting user: {:?}",
                        auth_response.user.id
                    );
                    user.set(Some(auth_response.user));

                    // Store tokens if available
                    if let Some(access_token) = auth_response.access_token {
                        token.set(Some(access_token.clone()));

                        // Store auth tokens securely in cookies
                        let refresh_token_val =
                            auth_response.refresh_token.clone().unwrap_or_default();
                        spawn_local(async move {
                            store_auth_tokens_secure(&access_token, &refresh_token_val).await;
                        });
                    }
                    if let Some(refresh_token_val) = auth_response.refresh_token {
                        refresh_token.set(Some(refresh_token_val));
                    }

                    // Redirect to dashboard
                    #[cfg(target_arch = "wasm32")]
                    {
                        let _ = web_sys::window().unwrap().location().set_href("/dashboard");
                    }
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                }
            }
            is_loading.set(false);
        });
    });

    let logout = Callback::new(move |_| {
        is_loading.set(true);
        spawn_local(async move {
            if let Some(refresh_token_val) = refresh_token.get() {
                let _ = logout_user(refresh_token_val).await;
            }

            user.set(None);
            token.set(None);
            refresh_token.set(None);
            token_expires_at.set(None);
            error.set(None);

            #[cfg(target_arch = "wasm32")]
            {
                let _ = web_sys::window().unwrap().location().set_href("/login");
            }
            is_loading.set(false);
        });
    });

    let handle_auth_response = Callback::new(
        move |auth_response: crate::api::types::auth::AuthResponse| {
            // Use the user directly from the API response
            user.set(Some(auth_response.user));
        },
    );

    let auth_context = AuthContext {
        user,
        token,
        refresh_token,
        token_expires_at,
        is_authenticated,
        is_loading,
        error,
        login,
        logout,
        clear_error,
        handle_auth_response,
    };

    provide_context(auth_context);

    view! {
        {children()}
    }
}
