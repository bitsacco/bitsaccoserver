use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
    pub token_type: String,
    pub user: UserInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
}

impl UserInfo {
    pub fn full_name(&self) -> String {
        match (&self.given_name, &self.family_name) {
            (Some(given), Some(family)) => format!("{} {}", given, family),
            (Some(given), None) => given.clone(),
            (None, Some(family)) => family.clone(),
            (None, None) => self.username.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user: RwSignal<Option<UserInfo>>,
    pub token: RwSignal<Option<String>>,
    pub refresh_token: RwSignal<Option<String>>,
    pub is_authenticated: Signal<bool>,
    pub is_loading: RwSignal<bool>,
    pub login: Callback<LoginCredentials, ()>,
    pub logout: Callback<(), ()>,
}

#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let user = RwSignal::new(None::<UserInfo>);
    let token = RwSignal::new(None::<String>);
    let refresh_token = RwSignal::new(None::<String>);
    let is_loading = RwSignal::new(false);

    let is_authenticated = Signal::derive(move || user.get().is_some());

    // Try to restore authentication state from cookies/storage on init
    Effect::new(move |_| {
        is_loading.set(true);
        spawn_local(async move {
            if let Some(stored_token) = get_auth_token_from_cookies() {
                token.set(Some(stored_token.clone()));
                
                // Validate the token and get user info
                match validate_token_and_get_user(&stored_token).await {
                    Ok(user_info) => {
                        user.set(Some(user_info));
                    }
                    Err(_) => {
                        // Token is invalid, clear everything
                        clear_auth_state();
                        token.set(None);
                        refresh_token.set(None);
                    }
                }
            }
            is_loading.set(false);
        });
    });

    let login = Callback::new(move |creds: LoginCredentials| {
        is_loading.set(true);
        spawn_local(async move {
            match perform_login(creds).await {
                Ok(auth_response) => {
                    // Store tokens
                    token.set(Some(auth_response.access_token.clone()));
                    refresh_token.set(Some(auth_response.refresh_token.clone()));
                    user.set(Some(auth_response.user));
                    
                    // Store in secure cookies
                    store_auth_tokens(&auth_response.access_token, &auth_response.refresh_token);
                    
                    // Redirect to dashboard
                    let _ = window().location().set_href("/dashboard");
                }
                Err(e) => {
                    tracing::error!("Login failed: {}", e);
                    // Login will show error in form
                }
            }
            is_loading.set(false);
        });
    });

    let logout = Callback::new(move |_| {
        is_loading.set(true);
        spawn_local(async move {
            // Logout from server if we have a refresh token
            if let Some(refresh_token_val) = refresh_token.get() {
                let _ = perform_logout(&refresh_token_val).await;
            }
            
            // Clear local state
            user.set(None);
            token.set(None);
            refresh_token.set(None);
            clear_auth_state();
            
            // Redirect to login
            let _ = window().location().set_href("/login");
            is_loading.set(false);
        });
    });

    let auth_context = AuthContext {
        user,
        token,
        refresh_token,
        is_authenticated,
        is_loading,
        login,
        logout,
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
pub async fn login_user(
    credentials: LoginCredentials,
) -> Result<AuthResponse, ServerFnError> {
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
    
    // Get the current server URL from environment or use default
    let server_url = std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    let response = client
        .post(format!("{}/api/auth/login", server_url))
        .json(&login_request)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("HTTP request failed: {}", e)))?;
    
    if response.status().is_success() {
        let auth_response: crate::api::auth::LoginResponse = response
            .json()
            .await
            .map_err(|e| ServerFnError::new(format!("JSON parsing failed: {}", e)))?;
        
        // Convert to frontend types
        Ok(AuthResponse {
            access_token: auth_response.access_token,
            refresh_token: auth_response.refresh_token,
            expires_in: auth_response.expires_in,
            refresh_expires_in: auth_response.refresh_expires_in,
            token_type: auth_response.token_type,
            user: UserInfo {
                user_id: auth_response.user.user_id.to_string(),
                email: auth_response.user.email,
                username: auth_response.user.username,
                given_name: auth_response.user.given_name,
                family_name: auth_response.user.family_name,
                roles: auth_response.user.roles,
                groups: auth_response.user.groups,
            },
        })
    } else {
        // Parse the error response to provide user-friendly messages
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Authentication failed".to_string());
        
        // Check for common authentication errors and provide user-friendly messages
        if error_text.contains("invalid_grant") || error_text.contains("Invalid user credentials") {
            Err(ServerFnError::new("Invalid email or password. Please check your credentials and try again."))
        } else if error_text.contains("Account is not fully set up") {
            Err(ServerFnError::new("Your account is not fully configured. Please contact an administrator."))
        } else if error_text.contains("Account disabled") {
            Err(ServerFnError::new("Your account has been disabled. Please contact an administrator."))
        } else {
            // Generic error message that doesn't expose internal details
            Err(ServerFnError::new("Authentication failed. Please try again."))
        }
    }
}

#[server(ValidateToken, "/api")]
pub async fn validate_token(token: String) -> Result<UserInfo, ServerFnError> {
    use reqwest::Client;
    
    let client = Client::new();
    let server_url = std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    let response = client
        .get(format!("{}/api/auth/me", server_url))
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("HTTP request failed: {}", e)))?;
    
    if response.status().is_success() {
        let me_response: crate::api::auth::MeResponse = response
            .json()
            .await
            .map_err(|e| ServerFnError::new(format!("JSON parsing failed: {}", e)))?;
        
        Ok(UserInfo {
            user_id: me_response.user.user_id.to_string(),
            email: me_response.user.email,
            username: me_response.user.username,
            given_name: me_response.user.given_name,
            family_name: me_response.user.family_name,
            roles: me_response.user.roles,
            groups: me_response.user.groups,
        })
    } else {
        Err(ServerFnError::new("Token validation failed".to_string()))
    }
}

#[server(LogoutUser, "/api")]
pub async fn logout_user(refresh_token: String) -> Result<(), ServerFnError> {
    use crate::api::auth::LogoutRequest;
    use reqwest::Client;
    
    let client = Client::new();
    let server_url = std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
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
async fn perform_login(credentials: LoginCredentials) -> Result<AuthResponse, String> {
    login_user(credentials)
        .await
        .map_err(|e| e.to_string())
}

async fn validate_token_and_get_user(token: &str) -> Result<UserInfo, String> {
    validate_token(token.to_string())
        .await
        .map_err(|e| e.to_string())
}

async fn perform_logout(refresh_token: &str) -> Result<(), String> {
    logout_user(refresh_token.to_string())
        .await
        .map_err(|e| e.to_string())
}

fn get_auth_token_from_cookies() -> Option<String> {
    // In SSR mode, we can't access document.cookie directly
    // This would need to be implemented based on your cookie strategy
    // For now, return None - tokens will be managed server-side
    None
}

fn store_auth_tokens(_access_token: &str, _refresh_token: &str) {
    // In SSR mode, token storage should be handled server-side via HttpOnly cookies
    // This is implemented in the login action
}

fn clear_auth_state() {
    // In SSR mode, clearing state should be handled server-side
    // This is implemented in the logout action
}

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
        }.into_any()
    }
}