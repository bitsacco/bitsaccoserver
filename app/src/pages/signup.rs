use leptos::prelude::*;
use leptos::form::ActionForm;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupCredentials {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub password: String,
    pub confirm_password: String,
    pub terms_accepted: bool,
}

#[server(SignupAction, "/api")]
pub async fn signup_action(
    email: String,
    first_name: String,
    last_name: String,
    _phone: String,
    password: String,
    confirm_password: String,
    terms: Option<String>, // checkbox comes as "on" or nothing
) -> Result<String, ServerFnError> {
    // Basic validation
    if email.is_empty() || first_name.is_empty() || last_name.is_empty() || password.is_empty() {
        return Err(ServerFnError::new("Please fill in all required fields"));
    }
    
    if password != confirm_password {
        return Err(ServerFnError::new("Passwords do not match"));
    }
    
    if terms.is_none() {
        return Err(ServerFnError::new("Please accept the terms and conditions"));
    }
    
    // For now, just return success message
    // In production, this would create the user account
    Ok("Registration successful! Please check your email for verification.".to_string())
}

#[component]
pub fn SignupPage() -> impl IntoView {
    let signup_action = ServerAction::<SignupAction>::new();
    let (error, set_error) = signal(Option::<String>::None);
    let (success, set_success) = signal(Option::<String>::None);

    // Watch for signup results
    Effect::new(move |_| {
        match signup_action.value().get() {
            Some(Ok(message)) => {
                set_success.set(Some(message));
                set_error.set(None);
            }
            Some(Err(e)) => {
                set_error.set(Some(e.to_string()));
                set_success.set(None);
            }
            None => {}
        }
    });

    view! {
        <div class="min-h-screen flex">
            // Left side - Form
            <div class="flex-1 flex flex-col justify-center py-12 px-4 sm:px-6 lg:px-20 xl:px-24 bg-white">
                <div class="mx-auto w-full max-w-sm lg:w-96">
                    // Logo
                    <div class="mb-8">
                        <div class="w-10 h-10 rounded-lg bg-gray-900 flex items-center justify-center">
                            <span class="text-white font-bold text-xl">"$"</span>
                        </div>
                    </div>

                    // Header
                    <div class="mb-8">
                        <h2 class="text-2xl font-bold text-gray-900">"Sign up"</h2>
                        <p class="mt-2 text-sm text-gray-600">
                            "Already have an account? "
                            <a href="/login" class="text-blue-600 hover:text-blue-500">"Sign in"</a>
                        </p>
                    </div>

                    // Form using ActionForm for server action
                    <ActionForm action=signup_action attr:class="space-y-4">
                        // Email
                        <div>
                            <input
                                id="email"
                                name="email"
                                type="email"
                                required=true
                                class="block w-full px-3 py-3 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Email address"
                            />
                        </div>

                        // First and Last Name
                        <div class="grid grid-cols-2 gap-3">
                            <div>
                                <input
                                    id="first_name"
                                    name="first_name"
                                    type="text"
                                    required=true
                                    class="block w-full px-3 py-3 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    placeholder="First name"
                                />
                            </div>
                            <div>
                                <input
                                    id="last_name"
                                    name="last_name"
                                    type="text"
                                    required=true
                                    class="block w-full px-3 py-3 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    placeholder="Last name"
                                />
                            </div>
                        </div>

                        // Phone (optional)
                        <div>
                            <input
                                id="phone"
                                name="phone"
                                type="tel"
                                class="block w-full px-3 py-3 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Phone number (optional)"
                            />
                        </div>

                        // Password
                        <div class="relative">
                            <input
                                id="password"
                                name="password"
                                type="password"
                                required=true
                                class="block w-full px-3 py-3 pr-10 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Password"
                            />
                            <div class="absolute inset-y-0 right-0 pr-3 flex items-center">
                                <svg class="h-5 w-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"></path>
                                </svg>
                            </div>
                        </div>

                        // Confirm Password
                        <div class="relative">
                            <input
                                id="confirm_password"
                                name="confirm_password"
                                type="password"
                                required=true
                                class="block w-full px-3 py-3 pr-10 border border-gray-300 rounded-lg shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Confirm password"
                            />
                            <div class="absolute inset-y-0 right-0 pr-3 flex items-center">
                                <svg class="h-5 w-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"></path>
                                </svg>
                            </div>
                        </div>

                        // Terms and Conditions
                        <div class="flex items-start">
                            <div class="flex items-center h-5">
                                <input
                                    id="terms"
                                    name="terms"
                                    type="checkbox"
                                    class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                                />
                            </div>
                            <div class="ml-3 text-sm">
                                <span class="text-gray-500">
                                    "I have read the "
                                    <a href="#" class="text-blue-600 hover:text-blue-500">"terms and conditions"</a>
                                </span>
                            </div>
                        </div>

                        // Error Message
                        <Show when=move || error.get().is_some()>
                            <div class="rounded-md bg-red-50 p-4">
                                <div class="text-sm text-red-700">
                                    {move || error.get().unwrap_or_default()}
                                </div>
                            </div>
                        </Show>

                        // Success Message
                        <Show when=move || success.get().is_some()>
                            <div class="rounded-md bg-green-50 p-4">
                                <div class="text-sm text-green-700">
                                    {move || success.get().unwrap_or_default()}
                                </div>
                            </div>
                        </Show>

                        // Submit Button
                        <div>
                            <button
                                type="submit"
                                disabled=move || signup_action.pending().get()
                                class="w-full flex justify-center py-3 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                {move || if signup_action.pending().get() { "Signing up..." } else { "Sign up" }}
                            </button>
                        </div>

                        // Info Message
                        <div class="rounded-md bg-blue-50 p-4">
                            <div class="flex">
                                <div class="flex-shrink-0">
                                    <svg class="h-5 w-5 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path>
                                    </svg>
                                </div>
                                <div class="ml-3">
                                    <p class="text-sm text-blue-700">
                                        "After registration, you will receive an email verification link."
                                        <br/>
                                        "You must verify your email before you can sign in."
                                    </p>
                                </div>
                            </div>
                        </div>
                    </ActionForm>
                </div>
            </div>

            // Right side - Dark section with logo
            <div class="hidden lg:block relative w-0 flex-1 bg-gradient-to-br from-gray-900 to-gray-800">
                <div class="absolute inset-0 flex flex-col items-center justify-center p-12">
                    <div class="text-center">
                        <h1 class="text-4xl font-bold text-teal-400 mb-4">"Admin Dashboard"</h1>
                        <p class="text-xl text-gray-300 mb-8">"Gateway to Community Service"</p>
                        
                        // Large circular logo
                        <div class="w-64 h-64 mx-auto border-4 border-gray-600 rounded-full flex items-center justify-center">
                            <div class="w-48 h-48 border-2 border-gray-500 rounded-full flex items-center justify-center">
                                <span class="text-6xl font-bold text-gray-400">"$"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }.into_any()
}