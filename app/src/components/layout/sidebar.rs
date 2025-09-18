use crate::contexts::auth::use_auth;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn Sidebar(mobile_open: Signal<bool>, set_mobile_open: WriteSignal<bool>) -> impl IntoView {
    view! {
        // Mobile sidebar overlay - full screen like web app
        {move || if mobile_open.get() {
            view! {
                <div class="fixed inset-0 z-50 bg-slate-800/95 backdrop-blur-xl lg:hidden">
                    <div class="flex h-screen flex-col bg-slate-800/95 backdrop-blur-xl">
                        // Header with logo and close button (matching web app)
                        <div class="flex items-center justify-between border-b border-slate-700 px-6 py-4">
                            <div class="flex items-center">
                                <img src="/assets/logo.svg" alt="Bitsacco" class="h-10 w-10 filter brightness-0 invert" />
                                <span class="ml-3 text-xl font-bold font-title text-white">"Bitsacco"</span>
                            </div>
                            <button
                                type="button"
                                class="flex size-10 items-center justify-center rounded-lg transition-colors hover:bg-slate-700/50 focus-ring"
                                on:click=move |_| set_mobile_open.set(false)
                                aria-label="Close menu"
                            >
                                <span class="sr-only">"Close menu"</span>
                                // X icon (matching web app)
                                <svg class="h-6 w-6 text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
                                </svg>
                            </button>
                        </div>
                        // Scrollable content area
                        <div class="flex-1 overflow-y-auto scrollbar-thin">
                            <MobileSidebarContent set_mobile_open=set_mobile_open/>
                        </div>
                    </div>
                </div>
            }.into_any()
        } else {
            view! { <div></div> }.into_any()
        }}

        // Desktop sidebar
        <div class="hidden lg:flex lg:w-64 lg:flex-col lg:fixed lg:inset-y-0 lg:border-r lg:border-gray-200 lg:bg-white">
            <DesktopSidebarContent/>
        </div>
    }
}

#[component]
fn MobileSidebarContent(set_mobile_open: WriteSignal<bool>) -> impl IntoView {
    let auth = use_auth();

    view! {
        // Mobile navigation content - full screen dark style like web app
        <div class="flex flex-col flex-1">
            <nav class="px-6 py-8 space-y-1">
                <a href="/dashboard" class="group flex items-center px-4 py-3 text-xl font-semibold rounded-lg text-white hover:bg-slate-700/50 transition-all duration-200"
                   on:click=move |_| set_mobile_open.set(false)>
                    <svg class="w-6 h-6 mr-3 text-gray-400 group-hover:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2z" />
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 5a2 2 0 012-2h4a2 2 0 012 2v4H8V5z" />
                    </svg>
                    <span class="truncate font-body">"Dashboard"</span>
                </a>

                <a href="/members" class="group flex items-center px-4 py-3 text-xl font-semibold rounded-lg text-white hover:bg-slate-700/50 transition-all duration-200"
                   on:click=move |_| set_mobile_open.set(false)>
                    <svg class="w-6 h-6 mr-3 text-gray-400 group-hover:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z" />
                    </svg>
                    <span class="truncate font-body">"Members"</span>
                </a>

                <a href="/groups" class="group flex items-center px-4 py-3 text-xl font-semibold rounded-lg text-white hover:bg-slate-700/50 transition-all duration-200"
                   on:click=move |_| set_mobile_open.set(false)>
                    <svg class="w-6 h-6 mr-3 text-gray-400 group-hover:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                    </svg>
                    <span class="truncate font-body">"Groups"</span>
                </a>

                <a href="/shares" class="group flex items-center px-4 py-3 text-xl font-semibold rounded-lg text-white hover:bg-slate-700/50 transition-all duration-200"
                   on:click=move |_| set_mobile_open.set(false)>
                    <svg class="w-6 h-6 mr-3 text-gray-400 group-hover:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                    </svg>
                    <span class="truncate font-body">"Shares"</span>
                </a>

                <a href="/settings" class="group flex items-center px-4 py-3 text-xl font-semibold rounded-lg text-white hover:bg-slate-700/50 transition-all duration-200 mt-8"
                   on:click=move |_| set_mobile_open.set(false)>
                    <svg class="w-6 h-6 mr-3 text-gray-400 group-hover:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    <span class="truncate font-body">"System Settings"</span>
                </a>
            </nav>

            // User section - Fixed at bottom with dark styling
            <div class="flex-shrink-0 border-t border-slate-700 px-6 py-4 mt-8">
                <div class="flex items-center space-x-3 mb-4">
                    <div class="w-10 h-10 bg-gradient-to-br from-teal-500 to-teal-600 rounded-full flex items-center justify-center shadow-lg">
                        <svg class="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                        </svg>
                    </div>
                    <div class="flex-1 min-w-0">
                        <p class="text-base font-medium text-gray-100 truncate font-body">
                            {move || {
                                auth.user
                                    .get()
                                    .as_ref()
                                    .map(|u| {
                                        if let Some(phone) = &u.phone {
                                            let phone_number = &phone.number;
                                            if phone_number.len() > 6 {
                                                format!(
                                                    "{} {} {}",
                                                    &phone_number[..4],
                                                    &phone_number[4..7],
                                                    &phone_number[7..]
                                                )
                                            } else {
                                                phone_number.clone()
                                            }
                                        } else {
                                            "User".to_string()
                                        }
                                    })
                                    .unwrap_or_else(|| "User".to_string())
                            }}
                        </p>
                        <p class="text-sm text-gray-400 truncate font-body">"Member"</p>
                    </div>
                </div>
                <button
                    type="button"
                    class="w-full flex items-center justify-center px-4 py-2 bg-slate-700/60 text-gray-200 border border-slate-600 rounded-lg hover:bg-red-500/20 hover:text-red-300 hover:border-red-500/50 transition-all duration-200 font-body"
                    on:click=move |_| {
                        set_mobile_open.set(false);
                        auth.logout.run(());
                    }
                >
                    <svg class="w-4 h-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                    </svg>
                    "Sign Out"
                </button>
            </div>
        </div>
    }
}

