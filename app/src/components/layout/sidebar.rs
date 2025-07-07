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
        // Mobile sidebar overlay
        {let location_clone = location.clone();
        move || if mobile_open.get() {
            view! {
                <div class="fixed inset-0 z-40 lg:hidden">
                    <div
                        class="fixed inset-0 bg-gray-600 bg-opacity-75"
                        on:click=move |_| {
                            if let Some(setter) = set_mobile_open {
                                setter.set(false);
                            }
                        }
                    ></div>
                    <div class="relative flex-1 flex flex-col max-w-xs w-full bg-gray-800">
                        <div class="absolute top-0 right-0 -mr-12 pt-2">
                            <button
                                type="button"
                                class="ml-1 flex items-center justify-center h-10 w-10 rounded-full focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
                                on:click=move |_| {
                                    if let Some(setter) = set_mobile_open {
                                        setter.set(false);
                                    }
                                }
                            >
                                <span class="sr-only">"Close sidebar"</span>
                                <svg class="h-6 w-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        </div>
                        <SidebarContent location=location_clone.clone()/>
                    </div>
                </div>
            }.into_any()
        } else {
            view! { <div></div> }.into_any()
        }}

        // Desktop sidebar
        <div class="hidden lg:flex lg:w-64 lg:flex-col lg:fixed lg:inset-y-0">
            <SidebarContent location=location.clone()/>
        </div>
    }
}

#[component]
fn SidebarContent(location: leptos_router::location::Location) -> impl IntoView {
    let _app_state = use_app_state();
    let auth = use_auth();

    view! {
        <div class="flex-1 flex flex-col min-h-0 bg-gray-800">
            // Logo
            <div class="flex items-center h-16 flex-shrink-0 px-4 bg-gray-900">
                <div class="flex items-center">
                    <div class="flex-shrink-0 h-8 w-8 bg-blue-600 rounded flex items-center justify-center">
                        <span class="text-white font-bold text-sm">"BS"</span>
                    </div>
                    <div class="ml-3">
                        <span class="text-white text-lg font-semibold">"Bitsacco Server"</span>
                    </div>
                </div>
            </div>

            // Navigation
            <div class="flex-1 flex flex-col pt-5 pb-4 overflow-y-auto">
                <nav class="mt-5 flex-1 px-2 space-y-1">
                    <NavItem href="/dashboard" icon="🏠" text="Dashboard" current_path=location.pathname.into()/>
                    <NavItem href="/members" icon="👥" text="Members" current_path=location.pathname.into()/>
                    <NavItem href="/groups" icon="👥" text="Groups" current_path=location.pathname.into()/>
                    <NavItem href="/shares" icon="📊" text="Shares" current_path=location.pathname.into()/>
                    <NavItem href="/settings" icon="⚙️" text="Settings" current_path=location.pathname.into()/>
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
    icon: &'static str,
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
                if is_current() {
                    "bg-gray-900 text-white group flex items-center px-2 py-2 text-sm font-medium rounded-md"
                } else {
                    "text-gray-300 hover:bg-gray-700 hover:text-white group flex items-center px-2 py-2 text-sm font-medium rounded-md"
                }
            }
        >
            <span class="mr-3 text-lg">{icon}</span>
            {text}
        </a>
    }
}

#[component]
fn UserProfile(auth: crate::contexts::auth::AuthContext) -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);
    
    // Get user info or fallback to defaults
    let user_name = move || {
        auth.user.get()
            .as_ref()
            .map(|u| u.full_name())
            .unwrap_or_else(|| "Guest User".to_string())
    };
    
    let user_email = move || {
        auth.user.get()
            .as_ref()
            .map(|u| u.email.clone())
            .unwrap_or_else(|| "guest@example.com".to_string())
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

    let handle_logout = move |_| {
        set_menu_open.set(false);
        auth.logout.run(());
        // Navigate to login page
        window().location().set_href("/login").unwrap_or_default();
    };

    view! {
        <div class="flex-shrink-0 relative bg-gray-700 p-4">
            <div class="flex items-center w-full">
                <div class="flex-shrink-0">
                    <div class="h-8 w-8 rounded-full bg-blue-100 flex items-center justify-center">
                        <span class="text-sm font-medium text-blue-600">{user_initials}</span>
                    </div>
                </div>
                <div class="ml-3 flex-1">
                    <p class="text-sm font-medium text-white">{user_name}</p>
                    <p class="text-xs text-gray-300">{user_email}</p>
                </div>
                <div class="ml-2">
                    <button
                        type="button"
                        class="flex-shrink-0 p-1 text-gray-400 hover:text-gray-300 focus:outline-none focus:ring-2 focus:ring-white rounded"
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
                    <div class="absolute bottom-full left-4 right-4 mb-2 bg-white rounded-md shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none z-50">
                        <div class="py-1">
                            <button
                                type="button"
                                class="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 flex items-center"
                                on:click=handle_logout
                            >
                                <svg class="mr-3 h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
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
