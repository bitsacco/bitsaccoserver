use super::theme_provider::ThemeToggle;
use leptos::prelude::*;

#[component]
pub fn Header(#[prop(optional)] set_mobile_open: Option<WriteSignal<bool>>) -> impl IntoView {
    let (search_query, set_search_query) = signal(String::new());
    let (notifications_open, set_notifications_open) = signal(false);

    view! {
        <div class="sticky top-0 z-10 flex-shrink-0 flex h-16 bg-white shadow">
            // Mobile menu button
            <button
                type="button"
                class="px-4 border-r border-gray-200 text-gray-500 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-blue-500 lg:hidden"
                on:click=move |_| {
                    if let Some(setter) = set_mobile_open {
                        setter.set(true);
                    }
                }
            >
                <span class="sr-only">"Open sidebar"</span>
                <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7" />
                </svg>
            </button>

            <div class="flex-1 px-4 flex justify-between">
                // Search bar (responsive)
                <div class="flex-1 flex max-w-xs lg:max-w-none">
                    <div class="relative w-full text-gray-400 focus-within:text-gray-600">
                        <div class="absolute inset-y-0 left-0 flex items-center pointer-events-none">
                            <svg class="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
                            </svg>
                        </div>
                        <input
                            class="block w-full h-full pl-8 pr-3 py-2 border-transparent text-gray-900 placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-0 focus:border-transparent"
                            placeholder="Search members, groups, transactions..."
                            type="search"
                            prop:value=move || search_query.get()
                            on:input=move |ev| {
                                set_search_query.set(event_target_value(&ev));
                            }
                        />
                    </div>
                </div>

                // Right side
                <div class="ml-4 flex items-center space-x-4">
                    // Theme toggle
                    <ThemeToggle/>

                    // Notifications
                    <div class="relative">
                        <button
                            type="button"
                            class="bg-white p-1 rounded-full text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                            on:click=move |_| set_notifications_open.update(|open| *open = !*open)
                        >
                            <span class="sr-only">"View notifications"</span>
                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-5 5v-5z" />
                            </svg>
                            // Notification badge
                            <span class="absolute -top-1 -right-1 h-4 w-4 bg-red-500 text-white text-xs rounded-full flex items-center justify-center">
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
        <div class="origin-top-right absolute right-0 mt-2 w-80 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 focus:outline-none z-50">
            <div class="p-4">
                <div class="flex items-center justify-between mb-3">
                    <h3 class="text-lg font-medium text-gray-900">"Notifications"</h3>
                    <button
                        type="button"
                        class="text-gray-400 hover:text-gray-500"
                        on:click=move |_| on_close()
                    >
                        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>
                <div class="space-y-3">
                    <NotificationItem
                        title="New member registration"
                        message="John Doe has registered as a new member"
                        time="2 minutes ago"
                        unread=true
                    />
                    <NotificationItem
                        title="Share purchase completed"
                        message="Alice Smith purchased 10 shares"
                        time="1 hour ago"
                        unread=true
                    />
                    <NotificationItem
                        title="System backup completed"
                        message="Daily backup completed successfully"
                        time="3 hours ago"
                        unread=false
                    />
                </div>
                <div class="mt-4 pt-3 border-t border-gray-200">
                    <a href="/notifications" class="text-sm text-blue-600 hover:text-blue-700">
                        "View all notifications"
                    </a>
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
) -> impl IntoView {
    view! {
        <div class={format!("p-3 rounded-lg {}", if unread { "bg-blue-50" } else { "bg-gray-50" })}>
            <div class="flex items-start">
                {if unread {
                    view! {
                        <div class="flex-shrink-0 w-2 h-2 bg-blue-500 rounded-full mt-2 mr-3"></div>
                    }.into_any()
                } else {
                    view! { <div class="w-5 mr-3"></div> }.into_any()
                }}
                <div class="flex-1">
                    <p class="text-sm font-medium text-gray-900">{title}</p>
                    <p class="text-sm text-gray-600">{message}</p>
                    <p class="text-xs text-gray-500 mt-1">{time}</p>
                </div>
            </div>
        </div>
    }
}
