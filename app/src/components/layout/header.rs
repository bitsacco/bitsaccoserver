use super::theme_provider::ThemeToggle;
use crate::contexts::auth::use_auth;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn Header(#[prop(optional)] set_mobile_open: Option<WriteSignal<bool>>) -> impl IntoView {
    let auth = use_auth();
    let location = use_location();
    let (search_query, set_search_query) = signal(String::new());
    let (notifications_open, set_notifications_open) = signal(false);
    let (user_menu_open, set_user_menu_open) = signal(false);

    // Get current page title based on route
    let page_title = Signal::derive(move || {
        let path = location.pathname.get();
        match path.as_str() {
            "/dashboard" => "Dashboard",
            "/members" => "Members",
            "/groups" => "Groups",
            "/shares" => "Shares",
            "/settings" => "Settings",
            _ => "Dashboard",
        }
    });

    // Get user info
    let user_name = move || {
        auth.user
            .get()
            .as_ref()
            .map(|u| u.display_name())
            .unwrap_or_else(|| "Guest User".to_string())
    };

    let user_initials = move || {
        let name = user_name();
        if name.is_empty() {
            "U".to_string()
        } else {
            name.split_whitespace()
                .map(|word| word.chars().next().unwrap_or('U'))
                .take(2)
                .collect::<String>()
                .to_uppercase()
        }
    };

    view! {
        <div class="sticky top-0 z-10 flex-shrink-0 flex h-16 bg-white border-b border-gray-200 shadow-sm">
            // Mobile menu button with enhanced animations
            <button
                type="button"
                class="group px-4 border-r border-gray-200 text-gray-500 hover:text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-blue-500 lg:hidden transition-all duration-200"
                on:click=move |_| {
                    if let Some(setter) = set_mobile_open {
                        setter.set(true);
                    }
                }
            >
                <span class="sr-only">"Open sidebar"</span>
                <div class="relative w-6 h-6">
                    // Animated hamburger menu
                    <svg class="h-6 w-6 transform transition-transform duration-200 group-hover:scale-110" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path class="transform transition-transform duration-200 origin-center group-hover:translate-y-0.5" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16"/>
                        <path class="transition-opacity duration-200 group-hover:opacity-75" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 12h16"/>
                        <path class="transform transition-transform duration-200 origin-center group-hover:-translate-y-0.5" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 18h7"/>
                    </svg>
                </div>
            </button>

            <div class="flex-1 px-4 lg:px-6 flex justify-between items-center">
                // Page title and breadcrumb
                <div class="flex items-center space-x-4">
                    <div class="hidden lg:block">
                        <h1 class="text-2xl font-bold text-gray-900">
                            {page_title}
                        </h1>
                    </div>

                    // Search bar (responsive)
                    <div class="flex-1 flex max-w-xs lg:max-w-md">
                        <div class="relative w-full text-gray-400 focus-within:text-gray-600">
                            <div class="absolute inset-y-0 left-0 flex items-center pointer-events-none pl-3">
                                <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                                </svg>
                            </div>
                            <input
                                class="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-lg text-gray-900 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors"
                                placeholder="Search members, groups, transactions..."
                                type="search"
                                prop:value=move || search_query.get()
                                on:input=move |ev| {
                                    set_search_query.set(event_target_value(&ev));
                                }
                            />
                        </div>
                    </div>
                </div>

                // Right side - Quick actions and user menu
                <div class="flex items-center space-x-3">
                    // Quick action: Add new
                    <button
                        type="button"
                        class="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
                    >
                        <svg class="h-4 w-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                        </svg>
                        "Add New"
                    </button>

                    // Theme toggle
                    <ThemeToggle/>

                    // Notifications
                    <div class="relative">
                        <button
                            type="button"
                            class="relative p-2 text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 rounded-full transition-colors"
                            on:click=move |_| set_notifications_open.update(|open| *open = !*open)
                        >
                            <span class="sr-only">"View notifications"</span>
                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-5 5v-5z" />
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9v-9" />
                            </svg>
                            // Notification badge
                            <span class="absolute -top-0.5 -right-0.5 h-5 w-5 bg-red-500 text-white text-xs rounded-full flex items-center justify-center font-medium">
                                "3"
                            </span>
                        </button>

                        // Notifications dropdown
                        {move || if notifications_open.get() {
                            view! {
                                <NotificationDropdown on_close=move || set_notifications_open.set(false)/>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}
                    </div>

                    // User menu
                    <div class="relative">
                        <button
                            type="button"
                            class="flex items-center space-x-3 text-sm rounded-full focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 p-1.5 hover:bg-gray-50 transition-colors"
                            on:click=move |_| set_user_menu_open.update(|open| *open = !*open)
                        >
                            <div class="h-8 w-8 rounded-full bg-gradient-to-br from-blue-500 to-blue-600 flex items-center justify-center">
                                <span class="text-xs font-bold text-white">{user_initials}</span>
                            </div>
                            <div class="hidden lg:flex lg:flex-col lg:items-start">
                                <div class="text-sm font-medium text-gray-900 truncate max-w-32">
                                    {user_name}
                                </div>
                                <div class="text-xs text-gray-500">
                                    "Administrator"
                                </div>
                            </div>
                            <svg class="h-4 w-4 text-gray-400 hidden lg:block" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                            </svg>
                        </button>

                        // User dropdown menu
                        {
                            let auth_clone = auth.clone();
                            let set_user_menu_open_clone = set_user_menu_open;
                            move || if user_menu_open.get() {
                                let auth_for_dropdown = auth_clone.clone();
                                view! {
                                    <UserDropdown on_close=move || set_user_menu_open_clone.set(false) auth=auth_for_dropdown/>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn NotificationDropdown(on_close: impl Fn() + 'static + Copy) -> impl IntoView {
    view! {
        <div class="origin-top-right absolute right-0 mt-2 w-96 rounded-lg shadow-xl bg-white ring-1 ring-black ring-opacity-5 focus:outline-none z-50 border border-gray-200">
            <div class="p-0">
                <div class="flex items-center justify-between p-4 border-b border-gray-100">
                    <div>
                        <h3 class="text-lg font-semibold text-gray-900">"Notifications"</h3>
                        <p class="text-sm text-gray-500">"3 unread messages"</p>
                    </div>
                    <button
                        type="button"
                        class="text-gray-400 hover:text-gray-500 p-1 rounded-md transition-colors"
                        on:click=move |_| on_close()
                    >
                        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <div class="max-h-96 overflow-y-auto">
                    <NotificationItem
                        title="New member registration"
                        message="John Doe has registered as a new member and is awaiting approval"
                        time="2 minutes ago"
                        unread=true
                        icon_type="user"
                    />
                    <NotificationItem
                        title="Share purchase completed"
                        message="Alice Smith purchased 10 shares for $1,500 total value"
                        time="1 hour ago"
                        unread=true
                        icon_type="money"
                    />
                    <NotificationItem
                        title="System maintenance scheduled"
                        message="Scheduled maintenance will begin at 2:00 AM tonight"
                        time="2 hours ago"
                        unread=true
                        icon_type="warning"
                    />
                    <NotificationItem
                        title="System backup completed"
                        message="Daily backup completed successfully. All data secure"
                        time="3 hours ago"
                        unread=false
                        icon_type="success"
                    />
                    <NotificationItem
                        title="Group meeting reminder"
                        message="Monthly board meeting scheduled for tomorrow at 10 AM"
                        time="5 hours ago"
                        unread=false
                        icon_type="calendar"
                    />
                </div>

                <div class="p-4 border-t border-gray-100 bg-gray-50">
                    <div class="flex justify-between items-center">
                        <button class="text-sm text-blue-600 hover:text-blue-700 font-medium">
                            "Mark all as read"
                        </button>
                        <a href="/notifications" class="text-sm text-blue-600 hover:text-blue-700 font-medium">
                            "View all notifications"
                        </a>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn UserDropdown(
    on_close: impl Fn() + 'static + Copy,
    auth: crate::contexts::auth::AuthContext,
) -> impl IntoView {
    let user_name = move || {
        auth.user
            .get()
            .as_ref()
            .map(|u| u.display_name())
            .unwrap_or_else(|| "Guest User".to_string())
    };

    let user_email = move || {
        auth.user
            .get()
            .as_ref()
            .map(|u| {
                u.phone
                    .clone()
                    .unwrap_or_else(|| "No contact info".to_string())
            })
            .unwrap_or_else(|| "guest@example.com".to_string())
    };

    let handle_logout = move |_| {
        on_close();
        auth.logout.run(());
        let _ = window().location().set_href("/login");
    };

    view! {
        <div class="origin-top-right absolute right-0 mt-2 w-72 rounded-lg shadow-xl bg-white ring-1 ring-black ring-opacity-5 focus:outline-none z-50 border border-gray-200">
            // User info header
            <div class="px-4 py-3 border-b border-gray-100 bg-gradient-to-r from-blue-50 to-indigo-50">
                <div class="flex items-center space-x-3">
                    <div class="h-12 w-12 rounded-full bg-gradient-to-br from-blue-500 to-blue-600 flex items-center justify-center">
                        <span class="text-sm font-bold text-white">
                            {move || {
                                let name = user_name();
                                if name.is_empty() {
                                    "U".to_string()
                                } else {
                                    name.split_whitespace()
                                        .map(|word| word.chars().next().unwrap_or('U'))
                                        .take(2)
                                        .collect::<String>()
                                        .to_uppercase()
                                }
                            }}
                        </span>
                    </div>
                    <div class="flex-1 min-w-0">
                        <p class="text-sm font-semibold text-gray-900 truncate">
                            {user_name}
                        </p>
                        <p class="text-xs text-gray-600 truncate">
                            {user_email}
                        </p>
                        <div class="flex items-center mt-1">
                            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800">
                                "Active"
                            </span>
                        </div>
                    </div>
                </div>
            </div>

            // Menu items
            <div class="py-2">
                <a
                    href="/profile"
                    class="flex items-center px-4 py-3 text-sm text-gray-700 hover:bg-gray-50 transition-colors"
                    on:click=move |_| on_close()
                >
                    <svg class="mr-3 h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                    </svg>
                    <div>
                        <div class="font-medium">"Your Profile"</div>
                        <div class="text-xs text-gray-500">"View and edit your profile"</div>
                    </div>
                </a>

                <a
                    href="/settings"
                    class="flex items-center px-4 py-3 text-sm text-gray-700 hover:bg-gray-50 transition-colors"
                    on:click=move |_| on_close()
                >
                    <svg class="mr-3 h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    <div>
                        <div class="font-medium">"Preferences"</div>
                        <div class="text-xs text-gray-500">"App settings and preferences"</div>
                    </div>
                </a>

                <a
                    href="/help"
                    class="flex items-center px-4 py-3 text-sm text-gray-700 hover:bg-gray-50 transition-colors"
                    on:click=move |_| on_close()
                >
                    <svg class="mr-3 h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    <div>
                        <div class="font-medium">"Help & Support"</div>
                        <div class="text-xs text-gray-500">"Get help and support"</div>
                    </div>
                </a>

                <div class="border-t border-gray-100 my-1"></div>

                <button
                    type="button"
                    class="flex items-center w-full px-4 py-3 text-sm text-red-700 hover:bg-red-50 transition-colors"
                    on:click=handle_logout
                >
                    <svg class="mr-3 h-5 w-5 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                    </svg>
                    <div>
                        <div class="font-medium">"Sign out"</div>
                        <div class="text-xs text-red-500">"End your current session"</div>
                    </div>
                </button>
            </div>
        </div>
    }
}

#[component]
fn NotificationItem(
    title: &'static str,
    message: &'static str,
    time: &'static str,
    unread: bool,
    icon_type: &'static str,
) -> impl IntoView {
    let (icon_bg, icon_svg) = match icon_type {
        "user" => (
            "bg-blue-100",
            view! {
                <svg class="h-4 w-4 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                </svg>
            },
        ),
        "money" => (
            "bg-green-100",
            view! {
                <svg class="h-4 w-4 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1" />
                </svg>
            },
        ),
        "warning" => (
            "bg-yellow-100",
            view! {
                <svg class="h-4 w-4 text-yellow-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.464 0L4.35 16.5c-.77.833.192 2.5 1.732 2.5z" />
                </svg>
            },
        ),
        "success" => (
            "bg-green-100",
            view! {
                <svg class="h-4 w-4 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
            },
        ),
        "calendar" => (
            "bg-purple-100",
            view! {
                <svg class="h-4 w-4 text-purple-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
            },
        ),
        _ => (
            "bg-gray-100",
            view! {
                <svg class="h-4 w-4 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
            },
        ),
    };

    view! {
        <div class={format!("px-4 py-3 hover:bg-gray-50 transition-colors cursor-pointer border-l-4 {}",
            if unread { "border-blue-400 bg-blue-50/30" } else { "border-transparent" }
        )}>
            <div class="flex items-start space-x-3">
                <div class={format!("flex-shrink-0 h-8 w-8 rounded-full {} flex items-center justify-center", icon_bg)}>
                    {icon_svg}
                </div>
                <div class="flex-1 min-w-0">
                    <div class="flex items-center justify-between">
                        <p class={format!("text-sm font-medium truncate {}",
                            if unread { "text-gray-900" } else { "text-gray-700" }
                        )}>
                            {title}
                        </p>
                        <div class="flex items-center space-x-2">
                            {if unread {
                                view! {
                                    <div class="w-2 h-2 bg-blue-500 rounded-full"></div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }}
                            <p class="text-xs text-gray-500 whitespace-nowrap">
                                {time}
                            </p>
                        </div>
                    </div>
                    <p class="text-sm text-gray-600 mt-1 line-clamp-2">
                        {message}
                    </p>
                </div>
            </div>
        </div>
    }
}
