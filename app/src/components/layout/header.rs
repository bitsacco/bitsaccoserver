use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn Header(#[prop(optional)] set_mobile_open: Option<WriteSignal<bool>>) -> impl IntoView {
    let location = use_location();
    let (search_query, set_search_query) = signal(String::new());
    let (notifications_open, set_notifications_open) = signal(false);
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

                // Right side - actions
                <div class="flex items-center space-x-3">
                    // Notifications
                    <div class="relative">
                        <button
                            type="button"
                            class="relative p-2 text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 rounded-full transition-colors"
                            on:click=move |_| set_notifications_open.update(|open| *open = !*open)
                        >
                            <span class="sr-only">"View notifications"</span>
                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
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
