use crate::contexts::auth::{use_auth, LoginCredentials};
use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use web_sys::window;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuthMethod {
    Email,
    Phone,
    Pin,
    Nostr,
}

impl AuthMethod {
    pub fn display_name(&self) -> &'static str {
        match self {
            AuthMethod::Email => "Email",
            AuthMethod::Phone => "Phone",
            AuthMethod::Pin => "PIN",
            AuthMethod::Nostr => "Nostr",
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

    pub fn placeholder(&self) -> &'static str {
        match self {
            AuthMethod::Email => "Enter your email",
            AuthMethod::Phone => "Enter your phone number",
            AuthMethod::Pin => "Enter your PIN",
            AuthMethod::Nostr => "Enter your Nostr public key",
        }
    }
}

#[server(GetBackendConfig, "/api")]
pub async fn get_backend_config() -> Result<String, ServerFnError> {
    // Check if we're using NestJS backend
    let backend = std::env::var("API_BACKEND").unwrap_or_else(|_| "rust".to_string());
    Ok(backend)
}

#[server(NestJsLoginAction, "/api")]
pub async fn nestjs_login_action(phone: String, pin: String) -> Result<String, ServerFnError> {
    tracing::info!(
        "🔥 NestJS login action called with phone: {}, pin: {}",
        phone,
        pin
    );
    use crate::api::abstraction::AbstractedApiClient;
    use crate::api::config::{ApiConfig, Backend};
    use crate::api::types::LoginRequest as ApiLoginRequest;
    use http::{header::SET_COOKIE, HeaderValue};
    use leptos::prelude::*;
    use leptos_axum::redirect;

    // Create API config for NestJS backend
    let nestjs_url =
        std::env::var("NESTJS_API_URL").unwrap_or_else(|_| "http://localhost:4000/v1".to_string());
    let config = ApiConfig::new(Backend::NestJs, nestjs_url);
    let client = AbstractedApiClient::new(config)
        .map_err(|e| ServerFnError::new(format!("Failed to create API client: {}", e)))?;

    let login_request = ApiLoginRequest {
        pin,
        phone: Some(phone.clone()),
        npub: None,
    };

    match client.auth().login(login_request).await {
        Ok(auth_response) => {
            tracing::info!(
                "🔥 NestJS login successful: user_id={}, roles={:?}",
                auth_response.user.id,
                auth_response
                    .user
                    .roles
                    .iter()
                    .map(|r| r.to_auth_string())
                    .collect::<Vec<_>>()
            );

            // Store auth cookies - get response options once and use append_header for multiple cookies
            let response_options = use_context::<leptos_axum::ResponseOptions>()
                .expect("ResponseOptions should be available");

            // Store auth token in cookie for auth context to pick up (NOT HttpOnly so client can read it)
            if let Some(access_token) = &auth_response.access_token {
                response_options.append_header(
                    SET_COOKIE,
                    HeaderValue::from_str(&format!(
                        "auth_token={}; Path=/; SameSite=Strict; Max-Age=3600",
                        access_token
                    ))
                    .unwrap(),
                );
                tracing::info!("🔥 Stored auth_token cookie (readable by client)");
            }

            // Store refresh token if available
            if let Some(refresh_token) = &auth_response.refresh_token {
                response_options.append_header(
                    SET_COOKIE,
                    HeaderValue::from_str(&format!(
                        "refresh_token={}; Path=/; SameSite=Strict; Max-Age=604800; HttpOnly",
                        refresh_token
                    ))
                    .unwrap(),
                );
                tracing::info!("🔥 Stored refresh_token cookie");
            }

            // Redirect to dashboard after successful login
            redirect("/dashboard");
            Ok("Login successful".to_string())
        }
        Err(e) => {
            tracing::error!("🔥 NestJS login failed: {}", e);
            Err(ServerFnError::new(format!("Login failed: {}", e)))
        }
    }
}

#[server(EnhancedLoginAction, "/api")]
pub async fn enhanced_login_action(
    method: String,
    identifier: String,
    credential: String,
) -> Result<String, ServerFnError> {
    use crate::contexts::auth::login_user;

    // Parse auth method
    let _auth_method = match method.as_str() {
        "email" => AuthMethod::Email,
        "phone" => AuthMethod::Phone,
        "pin" => AuthMethod::Pin,
        "nostr" => AuthMethod::Nostr,
        _ => return Err(ServerFnError::new("Invalid authentication method")),
    };

    // For backward compatibility, convert to LoginCredentials
    // In a full implementation, this would use the enhanced LoginRequest
    let login_creds = LoginCredentials {
        email: identifier.clone(),
        password: credential.clone(),
    };

    // Call the authentication service
    let _auth_response = login_user(login_creds).await?;

    // Authentication successful - no extended session needed

    // Return success without server-side redirect
    // The client will handle auth context and redirect
    Ok("Login successful".to_string())
}

