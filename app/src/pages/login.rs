use crate::components::auth::EnhancedLoginForm;
use crate::contexts::auth::{login_user, use_auth, LoginCredentials};
use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

#[server(LoginAction, "/api")]
pub async fn login_action(email: String, password: String) -> Result<String, ServerFnError> {
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

    // If already authenticated, redirect to dashboard
    Effect::new(move |_| {
        if auth.is_authenticated.get() {
            let _ = window().location().set_href("/dashboard");
        }
    });

    view! {
        <div class="min-h-screen flex bg-gray-50">
            // Left side - Form
            <div class="flex-1 flex flex-col justify-center py-12 px-4 sm:px-6 lg:px-20 xl:px-24 bg-white">
                <div class="mx-auto w-full max-w-sm lg:w-96">
                    // Logo
                    <div class="mb-8">
                        <img src="/assets/logo.svg" alt="Bitsacco" class="h-12 w-12" />
                    </div>

                    // Header
                    <div class="mb-8">
                        <h2 class="text-3xl font-bold text-gray-900 mb-2">"Welcome back"</h2>
                        <p class="text-base text-gray-600">
                            "Sign in to access your admin dashboard"
                        </p>
                    </div>

                    // Enhanced Login Form
                    <EnhancedLoginForm/>

                </div>
            </div>

            // Right side - Branding section with modern design
            <div class="hidden lg:block relative w-0 flex-1 bg-teal-600">
                <div class="absolute inset-0 flex flex-col items-center justify-center p-12">
                    <div class="text-center">
                        // Modern logo design
                        <div class="mb-8">
                            <div class="w-40 h-40 mx-auto rounded-3xl bg-white/10 backdrop-blur-sm border border-white/20 flex items-center justify-center shadow-2xl">
                                <img src="/assets/logo.svg" alt="Bitsacco" class="w-24 h-24 filter brightness-0 invert" />
                            </div>
                        </div>

                        <p class="text-xl text-teal-100 mb-8 font-medium">
                            "Community Financial Management"
                        </p>

                        <div class="space-y-4 text-teal-100/80">
                            <div class="flex items-center justify-center space-x-3">
                                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                                </svg>
                                <span>"Secure member management"</span>
                            </div>
                            <div class="flex items-center justify-center space-x-3">
                                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                                </svg>
                                <span>"Advanced analytics dashboard"</span>
                            </div>
                            <div class="flex items-center justify-center space-x-3">
                                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                                </svg>
                                <span>"Real-time notifications"</span>
                            </div>
                        </div>
                    </div>
                </div>

                // Subtle geometric pattern overlay
                <div class="absolute inset-0 bg-gradient-to-t from-black/20 to-transparent"></div>
                <div class="absolute top-0 left-0 w-full h-full">
                    <div class="absolute top-10 left-10 w-20 h-20 border border-white/10 rounded-full"></div>
                    <div class="absolute top-32 right-16 w-12 h-12 border border-white/10 rounded-lg rotate-45"></div>
                    <div class="absolute bottom-20 left-20 w-16 h-16 border border-white/10 rounded-full"></div>
                    <div class="absolute bottom-32 right-32 w-8 h-8 border border-white/10 rounded-full"></div>
                </div>
            </div>
        </div>
    }.into_any()
}
