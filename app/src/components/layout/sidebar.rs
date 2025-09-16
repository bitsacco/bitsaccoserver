use crate::contexts::app_state::use_app_state;
use crate::contexts::auth::use_auth;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn Sidebar(
    #[prop(optional)] mobile_open: Signal<bool>,
    #[prop(optional)] set_mobile_open: Option<WriteSignal<bool>>,
) -> impl IntoView {
    let location = use_location();
    let _app_state = use_app_state();

    view! {
        // Mobile sidebar overlay with smooth animations
        {let location_clone = location.clone();
        move || if mobile_open.get() {
            view! {
                <div class="fixed inset-0 z-40 lg:hidden">
                    // Backdrop with fade animation
                    <div
                        class="fixed inset-0 bg-gray-600 transition-opacity duration-300 ease-in-out"
                        style=move || if mobile_open.get() { "opacity: 0.75;" } else { "opacity: 0;" }
                        on:click=move |_| {
                            if let Some(setter) = set_mobile_open {
                                setter.set(false);
                            }
                        }
                    ></div>

                    // Sliding sidebar panel
                    <div
                        class="relative flex-1 flex flex-col max-w-xs w-full bg-white shadow-xl border-r border-gray-200 transform transition-transform duration-300 ease-in-out"
                        style=move || if mobile_open.get() { "transform: translateX(0);" } else { "transform: translateX(-100%);" }
                    >
                        // Close button with smooth hover effects
                        <div class="absolute top-0 right-0 -mr-12 pt-2">
                            <button
                                type="button"
                                class="ml-1 flex items-center justify-center h-10 w-10 rounded-full bg-black/20 backdrop-blur-sm focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white transition-all duration-200 hover:bg-black/30 hover:scale-105"
                                on:click=move |_| {
                                    if let Some(setter) = set_mobile_open {
                                        setter.set(false);
                                    }
                                }
                            >
                                <span class="sr-only">"Close sidebar"</span>
                                <svg class="h-6 w-6 text-white transform transition-transform duration-200 hover:rotate-90" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        </div>
                        <SidebarContent location=location_clone.clone() is_mobile=true/>
                    </div>
                </div>
            }.into_any()
        } else {
            view! { <div></div> }.into_any()
        }}

        // Desktop sidebar
        <div class="hidden lg:flex lg:w-64 lg:flex-col lg:fixed lg:inset-y-0 lg:border-r lg:border-gray-200 lg:bg-white">
            <SidebarContent location=location.clone() is_mobile=false/>
        </div>
    }
}

