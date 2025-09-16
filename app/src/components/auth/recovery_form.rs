use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use url;
use wasm_bindgen::JsCast;

#[server(RequestPasswordReset, "/api")]
pub async fn request_password_reset(email: String) -> Result<String, ServerFnError> {
    // Basic email validation
    if email.trim().is_empty() || !email.contains('@') {
        return Err(ServerFnError::new("Please enter a valid email address"));
    }

    // TODO: Implement actual password reset using the API client
    // This would typically:
    // 1. Check if user exists
    // 2. Generate a secure reset token
    // 3. Send email with reset link
    // 4. Store token for later verification

    tracing::info!("Password reset requested for email: {}", email);

    Ok(format!(
        "If an account with {} exists, you will receive a password reset link shortly.",
        email
    ))
}

#[server(ResetPassword, "/api")]
pub async fn reset_password(
    token: String,
    new_password: String,
    confirm_password: String,
) -> Result<String, ServerFnError> {
    // Validation
    if new_password != confirm_password {
        return Err(ServerFnError::new("Passwords do not match"));
    }

    if new_password.len() < 8 {
        return Err(ServerFnError::new("Password must be at least 8 characters"));
    }

    if token.trim().is_empty() {
        return Err(ServerFnError::new("Invalid reset token"));
    }

    // TODO: Implement actual password reset using the API client
    // This would typically:
    // 1. Validate the reset token
    // 2. Check token expiration
    // 3. Update user's password
    // 4. Invalidate the reset token
    // 5. Optionally log out all sessions

    tracing::info!("Password reset completed for token: {}", token);

    Ok(
        "Your password has been successfully reset. You can now sign in with your new password."
            .to_string(),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub enum RecoveryStep {
    RequestReset,
    ResetPassword,
    Success,
}

#[component]
pub fn PasswordRecoveryForm() -> impl IntoView {
    let (current_step, set_current_step) = signal(RecoveryStep::RequestReset);
    let (email, set_email) = signal(String::new());
    let (token, set_token) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (show_password, set_show_password) = signal(false);
    let (show_confirm_password, set_show_confirm_password) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (success, set_success) = signal(Option::<String>::None);
    let (_is_loading, _set_is_loading) = signal(false);

    // Server actions
    let reset_request_action = ServerAction::<RequestPasswordReset>::new();
    let reset_password_action = ServerAction::<ResetPassword>::new();

    // Check for reset token in URL params
    Effect::new(move |_| {
        if let Ok(window) = window().dyn_into::<web_sys::Window>() {
            if let Ok(location) = window.location().href() {
                if let Ok(url) = url::Url::parse(&location) {
                    if let Some(url_token) = url
                        .query_pairs()
                        .find(|(key, _)| key == "token")
                        .map(|(_, value)| value.to_string())
                    {
                        set_token.set(url_token);
                        set_current_step.set(RecoveryStep::ResetPassword);
                    }
                }
            }
        }
    });

    // Watch for reset request results
    Effect::new(move |_| match reset_request_action.value().get() {
        Some(Ok(message)) => {
            set_error.set(None);
            set_success.set(Some(message));
            set_current_step.set(RecoveryStep::Success);
        }
        Some(Err(e)) => {
            set_error.set(Some(e.to_string()));
            set_success.set(None);
        }
        None => {}
    });

    // Watch for password reset results
    Effect::new(move |_| match reset_password_action.value().get() {
        Some(Ok(message)) => {
            set_error.set(None);
            set_success.set(Some(message));
            set_current_step.set(RecoveryStep::Success);
        }
        Some(Err(e)) => {
            set_error.set(Some(e.to_string()));
            set_success.set(None);
        }
        None => {}
    });

    // Form validation
    let email_valid = Signal::derive(move || {
        let email = email.get();
        !email.is_empty() && email.contains('@') && email.contains('.')
    });

    let passwords_valid = Signal::derive(move || {
        let new_pass = new_password.get();
        let confirm_pass = confirm_password.get();
        !new_pass.is_empty() && new_pass.len() >= 8 && new_pass == confirm_pass
    });

    let handle_reset_request = move |ev: SubmitEvent| {
        ev.prevent_default();

        if !email_valid.get() {
            set_error.set(Some("Please enter a valid email address".to_string()));
            return;
        }

        set_error.set(None);
        reset_request_action.dispatch(RequestPasswordReset { email: email.get() });
    };

    let handle_password_reset = move |ev: SubmitEvent| {
        ev.prevent_default();

        if token.get().trim().is_empty() {
            set_error.set(Some("Invalid or missing reset token".to_string()));
            return;
        }

        if !passwords_valid.get() {
            set_error.set(Some(
                "Please enter matching passwords (at least 8 characters)".to_string(),
            ));
            return;
        }

        set_error.set(None);
        reset_password_action.dispatch(ResetPassword {
            token: token.get(),
            new_password: new_password.get(),
            confirm_password: confirm_password.get(),
        });
    };

    view! {
        <div class="w-full max-w-md mx-auto">
            // Header
            <div class="text-center mb-8">
                <div class="w-12 h-12 mx-auto rounded-full bg-blue-100 flex items-center justify-center mb-4">
                    <svg class="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                    </svg>
                </div>
                <h2 class="text-2xl font-bold text-gray-900">
                    {move || match current_step.get() {
                        RecoveryStep::RequestReset => "Reset your password",
                        RecoveryStep::ResetPassword => "Create new password",
                        RecoveryStep::Success => "Password reset successful",
                    }}
                </h2>
                <p class="mt-2 text-sm text-gray-600">
                    {move || match current_step.get() {
                        RecoveryStep::RequestReset => "Enter your email address and we'll send you a link to reset your password.",
                        RecoveryStep::ResetPassword => "Enter your new password below.",
                        RecoveryStep::Success => "You can now sign in with your new password.",
                    }}
                </p>
            </div>

            // Step 1: Request Reset
            <Show when=move || current_step.get() == RecoveryStep::RequestReset>
                <form on:submit=handle_reset_request class="space-y-4">
                    <div>
                        <label for="email" class="block text-sm font-medium text-gray-700 mb-2">
                            "Email Address"
                        </label>
                        <input
                            id="email"
                            name="email"
                            type="email"
                            required=true
                            class=move || format!(
                                "block w-full px-3 py-3 border rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors {}",
                                if email.get().is_empty() {
                                    "border-gray-300"
                                } else if email_valid.get() {
                                    "border-green-300 bg-green-50"
                                } else {
                                    "border-red-300 bg-red-50"
                                }
                            )
                            placeholder="Enter your email address"
                            prop:value=email
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                        />
                        <Show when=move || !email.get().is_empty() && !email_valid.get()>
                            <p class="mt-1 text-sm text-red-600">
                                "Please enter a valid email address"
                            </p>
                        </Show>
                    </div>

                    <div>
                        <button
                            type="submit"
                            disabled=move || reset_request_action.pending().get() || !email_valid.get()
                            class="w-full flex justify-center py-3 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                        >
                            <Show
                                when=move || reset_request_action.pending().get()
                                fallback=|| "Send reset link"
                            >
                                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
                                </svg>
                                "Sending..."
                            </Show>
                        </button>
                    </div>
                </form>
            </Show>

            // Step 2: Reset Password
            <Show when=move || current_step.get() == RecoveryStep::ResetPassword>
                <form on:submit=handle_password_reset class="space-y-4">
                    <div class="relative">
                        <label for="new_password" class="block text-sm font-medium text-gray-700 mb-2">
                            "New Password"
                        </label>
                        <input
                            id="new_password"
                            name="new_password"
                            type=move || if show_password.get() { "text" } else { "password" }
                            required=true
                            class=move || format!(
                                "block w-full px-3 py-3 pr-10 border rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors {}",
                                if new_password.get().is_empty() {
                                    "border-gray-300"
                                } else if new_password.get().len() >= 8 {
                                    "border-green-300 bg-green-50"
                                } else {
                                    "border-red-300 bg-red-50"
                                }
                            )
                            placeholder="Enter your new password"
                            prop:value=new_password
                            on:input=move |ev| set_new_password.set(event_target_value(&ev))
                        />
                        <button
                            type="button"
                            class="absolute inset-y-0 right-0 pr-3 flex items-center top-8"
                            on:click=move |_| set_show_password.update(|show| *show = !*show)
                        >
                            <svg class="h-5 w-5 text-gray-400 hover:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                {move || if show_password.get() {
                                    view! {
                                        <g>
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 3l18 18" />
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.584 10.587a2 2 0 002.828 2.83M9.878 9.878l4.242 4.242M9.878 9.878L3 3m6.878 6.878L21 21" />
                                        </g>
                                    }
                                } else {
                                    view! {
                                        <g>
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                                        </g>
                                    }
                                }}
                            </svg>
                        </button>
                        <Show when=move || !new_password.get().is_empty() && new_password.get().len() < 8>
                            <p class="mt-1 text-sm text-red-600">
                                "Password must be at least 8 characters"
                            </p>
                        </Show>
                    </div>

                    <div class="relative">
                        <label for="confirm_password" class="block text-sm font-medium text-gray-700 mb-2">
                            "Confirm New Password"
                        </label>
                        <input
                            id="confirm_password"
                            name="confirm_password"
                            type=move || if show_confirm_password.get() { "text" } else { "password" }
                            required=true
                            class=move || format!(
                                "block w-full px-3 py-3 pr-10 border rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors {}",
                                if confirm_password.get().is_empty() {
                                    "border-gray-300"
                                } else if confirm_password.get() == new_password.get() && !new_password.get().is_empty() {
                                    "border-green-300 bg-green-50"
                                } else {
                                    "border-red-300 bg-red-50"
                                }
                            )
                            placeholder="Confirm your new password"
                            prop:value=confirm_password
                            on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                        />
                        <button
                            type="button"
                            class="absolute inset-y-0 right-0 pr-3 flex items-center top-8"
                            on:click=move |_| set_show_confirm_password.update(|show| *show = !*show)
                        >
                            <svg class="h-5 w-5 text-gray-400 hover:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                {move || if show_confirm_password.get() {
                                    view! {
                                        <g>
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 3l18 18" />
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.584 10.587a2 2 0 002.828 2.83M9.878 9.878l4.242 4.242M9.878 9.878L3 3m6.878 6.878L21 21" />
                                        </g>
                                    }
                                } else {
                                    view! {
                                        <g>
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                                        </g>
                                    }
                                }}
                            </svg>
                        </button>
                        <Show when=move || !confirm_password.get().is_empty() && confirm_password.get() != new_password.get()>
                            <p class="mt-1 text-sm text-red-600">
                                "Passwords do not match"
                            </p>
                        </Show>
                    </div>

                    <div>
                        <button
                            type="submit"
                            disabled=move || reset_password_action.pending().get() || !passwords_valid.get()
                            class="w-full flex justify-center py-3 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                        >
                            <Show
                                when=move || reset_password_action.pending().get()
                                fallback=|| "Reset password"
                            >
                                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
                                </svg>
                                "Resetting..."
                            </Show>
                        </button>
                    </div>
                </form>
            </Show>

            // Step 3: Success
            <Show when=move || current_step.get() == RecoveryStep::Success>
                <div class="text-center space-y-4">
                    <div class="w-16 h-16 mx-auto rounded-full bg-green-100 flex items-center justify-center">
                        <svg class="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                        </svg>
                    </div>

                    <div>
                        <a
                            href="/login"
                            class="w-full flex justify-center py-3 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
                        >
                            "Continue to Sign In"
                        </a>
                    </div>
                </div>
            </Show>

            // Error/Success Messages
            <Show when=move || error.get().is_some()>
                <div class="mt-4 rounded-md bg-red-50 p-4 animate-fade-in">
                    <div class="flex">
                        <div class="flex-shrink-0">
                            <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                            </svg>
                        </div>
                        <div class="ml-3">
                            <p class="text-sm font-medium text-red-800">
                                {move || error.get().unwrap_or_default()}
                            </p>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || success.get().is_some() && current_step.get() != RecoveryStep::Success>
                <div class="mt-4 rounded-md bg-blue-50 p-4 animate-fade-in">
                    <div class="flex">
                        <div class="flex-shrink-0">
                            <svg class="h-5 w-5 text-blue-400" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
                            </svg>
                        </div>
                        <div class="ml-3">
                            <p class="text-sm font-medium text-blue-800">
                                {move || success.get().unwrap_or_default()}
                            </p>
                        </div>
                    </div>
                </div>
            </Show>

            // Back to login link
            <Show when=move || current_step.get() != RecoveryStep::Success>
                <div class="mt-6 text-center">
                    <a href="/login" class="text-sm text-blue-600 hover:text-blue-500 transition-colors">
                        "Back to sign in"
                    </a>
                </div>
            </Show>
        </div>
    }
}
