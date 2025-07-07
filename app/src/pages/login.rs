use leptos::prelude::*;
use leptos::form::ActionForm;
use crate::contexts::auth::{use_auth, LoginCredentials, login_user};

#[server(LoginAction, "/api")]
pub async fn login_action(
    email: String,
    password: String,
) -> Result<String, ServerFnError> {
    use leptos_axum::redirect;
    
    // Call the real authentication
    let _auth_response = login_user(LoginCredentials { email, password }).await?;
    
    // For now, redirect to dashboard on success
    // TODO: Implement proper cookie-based session management
    redirect("/dashboard");
    Ok("Login successful".to_string())
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_auth();
    let login_action = ServerAction::<LoginAction>::new();
    let (error, set_error) = signal(Option::<String>::None);

    // If already authenticated, redirect to dashboard
    Effect::new(move |_| {
        if auth.is_authenticated.get() {
            let _ = window().location().set_href("/dashboard");
        }
    });

    // Watch for login action results
    Effect::new(move |_| {
        match login_action.value().get() {
            Some(Ok(_)) => {
                // Success - redirect will be handled by server action
                set_error.set(None);
            }
            Some(Err(e)) => {
                set_error.set(Some(e.to_string()));
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
                        <h2 class="text-2xl font-bold text-gray-900">"Sign in"</h2>
                        <p class="mt-2 text-sm text-gray-600">
                            "Don't have an account? "
                            <a href="/signup" class="text-blue-600 hover:text-blue-500">"Sign up"</a>
                        </p>
                    </div>

                    // Form using ActionForm for SSR
                    <ActionForm action=login_action attr:class="space-y-4">
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

                        // Forgot password link
                        <div class="text-left">
                            <a href="#" class="text-sm text-blue-600 hover:text-blue-500">
                                "Forgot password?"
                            </a>
                        </div>

                        // Error Message
                        <Show when=move || error.get().is_some()>
                            <div class="rounded-md bg-red-50 p-4">
                                <div class="flex">
                                    <div class="flex-shrink-0">
                                        <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                                        </svg>
                                    </div>
                                    <div class="ml-3">
                                        <p class="text-sm text-red-700">
                                            {move || error.get().unwrap_or_default()}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </Show>

                        // Submit Button
                        <div>
                            <button
                                type="submit"
                                disabled=move || login_action.pending().get()
                                class="w-full flex justify-center py-3 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                {move || if login_action.pending().get() { "Signing in..." } else { "Sign in" }}
                            </button>
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
