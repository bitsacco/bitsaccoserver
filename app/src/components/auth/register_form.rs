use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub phone: Option<String>,
    pub username: String,
    pub given_name: String,
    pub family_name: String,
    pub password: String,
    pub confirm_password: String,
    pub accept_terms: bool,
}

#[server(RegisterAction, "/api")]
pub async fn register_action(
    email: String,
    _phone: Option<String>,
    username: String,
    _given_name: String,
    _family_name: String,
    password: String,
    confirm_password: String,
    accept_terms: bool,
) -> Result<String, ServerFnError> {
    // Validation
    if password != confirm_password {
        return Err(ServerFnError::new("Passwords do not match"));
    }

    if !accept_terms {
        return Err(ServerFnError::new("Please accept the terms and conditions"));
    }

    if password.len() < 8 {
        return Err(ServerFnError::new("Password must be at least 8 characters"));
    }

    // TODO: Implement actual registration using the API client
    // For now, return success
    tracing::info!("Registration requested for user: {} ({})", username, email);

    Ok("Registration successful! Please check your email for verification.".to_string())
}

#[component]
pub fn RegisterForm() -> impl IntoView {
    let register_action = ServerAction::<RegisterAction>::new();

    // Form state
    let (email, set_email) = signal(String::new());
    let (phone, set_phone) = signal(String::new());
    let (username, set_username) = signal(String::new());
    let (given_name, set_given_name) = signal(String::new());
    let (family_name, set_family_name) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (accept_terms, set_accept_terms) = signal(false);
    let (show_password, set_show_password) = signal(false);
    let (show_confirm_password, set_show_confirm_password) = signal(false);
    let (current_step, set_current_step) = signal(1);
    let (error, set_error) = signal(Option::<String>::None);
    let (success, set_success) = signal(Option::<String>::None);

    // Real-time validation
    let email_valid = Signal::derive(move || {
        let email = email.get();
        !email.is_empty() && email.contains('@') && email.contains('.')
    });

    let username_valid = Signal::derive(move || {
        let username = username.get();
        username.len() >= 3 && username.chars().all(|c| c.is_alphanumeric() || c == '_')
    });

    let password_valid = Signal::derive(move || password.get().len() >= 8);

    let passwords_match = Signal::derive(move || {
        !password.get().is_empty() && password.get() == confirm_password.get()
    });

    let can_proceed_step1 = Signal::derive(move || {
        email_valid.get()
            && username_valid.get()
            && !given_name.get().trim().is_empty()
            && !family_name.get().trim().is_empty()
    });

    let can_proceed_step2 = Signal::derive(move || password_valid.get() && passwords_match.get());

    let can_submit = Signal::derive(move || {
        can_proceed_step1.get() && can_proceed_step2.get() && accept_terms.get()
    });

    // Watch for register action results
    Effect::new(move |_| match register_action.value().get() {
        Some(Ok(message)) => {
            set_error.set(None);
            set_success.set(Some(message));
        }
        Some(Err(e)) => {
            set_error.set(Some(e.to_string()));
            set_success.set(None);
        }
        None => {}
    });

    let next_step = move |_| match current_step.get() {
        1 if can_proceed_step1.get() => {
            set_current_step.set(2);
            set_error.set(None);
        }
        1 => {
            set_error.set(Some(
                "Please fill in all required fields correctly".to_string(),
            ));
        }
        2 if can_proceed_step2.get() => {
            set_current_step.set(3);
            set_error.set(None);
        }
        2 => {
            set_error.set(Some(
                "Please enter matching passwords (at least 8 characters)".to_string(),
            ));
        }
        _ => {}
    };

    let previous_step = move |_| {
        if current_step.get() > 1 {
            set_current_step.update(|step| *step -= 1);
            set_error.set(None);
        }
    };

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        if !can_submit.get() {
            set_error.set(Some("Please complete all steps correctly".to_string()));
            return;
        }

        register_action.dispatch(RegisterAction {
            email: email.get(),
            _phone: if phone.get().trim().is_empty() {
                None
            } else {
                Some(phone.get())
            },
            username: username.get(),
            _given_name: given_name.get(),
            _family_name: family_name.get(),
            password: password.get(),
            confirm_password: confirm_password.get(),
            accept_terms: accept_terms.get(),
        });
    };

    view! {
        <div class="w-full max-w-md mx-auto">
            // Progress indicator
            <div class="mb-8">
                <div class="flex justify-between items-center">
                    <div class="flex items-center">
                        <div class=move || format!(
                            "flex items-center justify-center w-8 h-8 rounded-full text-sm font-medium {}",
                            if current_step.get() >= 1 {
                                "bg-blue-600 text-white"
                            } else {
                                "bg-gray-200 text-gray-600"
                            }
                        )>
                            "1"
                        </div>
                        <div class="ml-3 text-sm font-medium text-gray-700">
                            "Personal Info"
                        </div>
                    </div>

                    <div class="flex-1 mx-4">
                        <div class=move || format!(
                            "h-1 rounded-full {}",
                            if current_step.get() >= 2 { "bg-blue-600" } else { "bg-gray-200" }
                        )/>
                    </div>

                    <div class="flex items-center">
                        <div class=move || format!(
                            "flex items-center justify-center w-8 h-8 rounded-full text-sm font-medium {}",
                            if current_step.get() >= 2 {
                                "bg-blue-600 text-white"
                            } else {
                                "bg-gray-200 text-gray-600"
                            }
                        )>
                            "2"
                        </div>
                        <div class="ml-3 text-sm font-medium text-gray-700">
                            "Security"
                        </div>
                    </div>

                    <div class="flex-1 mx-4">
                        <div class=move || format!(
                            "h-1 rounded-full {}",
                            if current_step.get() >= 3 { "bg-blue-600" } else { "bg-gray-200" }
                        )/>
                    </div>

                    <div class="flex items-center">
                        <div class=move || format!(
                            "flex items-center justify-center w-8 h-8 rounded-full text-sm font-medium {}",
                            if current_step.get() >= 3 {
                                "bg-blue-600 text-white"
                            } else {
                                "bg-gray-200 text-gray-600"
                            }
                        )>
                            "3"
                        </div>
                        <div class="ml-3 text-sm font-medium text-gray-700">
                            "Confirm"
                        </div>
                    </div>
                </div>
            </div>

            // Form
            <form on:submit=handle_submit class="space-y-6">
                // Step 1: Personal Information
                <div class=move || if current_step.get() == 1 { "block" } else { "hidden" }>
                    <div class="space-y-4">
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
                            <label for="phone" class="block text-sm font-medium text-gray-700 mb-2">
                                "Phone Number (Optional)"
                            </label>
                            <input
                                id="phone"
                                name="phone"
                                type="tel"
                                class="block w-full px-3 py-3 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors"
                                placeholder="Enter your phone number"
                                prop:value=phone
                                on:input=move |ev| set_phone.set(event_target_value(&ev))
                            />
                        </div>

                        <div>
                            <label for="username" class="block text-sm font-medium text-gray-700 mb-2">
                                "Username"
                            </label>
                            <input
                                id="username"
                                name="username"
                                type="text"
                                required=true
                                class=move || format!(
                                    "block w-full px-3 py-3 border rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors {}",
                                    if username.get().is_empty() {
                                        "border-gray-300"
                                    } else if username_valid.get() {
                                        "border-green-300 bg-green-50"
                                    } else {
                                        "border-red-300 bg-red-50"
                                    }
                                )
                                placeholder="Choose a username"
                                prop:value=username
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                            />
                            <Show when=move || !username.get().is_empty() && !username_valid.get()>
                                <p class="mt-1 text-sm text-red-600">
                                    "Username must be at least 3 characters (letters, numbers, underscore)"
                                </p>
                            </Show>
                        </div>

                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label for="given_name" class="block text-sm font-medium text-gray-700 mb-2">
                                    "First Name"
                                </label>
                                <input
                                    id="given_name"
                                    name="given_name"
                                    type="text"
                                    required=true
                                    class="block w-full px-3 py-3 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors"
                                    placeholder="First name"
                                    prop:value=given_name
                                    on:input=move |ev| set_given_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label for="family_name" class="block text-sm font-medium text-gray-700 mb-2">
                                    "Last Name"
                                </label>
                                <input
                                    id="family_name"
                                    name="family_name"
                                    type="text"
                                    required=true
                                    class="block w-full px-3 py-3 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors"
                                    placeholder="Last name"
                                    prop:value=family_name
                                    on:input=move |ev| set_family_name.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                    </div>
                </div>

                // Step 2: Security
                <div class=move || if current_step.get() == 2 { "block" } else { "hidden" }>
                    <div class="space-y-4">
                        <div class="relative">
                            <label for="password" class="block text-sm font-medium text-gray-700 mb-2">
                                "Password"
                            </label>
                            <input
                                id="password"
                                name="password"
                                type=move || if show_password.get() { "text" } else { "password" }
                                required=true
                                class=move || format!(
                                    "block w-full px-3 py-3 pr-10 border rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors {}",
                                    if password.get().is_empty() {
                                        "border-gray-300"
                                    } else if password_valid.get() {
                                        "border-green-300 bg-green-50"
                                    } else {
                                        "border-red-300 bg-red-50"
                                    }
                                )
                                placeholder="Create a strong password"
                                prop:value=password
                                on:input=move |ev| set_password.set(event_target_value(&ev))
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
                            <Show when=move || !password.get().is_empty() && !password_valid.get()>
                                <p class="mt-1 text-sm text-red-600">
                                    "Password must be at least 8 characters"
                                </p>
                            </Show>
                        </div>

                        <div class="relative">
                            <label for="confirm_password" class="block text-sm font-medium text-gray-700 mb-2">
                                "Confirm Password"
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
                                    } else if passwords_match.get() {
                                        "border-green-300 bg-green-50"
                                    } else {
                                        "border-red-300 bg-red-50"
                                    }
                                )
                                placeholder="Confirm your password"
                                prop:value=confirm_password
                                on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                            />
                            <button
                                type="button"
                                class="absolute inset-y-0 right-0 pr-3 flex items-center top-8"
                                on:click=move |_| set_show_confirm_password.update(|show| *show = !*show)
                            >
                                <svg class="h-5 w-5 text-gray-400 hover:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                                </svg>
                            </button>
                            <Show when=move || !confirm_password.get().is_empty() && !passwords_match.get()>
                                <p class="mt-1 text-sm text-red-600">
                                    "Passwords do not match"
                                </p>
                            </Show>
                        </div>
                    </div>
                </div>

                // Step 3: Confirmation
                <div class=move || if current_step.get() == 3 { "block" } else { "hidden" }>
                    <div class="space-y-4">
                        <div class="bg-gray-50 rounded-lg p-4">
                            <h3 class="text-lg font-medium text-gray-900 mb-2">
                                "Review Your Information"
                            </h3>
                            <dl class="space-y-2">
                                <div class="flex justify-between">
                                    <dt class="text-sm text-gray-600">"Name:"</dt>
                                    <dd class="text-sm font-medium text-gray-900">
                                        {move || format!("{} {}", given_name.get(), family_name.get())}
                                    </dd>
                                </div>
                                <div class="flex justify-between">
                                    <dt class="text-sm text-gray-600">"Username:"</dt>
                                    <dd class="text-sm font-medium text-gray-900">
                                        {username}
                                    </dd>
                                </div>
                                <div class="flex justify-between">
                                    <dt class="text-sm text-gray-600">"Email:"</dt>
                                    <dd class="text-sm font-medium text-gray-900">
                                        {email}
                                    </dd>
                                </div>
                                <Show when=move || !phone.get().trim().is_empty()>
                                    <div class="flex justify-between">
                                        <dt class="text-sm text-gray-600">"Phone:"</dt>
                                        <dd class="text-sm font-medium text-gray-900">
                                            {phone}
                                        </dd>
                                    </div>
                                </Show>
                            </dl>
                        </div>

                        <div class="flex items-center">
                            <input
                                id="accept_terms"
                                name="accept_terms"
                                type="checkbox"
                                class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                                prop:checked=accept_terms
                                on:change=move |ev| set_accept_terms.set(event_target_checked(&ev))
                            />
                            <label for="accept_terms" class="ml-2 block text-sm text-gray-700">
                                "I agree to the "
                                <a href="/terms" class="text-blue-600 hover:text-blue-500">
                                    "Terms of Service"
                                </a>
                                " and "
                                <a href="/privacy" class="text-blue-600 hover:text-blue-500">
                                    "Privacy Policy"
                                </a>
                            </label>
                        </div>
                    </div>
                </div>

                // Error/Success Messages
                <Show when=move || error.get().is_some()>
                    <div class="rounded-md bg-red-50 p-4 animate-fade-in">
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

                <Show when=move || success.get().is_some()>
                    <div class="rounded-md bg-green-50 p-4 animate-fade-in">
                        <div class="flex">
                            <div class="flex-shrink-0">
                                <svg class="h-5 w-5 text-green-400" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                                </svg>
                            </div>
                            <div class="ml-3">
                                <p class="text-sm font-medium text-green-800">
                                    {move || success.get().unwrap_or_default()}
                                </p>
                            </div>
                        </div>
                    </div>
                </Show>

                // Navigation buttons
                <div class="flex justify-between space-x-4">
                    <Show when=move || { current_step.get() > 1 }>
                        <button
                            type="button"
                            class="flex justify-center py-3 px-6 border border-gray-300 rounded-lg shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
                            on:click=previous_step
                        >
                            "Previous"
                        </button>
                    </Show>

                    <Show
                        when=move || current_step.get() < 3
                        fallback=move || view! {
                            <button
                                type="submit"
                                disabled=move || register_action.pending().get() || !can_submit.get()
                                class="flex-1 flex justify-center py-3 px-6 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                            >
                                <Show
                                    when=move || register_action.pending().get()
                                    fallback=|| "Create Account"
                                >
                                    <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
                                    </svg>
                                    "Creating Account..."
                                </Show>
                            </button>
                        }
                    >
                        <button
                            type="button"
                            class="flex-1 flex justify-center py-3 px-6 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
                            on:click=next_step
                        >
                            "Next"
                        </button>
                    </Show>
                </div>

                // Sign in link
                <div class="text-center">
                    <p class="text-sm text-gray-600">
                        "Already have an account? "
                        <a href="/login" class="font-medium text-blue-600 hover:text-blue-500 transition-colors">
                            "Sign in"
                        </a>
                    </p>
                </div>
            </form>
        </div>
    }
}