#[component]
fn DesktopSidebarContent() -> impl IntoView {
    let location = use_location();
    let _auth = use_auth();

    view! {
        <div class="flex-1 flex flex-col min-h-0 bg-white">
            // Logo/Brand
            <div class="flex items-center h-16 flex-shrink-0 px-4 border-b border-gray-200 bg-gradient-to-r from-teal-600 to-teal-700">
                <div class="flex items-center w-full">
                    <img src="/assets/logo.svg" alt="Bitsacco" class="h-10 w-10 filter brightness-0 invert" />
                    <div class="ml-3">
                        <span class="text-white text-xl font-bold font-title tracking-tight">"Bitsacco"</span>
                        <div class="text-white/80 text-xs font-medium font-body tracking-wide">"Admin Dashboard"</div>
                    </div>
                </div>
            </div>

            // Navigation
            <div class="flex-1 flex flex-col pt-6 pb-4 overflow-y-auto scrollbar-thin">
                <nav class="flex-1 px-4 space-y-2">
                    <NavItem
                        href="/dashboard"
                        icon_svg=view! {
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2z" />
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 5a2 2 0 012-2h4a2 2 0 012 2v4H8V5z" />
                            </svg>
                        }
                        text="Dashboard"
                        current_path=location.pathname.into()
                    />
                    <NavItem
                        href="/members"
                        icon_svg=view! {
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z" />
                            </svg>
                        }
                        text="Members"
                        current_path=location.pathname.into()
                    />
                    <NavItem
                        href="/groups"
                        icon_svg=view! {
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                            </svg>
                        }
                        text="Groups"
                        current_path=location.pathname.into()
                    />
                    <NavItem
                        href="/shares"
                        icon_svg=view! {
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                            </svg>
                        }
                        text="Shares"
                        current_path=location.pathname.into()
                    />

                    <div class="pt-4">
                        <div class="text-xs font-semibold font-body text-gray-400 uppercase tracking-widest px-2 mb-2">
                            "Settings"
                        </div>
                        <NavItem
                            href="/settings"
                            icon_svg=view! {
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                </svg>
                            }
                            text="System Settings"
                            current_path=location.pathname.into()
                        />
                    </div>
                </nav>
            </div>

            // User section
            <UserProfile/>
        </div>
    }
}