#[component]
fn SidebarContent(
    location: leptos_router::location::Location,
    #[prop(optional)] is_mobile: bool,
) -> impl IntoView {
    let _app_state = use_app_state();
    let auth = use_auth();

    view! {
        <div class="flex-1 flex flex-col min-h-0 bg-white">
            // Logo/Brand
            <div class="flex items-center h-16 flex-shrink-0 px-4 border-b border-gray-200 bg-gradient-to-r from-blue-600 to-blue-700">
                <div class="flex items-center w-full">
                    <div class="flex-shrink-0 h-10 w-10 bg-white/20 backdrop-blur-sm rounded-lg flex items-center justify-center">
                        <svg class="h-6 w-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1" />
                        </svg>
                    </div>
                    <div class="ml-3">
                        <span class="text-white text-lg font-bold tracking-wide">"Bitsacco"</span>
                        <div class="text-white/80 text-xs font-medium">"Admin Dashboard"</div>
                    </div>
                </div>
            </div>

            // Navigation with staggered animations for mobile
            <div class="flex-1 flex flex-col pt-6 pb-4 overflow-y-auto">
                <nav class=format!("flex-1 px-4 space-y-2 {}", if is_mobile { "animate-fade-in-up" } else { "" })>
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
                        <div class="text-xs font-semibold text-gray-400 uppercase tracking-wider px-2 mb-2">
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
            <UserProfile auth=auth/>
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
                    format!("{} bg-gradient-to-r from-blue-50 to-blue-100 text-blue-700 border border-blue-200 shadow-sm transform", base_classes)
                } else {
                    format!("{} text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 hover:text-gray-900 hover:shadow-sm", base_classes)
                }
            }
        >
            <span class=move || {
                let transition_classes = "mr-3 transition-all duration-200 transform";
                if is_current() {
                    format!("{} text-blue-600 scale-110", transition_classes)
                } else {
                    format!("{} text-gray-400 group-hover:text-gray-600 group-hover:scale-110", transition_classes)
                }
            }>
                {icon_svg}
            </span>
            <span class="font-medium transition-colors duration-200">
                {text}
            </span>

            // Active indicator
            {move || if is_current() {
                view! {
                    <span class="ml-auto">
                        <div class="w-2 h-2 bg-blue-600 rounded-full animate-pulse"></div>
                    </span>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </a>
    }
}

#[component]
fn UserProfile(auth: crate::contexts::auth::AuthContext) -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);

    // Get user info - no hardcoded fallbacks
    let user_name = move || {
        auth.user
            .get()
            .as_ref()
            .map(|u| {
                // If phone is available, format it nicely, otherwise use ID
                if let Some(phone) = &u.phone {
                    // Format phone number for display (e.g., +254700123456 -> +254 700 123456)
                    if phone.len() > 6 {
                        format!("{} {} {}",
                            &phone[..4],  // Country code
                            &phone[4..7], // First 3 digits
                            &phone[7..]   // Remaining digits
                        )
                    } else {
                        phone.clone()
                    }
                } else if let Some(nostr) = &u.nostr {
                    // Show truncated nostr address
                    if nostr.len() > 16 {
                        format!("{}...{}", &nostr[..8], &nostr[nostr.len()-8..])
                    } else {
                        nostr.clone()
                    }
                } else {
                    format!("User {}", &u.id[..8.min(u.id.len())])
                }
            })
            .unwrap_or_else(|| "Not logged in".to_string())
    };

    let user_contact = move || {
        auth.user
            .get()
            .as_ref()
            .map(|u| {
                if let Some(phone) = &u.phone {
                    phone.clone()
                } else if let Some(nostr) = &u.nostr {
                    format!("nostr: {}", if nostr.len() > 20 {
                        format!("{}...", &nostr[..20])
                    } else {
                        nostr.clone()
                    })
                } else {
                    format!("ID: {}", &u.id[..12.min(u.id.len())])
                }
            })
            .unwrap_or_else(|| "Please log in".to_string())
    };

    let user_initials = move || {
        auth.user
            .get()
            .as_ref()
            .map(|u| {
                if let Some(phone) = &u.phone {
                    // For phone numbers, use last 2 digits
                    if phone.len() >= 2 {
                        phone[phone.len()-2..].to_string()
                    } else {
                        "U".to_string()
                    }
                } else if let Some(nostr) = &u.nostr {
                    // For nostr addresses, use first 2 characters after 'npub' if present
                    if nostr.starts_with("npub") && nostr.len() > 5 {
                        nostr[4..6].to_uppercase()
                    } else if nostr.len() >= 2 {
                        nostr[..2].to_uppercase()
                    } else {
                        "N".to_string()
                    }
                } else {
                    // For IDs, use first 2 characters
                    if u.id.len() >= 2 {
                        u.id[..2].to_uppercase()
                    } else {
                        "U".to_string()
                    }
                }
            })
            .unwrap_or_else(|| "?".to_string())
    };

    let handle_logout = move |_| {
        set_menu_open.set(false);
        auth.logout.run(());
        // Navigate to login page
        window().location().set_href("/login").unwrap_or_default();
    };

    view! {
        <div class="flex-shrink-0 relative bg-gray-50 border-t border-gray-200 p-4">
            <div class="flex items-center w-full">
                <div class="flex-shrink-0">
                    <div class="h-10 w-10 rounded-full bg-gradient-to-br from-blue-500 to-blue-600 flex items-center justify-center ring-2 ring-white">
                        <span class="text-sm font-bold text-white">{user_initials}</span>
                    </div>
                </div>
                <div class="ml-3 flex-1 min-w-0">
                    <p class="text-sm font-semibold text-gray-900 truncate">{user_name}</p>
                    <p class="text-xs text-gray-500 truncate">{user_contact}</p>
                </div>
                <div class="ml-2">
                    <button
                        type="button"
                        class="flex-shrink-0 p-1.5 text-gray-400 hover:text-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-500 rounded-md transition-colors"
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
                    <div class="absolute bottom-full left-4 right-4 mb-2 bg-white rounded-lg shadow-xl ring-1 ring-black ring-opacity-5 focus:outline-none z-50 border border-gray-200">
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
        </div>
    }
}
