use crate::components::auth::RegisterForm;
use leptos::prelude::*;

#[component]
pub fn SignupPage() -> impl IntoView {
    view! {
        <div class="min-h-screen flex bg-gray-50">
            // Left side - Form
            <div class="flex-1 flex flex-col justify-center py-12 px-4 sm:px-6 lg:px-20 xl:px-24 bg-white">
                <div class="mx-auto w-full max-w-md lg:w-full lg:max-w-md">
                    // Logo
                    <div class="mb-8">
                        <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-600 to-blue-700 flex items-center justify-center shadow-lg">
                            <svg class="w-7 h-7 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1" />
                            </svg>
                        </div>
                    </div>

                    // Header
                    <div class="mb-8">
                        <h2 class="text-3xl font-bold text-gray-900 mb-2">"Create your account"</h2>
                        <p class="text-base text-gray-600">
                            "Join our community management platform"
                        </p>
                    </div>

                    // Enhanced Registration Form
                    <RegisterForm/>

                    // Sign in link
                    <div class="mt-8 text-center">
                        <p class="text-sm text-gray-600">
                            "Already have an account? "
                            <a href="/login" class="font-medium text-blue-600 hover:text-blue-500 transition-colors">
                                "Sign in here"
                            </a>
                        </p>
                    </div>
                </div>
            </div>

            // Right side - Feature showcase
            <div class="hidden lg:block relative w-0 flex-1 bg-gradient-to-br from-green-600 via-blue-600 to-purple-700">
                <div class="absolute inset-0 flex flex-col items-center justify-center p-12">
                    <div class="text-center">
                        // Feature highlights
                        <div class="mb-8">
                            <h1 class="text-5xl font-bold text-white mb-4 tracking-tight">
                                "Join Us"
                            </h1>
                            <p class="text-xl text-white/90 mb-8 font-medium">
                                "Powerful tools for community management"
                            </p>
                        </div>

                        <div class="space-y-6 text-white/90">
                            <div class="flex items-center justify-center space-x-4">
                                <div class="w-12 h-12 rounded-xl bg-white/10 backdrop-blur-sm border border-white/20 flex items-center justify-center">
                                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                                    </svg>
                                </div>
                                <div class="text-left">
                                    <div class="font-semibold">"Member Management"</div>
                                    <div class="text-sm text-white/70">"Organize and track community members"</div>
                                </div>
                            </div>

                            <div class="flex items-center justify-center space-x-4">
                                <div class="w-12 h-12 rounded-xl bg-white/10 backdrop-blur-sm border border-white/20 flex items-center justify-center">
                                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                                    </svg>
                                </div>
                                <div class="text-left">
                                    <div class="font-semibold">"Financial Analytics"</div>
                                    <div class="text-sm text-white/70">"Track shares, investments, and growth"</div>
                                </div>
                            </div>

                            <div class="flex items-center justify-center space-x-4">
                                <div class="w-12 h-12 rounded-xl bg-white/10 backdrop-blur-sm border border-white/20 flex items-center justify-center">
                                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                                    </svg>
                                </div>
                                <div class="text-left">
                                    <div class="font-semibold">"Bank-Grade Security"</div>
                                    <div class="text-sm text-white/70">"Your data is protected and encrypted"</div>
                                </div>
                            </div>

                            <div class="flex items-center justify-center space-x-4">
                                <div class="w-12 h-12 rounded-xl bg-white/10 backdrop-blur-sm border border-white/20 flex items-center justify-center">
                                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-5 5v-5z" />
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9v-9" />
                                    </svg>
                                </div>
                                <div class="text-left">
                                    <div class="font-semibold">"Real-time Updates"</div>
                                    <div class="text-sm text-white/70">"Stay informed with live notifications"</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                // Decorative elements
                <div class="absolute inset-0 bg-gradient-to-t from-black/20 to-transparent"></div>
                <div class="absolute top-0 left-0 w-full h-full">
                    <div class="absolute top-16 left-16 w-24 h-24 border border-white/10 rounded-full animate-pulse"></div>
                    <div class="absolute top-40 right-20 w-16 h-16 border border-white/10 rounded-lg rotate-45"></div>
                    <div class="absolute bottom-24 left-24 w-20 h-20 border border-white/10 rounded-full"></div>
                    <div class="absolute bottom-40 right-40 w-12 h-12 border border-white/10 rounded-full animate-pulse"></div>
                </div>
            </div>
        </div>
    }.into_any()
}