#[component]
pub fn EnhancedLoginForm() -> impl IntoView {
    let auth = use_auth();
    let nestjs_action = ServerAction::<NestJsLoginAction>::new();
    let regular_action = ServerAction::<EnhancedLoginAction>::new();

    // Handle successful NestJS authentication
    Effect::new(move |_| {
        let action_value = nestjs_action.value().get();
        tracing::info!("NestJS action value changed: {:?}", action_value);

        if let Some(result) = action_value.as_ref() {
            match result {
                Ok(auth_response_json) => {
                    tracing::info!("NestJS login successful, response: {}", auth_response_json);
                    // Parse the auth response from the server action
                    match serde_json::from_str::<crate::api::types::auth::AuthResponse>(
                        auth_response_json,
                    ) {
                        Ok(auth_response) => {
                            tracing::info!(
                                "Successfully parsed auth response, updating auth context"
                            );
                            // Update auth context with the response
                            auth.handle_auth_response.run(auth_response);
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse auth response: {}", e);
                            tracing::error!("Raw response was: {}", auth_response_json);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("NestJS login failed: {}", e);
                }
            }
        }
    });

    // Form state
    let (selected_method, set_selected_method) = signal(AuthMethod::Phone);
    let (show_password, set_show_password) = signal(false);

    // Use LocalResource to avoid hydration issues
    let backend_config = LocalResource::new(|| async move {
        get_backend_config()
            .await
            .unwrap_or_else(|_| "nestjs".to_string())
    });

    // Check if we're using NestJS backend (phone + PIN flow)
    let is_nestjs_backend = move || {
        backend_config
            .get()
            .map(|config| config == "nestjs")
            .unwrap_or(true)
    };

    // Available auth methods based on backend
    let auth_methods = move || {
        if is_nestjs_backend() {
            // For NestJS backend, only show Phone (which includes PIN)
            vec![AuthMethod::Phone]
        } else {
            // For other backends, show all options
            vec![
                AuthMethod::Email,
                AuthMethod::Phone,
                AuthMethod::Pin,
                AuthMethod::Nostr,
            ]
        }
    };

    // If already authenticated, redirect to groups
    Effect::new(move |_| {
        if auth.is_authenticated.get() {
            if let Some(window) = window() {
                let _ = window.location().set_href("/dashboard");
            }
        }
    });

    // Set initial selected method based on backend
    Effect::new(move |_| {
        if let Some(backend) = backend_config.get() {
            if backend == "nestjs" {
                set_selected_method.set(AuthMethod::Phone);
            }
        }
    });

    view! {
        <div class="w-full max-w-md mx-auto">
            // Auth Method Selection (only show if not NestJS backend)
            <Show when=move || !is_nestjs_backend()>
                <div class="mb-6">
                    <div class="text-sm font-medium text-gray-700 mb-3">
                        "Sign in with:"
                    </div>
                    <div class="grid grid-cols-2 gap-3">
                        <For
                            each=auth_methods
                            key=|method| format!("{:?}", method)
                            children=move |method: AuthMethod| {
                                let is_selected = move || selected_method.get() == method;
                                let method_clone = method;

                                view! {
                                    <button
                                        type="button"
                                        class=move || format!(
                                            "px-3 py-2 text-sm font-medium rounded-lg border transition-colors {}",
                                            if is_selected() {
                                                "bg-blue-50 border-blue-500 text-blue-700"
                                            } else {
                                                "bg-white border-gray-300 text-gray-700 hover:bg-gray-50"
                                            }
                                        )
                                        on:click=move |_| set_selected_method.set(method_clone)
                                    >
                                        {method.display_name()}
                                    </button>
                                }
                            }
                        />
                    </div>
                </div>
            </Show>

            // Forms - Use ActionForm for proper form handling
            <Show
                when=is_nestjs_backend
                fallback=move || view! {
                    // Regular backend ActionForm
                    <ActionForm action=regular_action attr:class="space-y-4">
                        <input type="hidden" name="method" value=move || format!("{:?}", selected_method.get()).to_lowercase() />
                        // Regular backend identifier field
                        <div>
                            <label
                                for="identifier"
                                class="block text-sm font-medium text-gray-700 mb-2"
                            >
                                {move || selected_method.get().display_name()}
                            </label>
                            <input
                                id="identifier"
                                name="identifier"
                                type=move || selected_method.get().input_type()
                                required=true
                                class="block w-full px-3 py-3 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors"
                                placeholder=move || selected_method.get().placeholder()
                            />
                        </div>

                        // Regular backend credential field
                        <div class="relative">
                            <label
                                for="credential"
                                class="block text-sm font-medium text-gray-700 mb-2"
                            >
                                {move || match selected_method.get() {
                                    AuthMethod::Email => "Password",
                                    AuthMethod::Phone => "Password",
                                    AuthMethod::Pin => "PIN",
                                    AuthMethod::Nostr => "Private Key",
                                }}
                            </label>
                            <input
                                id="credential"
                                name="credential"
                                type=move || if show_password.get() { "text" } else { "password" }
                                required=true
                                class="block w-full px-3 py-3 pr-10 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors"
                                placeholder=move || match selected_method.get() {
                                    AuthMethod::Email => "Enter your password",
                                    AuthMethod::Phone => "Enter your password",
                                    AuthMethod::Pin => "Enter your PIN",
                                    AuthMethod::Nostr => "Enter your private key",
                                }
                            />
                            // Toggle visibility button
                            <button
                                type="button"
                                class="absolute inset-y-0 right-0 pr-3 flex items-center top-8"
                                on:click=move |_| set_show_password.update(|show| *show = !*show)
                            >
                                <svg class="h-5 w-5 text-gray-400 hover:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    {if show_password.get() {
                                        view! {
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L3 3m6.878 6.878L21 21" />
                                        }.into_any()
                                    } else {
                                        view! {
                                            <>
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                                            </>
                                        }.into_any()
                                    }}
                                </svg>
                            </button>
                        </div>

                        // Submit button for regular backend
                        <div>
                            <button
                                type="submit"
                                disabled=move || regular_action.pending().get()
                                class=move || {
                                    let base_class = "w-full flex justify-center py-3 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2";
                                    if regular_action.pending().get() {
                                        format!("{} text-white bg-gray-400 cursor-not-allowed", base_class)
                                    } else {
                                        format!("{} text-white bg-blue-600 hover:bg-blue-700 focus:ring-blue-500", base_class)
                                    }
                                }
                            >
                                <Show
                                    when=move || regular_action.pending().get()
                                    fallback=|| "Sign in"
                                >
                                    <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
                                    </svg>
                                    "Signing in..."
                                </Show>
                            </button>
                        </div>
                    </ActionForm>
                }
            >
                // NestJS ActionForm
                <ActionForm action=nestjs_action attr:class="space-y-4">
                    // Phone number field for NestJS
                    <div>
                        <label
                            for="phone"
                            class="block text-sm font-medium text-gray-700 mb-2"
                        >
                            "Phone Number"
                        </label>
                        <input
                            id="phone"
                            name="phone"
                            type="tel"
                            required=true
                            class="block w-full px-3 py-3 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors"
                            placeholder="Enter your phone number"
                        />
                    </div>

                    // PIN field for NestJS
                    <div>
                        <label
                            for="pin"
                            class="block text-sm font-medium text-gray-700 mb-2"
                        >
                            "PIN Code"
                        </label>
                        <input
                            id="pin"
                            name="pin"
                            type="password"
                            required=true
                            maxlength="6"
                            class="block w-full px-3 py-3 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors"
                            placeholder="Enter your 6-digit PIN"
                        />
                    </div>

                    // Submit button for NestJS
                    <div>
                        <button
                            type="submit"
                            disabled=move || nestjs_action.pending().get()
                            class=move || {
                                let base_class = "w-full flex justify-center py-3 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2";
                                if nestjs_action.pending().get() {
                                    format!("{} text-white bg-gray-400 cursor-not-allowed", base_class)
                                } else {
                                    format!("{} text-white bg-blue-600 hover:bg-blue-700 focus:ring-blue-500", base_class)
                                }
                            }
                        >
                            <Show
                                when=move || nestjs_action.pending().get()
                                fallback=|| "Sign in"
                            >
                                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
                                </svg>
                                "Signing in..."
                            </Show>
                        </button>
                    </div>
                </ActionForm>
            </Show>
        </div>
    }
}