#[component]
fn NavItem(
    href: &'static str,
    icon_svg: impl IntoView + 'static,
    text: &'static str,
    current_path: Signal<String>,
) -> impl IntoView {
    let is_current = move || {
        let path = current_path.get();
        path == href || (href != "/" && path.starts_with(href))
    };

    view! {
        <a
            href=href
            class=move || {
                let base_classes = "group flex items-center px-3 py-2.5 text-sm font-medium rounded-lg transition-all duration-200 hover:scale-[1.02] active:scale-[0.98]";
                if is_current() {
                    format!("{} bg-gradient-to-r from-teal-50 to-teal-100 text-teal-700 border border-teal-200 shadow-sm transform", base_classes)
                } else {
                    format!("{} text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 hover:text-gray-900 hover:shadow-sm", base_classes)
                }
            }
        >
            <span class=move || {
                let transition_classes = "mr-3 transition-all duration-200 transform";
                if is_current() {
                    format!("{} text-teal-600 scale-110", transition_classes)
                } else {
                    format!("{} text-gray-400 group-hover:text-gray-600 group-hover:scale-110", transition_classes)
                }
            }>
                {icon_svg}
            </span>
            <span class="font-medium font-body transition-colors duration-200">
                {text}
            </span>

            // Active indicator
            {move || if is_current() {
                view! {
                    <span class="ml-auto">
                        <div class="w-2 h-2 bg-teal-600 rounded-full animate-pulse"></div>
                    </span>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </a>
    }
}

#[component]
fn UserProfile() -> impl IntoView {
    let auth = use_auth();
    let (menu_open, set_menu_open) = signal(false);

    // Check if auth is still loading
    let is_loading = move || auth.is_loading.get();

    // Get user info - no hardcoded fallbacks
    let user_name = move || {
        let user_option = auth.user.get();

        // Debug logging to see what we're getting
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(ref user) = user_option {
                web_sys::console::log_1(
                    &format!("UserProfile: User is logged in - ID: {}", user.id).into(),
                );
                if let Some(phone) = &user.phone {
                    web_sys::console::log_1(
                        &format!("UserProfile: Phone: {}", phone.number).into(),
                    );
                }
            } else {
                web_sys::console::log_1(
                    &format!("UserProfile: No user found - showing fallback").into(),
                );
            }
        }

        user_option
            .as_ref()
            .map(|u| {
                // If phone is available, format it nicely, otherwise use ID
                if let Some(phone) = &u.phone {
                    // Format phone number for display (e.g., +254700123456 -> +254 700 123456)
                    let phone_number = &phone.number;
                    if phone_number.len() > 6 {
                        format!(
                            "{} {} {}",
                            &phone_number[..4],  // Country code
                            &phone_number[4..7], // First 3 digits
                            &phone_number[7..]   // Remaining digits
                        )
                    } else {
                        phone_number.clone()
                    }
                } else if let Some(nostr) = &u.nostr {
                    // Show truncated nostr address
                    let npub = &nostr.npub;
                    if npub.len() > 16 {
                        format!("{}...{}", &npub[..8], &npub[npub.len() - 8..])
                    } else {
                        npub.clone()
                    }
                } else {
                    let id_str = u.id.to_string();
                    format!("User {}", &id_str[..8.min(id_str.len())])
                }
            })
            .unwrap_or_else(|| "User".to_string())
    };

    let user_contact = move || {
        let user_option = auth.user.get();

        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(
                &format!(
                    "UserProfile user_contact: user present: {}",
                    user_option.is_some()
                )
                .into(),
            );
        }

        user_option
            .as_ref()
            .map(|u| {
                if let Some(phone) = &u.phone {
                    phone.number.clone()
                } else if let Some(nostr) = &u.nostr {
                    let npub = &nostr.npub;
                    format!(
                        "nostr: {}",
                        if npub.len() > 20 {
                            format!("{}...", &npub[..20])
                        } else {
                            npub.clone()
                        }
                    )
                } else {
                    let id_str = u.id.to_string();
                    format!("ID: {}", &id_str[..12.min(id_str.len())])
                }
            })
            .unwrap_or_else(|| "---".to_string())
    };

    let user_initials = move || {
        auth.user
            .get()
            .as_ref()
            .map(|u| {
                if let Some(phone) = &u.phone {
                    // For phone numbers, use last 2 digits
                    let phone_number = &phone.number;
                    if phone_number.len() >= 2 {
                        phone_number[phone_number.len() - 2..].to_string()
                    } else {
                        "U".to_string()
                    }
                } else if let Some(nostr) = &u.nostr {
                    // For nostr addresses, use first 2 characters after 'npub' if present
                    let npub = &nostr.npub;
                    if npub.starts_with("npub") && npub.len() > 5 {
                        npub[4..6].to_uppercase()
                    } else if npub.len() >= 2 {
                        npub[..2].to_uppercase()
                    } else {
                        "N".to_string()
                    }
                } else {
                    // For IDs, use first 2 characters
                    let id_str = u.id.to_string();
                    if id_str.len() >= 2 {
                        id_str[..2].to_uppercase()
                    } else {
                        "U".to_string()
                    }
                }
            })
            .unwrap_or_else(|| "U".to_string())
    };

    let handle_logout = move |_| {
        set_menu_open.set(false);
        auth.logout.run(());
        // Navigate to login page
        #[cfg(target_arch = "wasm32")]
        {
            let _ = web_sys::window().unwrap().location().set_href("/login");
        }
    };

    view! {
        <div class="flex-shrink-0 relative bg-gray-50 border-t border-gray-200 p-4">
            // Show loading state if auth is still loading
            {move || if is_loading() {
                view! {
                    <div class="flex items-center w-full">
                        <div class="flex-shrink-0">
                            <div class="h-10 w-10 rounded-full bg-gray-300 animate-pulse"></div>
                        </div>
                        <div class="ml-3 flex-1">
                            <div class="h-4 bg-gray-300 rounded animate-pulse mb-1"></div>
                            <div class="h-3 bg-gray-200 rounded animate-pulse w-3/4"></div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
            <div class="flex items-center w-full">
                <div class="flex-shrink-0">
                    <div class="h-10 w-10 rounded-full bg-gradient-to-br from-teal-500 to-teal-600 flex items-center justify-center ring-2 ring-white">
                        <span class="text-sm font-bold text-white">{user_initials}</span>
                    </div>
                </div>
                <div class="ml-3 flex-1 min-w-0">
                    <p class="text-sm font-semibold font-body text-gray-900 truncate">{user_name}</p>
                    <p class="text-xs font-body text-gray-500 truncate">{user_contact}</p>
                </div>
                <div class="ml-2">
                    <button
                        type="button"
                        class="flex-shrink-0 p-1.5 text-gray-400 hover:text-gray-600 focus-ring rounded-lg transition-colors hover:bg-gray-100"
                        on:click=move |_| set_menu_open.update(|open| *open = !*open)
                    >
                        <span class="sr-only">"User menu"</span>
                        <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
                        </svg>
                    </button>
                </div>
            </div>

            // Dropdown menu
            {move || if menu_open.get() {
                view! {
                    <div class="absolute bottom-full left-4 right-4 mb-2 bg-white rounded-xl shadow-xl ring-1 ring-black ring-opacity-5 focus:outline-none z-50 border border-gray-200 animate-fade-in animate-slide-in-up">
                        <div class="py-2">
                            <a
                                href="/profile"
                                class="flex items-center w-full text-left px-4 py-2.5 text-sm text-gray-700 hover:bg-gray-50 transition-colors"
                                on:click=move |_| set_menu_open.set(false)
                            >
                                <svg class="mr-3 h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                                </svg>
                                "Your Profile"
                            </a>
                            <a
                                href="/settings"
                                class="flex items-center w-full text-left px-4 py-2.5 text-sm text-gray-700 hover:bg-gray-50 transition-colors"
                                on:click=move |_| set_menu_open.set(false)
                            >
                                <svg class="mr-3 h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                </svg>
                                "Preferences"
                            </a>
                            <div class="border-t border-gray-100 my-1"></div>
                            <button
                                type="button"
                                class="flex items-center w-full text-left px-4 py-2.5 text-sm text-red-700 hover:bg-red-50 transition-colors"
                                on:click=handle_logout
                            >
                                <svg class="mr-3 h-4 w-4 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                                </svg>
                                "Sign out"
                            </button>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
                }.into_any()
            }}
        </div>
    }
}
