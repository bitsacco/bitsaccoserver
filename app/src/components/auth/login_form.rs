use crate::contexts::auth::use_auth;
use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use web_sys::window;

// Only phone+pin authentication is supported

#[server(NestJsLoginAction, "/api")]
pub async fn nestjs_login_action(phone: String, pin: String) -> Result<String, ServerFnError> {
    tracing::info!(
        "🔥 NestJS login action called with phone: {}, pin: {}",
        phone,
        pin
    );

    use crate::api::backends::nestjs::auth::NestJsAuthApi;
    use crate::api::backends::nestjs::client::NestJsClient;
    use crate::api::config::ApiConfig;
    use crate::api::traits::AuthApi;
    use crate::api::types::auth::LoginRequest;
    use http::{header::SET_COOKIE, HeaderValue};
    use leptos::prelude::*;
    use leptos_axum::redirect;

    let config = ApiConfig::from_env();
    let nestjs_client = NestJsClient::new(&config).map_err(|e| {
        tracing::error!("Failed to create NestJS client: {:?}", e);
        ServerFnError::new(format!("Failed to create API client: {}", e))
    })?;
    let auth_api = NestJsAuthApi::new(nestjs_client);

    let login_request = LoginRequest {
        pin,
        phone: Some(phone.clone()),
        npub: None,
    };

    match auth_api.login(login_request).await {
        Ok(auth_response) => {
            tracing::info!(
                "🔥 NestJS login successful: user_id={}, authenticated={}",
                auth_response.user.id,
                auth_response.authenticated
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

#[component]
pub fn EnhancedLoginForm() -> impl IntoView {
    let auth = use_auth();
    let nestjs_action = ServerAction::<NestJsLoginAction>::new();

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
    let (show_pin, set_show_pin) = signal(false);

    // If already authenticated, redirect to groups
    Effect::new(move |_| {
        if auth.is_authenticated.get() {
            if let Some(window) = window() {
                let _ = window.location().set_href("/dashboard");
            }
        }
    });

    view! {
        <div class="w-full max-w-md mx-auto">
            <ActionForm action=nestjs_action attr:class="space-y-4">
                // Phone number field
                <div>
                    <label
                        for="phone"
                        class="block text-sm font-medium font-body text-gray-700 mb-2"
                    >
                        "Phone Number"
                    </label>
                    <input
                        id="phone"
                        name="phone"
                        type="tel"
                        required=true
                        class="block w-full px-4 py-3 border border-gray-300 rounded-xl shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-teal-500 focus:border-teal-500 transition-all duration-200 font-body bg-gray-50/50 hover:bg-white"
                        placeholder="Enter your phone number"
                    />
                </div>

                // PIN field with toggle visibility
                <div class="relative">
                    <label
                        for="pin"
                        class="block text-sm font-medium font-body text-gray-700 mb-2"
                    >
                        "PIN Code"
                    </label>
                    <input
                        id="pin"
                        name="pin"
                        type=move || if show_pin.get() { "text" } else { "password" }
                        required=true
                        maxlength="6"
                        class="block w-full px-4 py-3 pr-12 border border-gray-300 rounded-xl shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-teal-500 focus:border-teal-500 transition-all duration-200 font-body bg-gray-50/50 hover:bg-white"
                        placeholder="Enter your 6-digit PIN"
                    />
                    // Toggle PIN visibility button
                    <button
                        type="button"
                        class="absolute inset-y-0 right-0 pr-4 flex items-center top-8 hover:bg-gray-100 rounded-r-xl transition-colors"
                        on:click=move |_| set_show_pin.update(|show| *show = !*show)
                    >
                        <svg class="h-5 w-5 text-gray-400 hover:text-teal-600 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            {if show_pin.get() {
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

                // Submit button
                <div>
                    <button
                        type="submit"
                        disabled=move || nestjs_action.pending().get()
                        class=move || {
                            let base_class = "w-full flex justify-center py-3 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2";
                            if nestjs_action.pending().get() {
                                format!("{} text-white bg-gray-400 cursor-not-allowed", base_class)
                            } else {
                                format!("{} text-white bg-teal-600 hover:bg-teal-700 focus:ring-teal-500", base_class)
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
        </div>
    }
}
