use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[derive(Clone, Debug)]
pub struct NavItem {
    pub key: String,
    pub title: String,
    pub href: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub children: Option<Vec<NavItem>>,
}

#[component]
pub fn Navigation(
    #[prop(into)] items: Signal<Vec<NavItem>>,
    #[prop(optional)] mobile: bool,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    view! {
        <nav class=format!(
            "space-y-1 {}",
            class.unwrap_or_default()
        )>
            <For
                each=move || items.get()
                key=|item| item.key.clone()
                children=move |item| {
                    view! {
                        <NavItemComponent item=item mobile=mobile />
                    }
                }
            />
        </nav>
    }
}

#[component]
pub fn NavItemComponent(item: NavItem, #[prop(optional)] mobile: bool) -> impl IntoView {
    let location = use_location();
    let (expanded, set_expanded) = signal(false);

    let is_active = Signal::derive({
        let href = item.href.clone();
        move || {
            let pathname = location.pathname.get();
            pathname == href || (href != "/" && pathname.starts_with(&href))
        }
    });

    let has_children = item.children.is_some();
    let item_clone = item.clone();

    view! {
        <div>
            {if has_children {
                view! {
                    <button
                        class=format!(
                            "w-full flex items-center justify-between px-2 py-2 text-sm font-medium rounded-md {} {}",
                            if is_active.get() {
                                "bg-blue-100 text-blue-900"
                            } else {
                                "text-gray-600 hover:bg-gray-50 hover:text-gray-900"
                            },
                            if mobile { "px-3 py-2" } else { "" }
                        )
                        on:click=move |_| set_expanded.update(|e| *e = !*e)
                    >
                        <div class="flex items-center">
                            {if let Some(icon) = &item.icon {
                                view! {
                                    <span class="mr-3 text-lg">{icon.clone()}</span>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }}
                            <span>{item.title.clone()}</span>
                        </div>

                        <div class="flex items-center">
                            {if let Some(badge) = &item.badge {
                                view! {
                                    <span class="mr-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                                        {badge.clone()}
                                    </span>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }}

                            <svg
                                class=format!(
                                    "h-5 w-5 transform transition-transform duration-200 {}",
                                    if expanded.get() { "rotate-90" } else { "" }
                                )
                                viewBox="0 0 20 20"
                                fill="currentColor"
                            >
                                <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
                            </svg>
                        </div>
                    </button>

                    <Show when=move || expanded.get()>
                        <div class="mt-1 space-y-1">
                            {if let Some(children) = &item.children {
                                children.iter().map(|child| {
                                    let child = child.clone();
                                    view! {
                                        <NavItemComponent item=child mobile=mobile />
                                    }
                                }).collect::<Vec<_>>()
                            } else {
                                vec![]
                            }}
                        </div>
                    </Show>
                }.into_any()
            } else {
                view! {
                    <a
                        href=item_clone.href
                        class=format!(
                            "flex items-center px-2 py-2 text-sm font-medium rounded-md {} {}",
                            if is_active.get() {
                                "bg-blue-100 text-blue-900"
                            } else {
                                "text-gray-600 hover:bg-gray-50 hover:text-gray-900"
                            },
                            if mobile { "px-3 py-2" } else { "" }
                        )
                    >
                        {if let Some(icon) = &item_clone.icon {
                            view! {
                                <span class="mr-3 text-lg">{icon.clone()}</span>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}

                        <span class="flex-1">{item_clone.title}</span>

                        {if let Some(badge) = &item_clone.badge {
                            view! {
                                <span class="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                                    {badge.clone()}
                                </span>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}
                    </a>
                }.into_any()
            }}
        </div>
    }
}

#[derive(Clone, Debug)]
pub struct BreadcrumbItem {
    pub title: String,
    pub href: String,
}

#[component]
pub fn Breadcrumbs(
    #[prop(into)] items: Signal<Vec<BreadcrumbItem>>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    view! {
        <nav class=format!("flex {}", class.unwrap_or_default()) aria-label="Breadcrumb">
            <ol class="inline-flex items-center space-x-1 md:space-x-3">
                {move || {
                    items.get().into_iter().enumerate().map(|(index, item)| {
                        let is_last = index == items.get().len() - 1;

                        view! {
                            <li class="inline-flex items-center">
                                {if index > 0 {
                                    view! {
                                        <svg class="w-6 h-6 text-gray-400" fill="currentColor" viewBox="0 0 20 20">
                                            <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"></path>
                                        </svg>
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }}

                                {if is_last {
                                    view! {
                                        <span class="ml-1 text-sm font-medium text-gray-500 md:ml-2">
                                            {item.title}
                                        </span>
                                    }.into_any()
                                } else {
                                    view! {
                                        <a
                                            href=item.href
                                            class="ml-1 text-sm font-medium text-gray-700 hover:text-blue-600 md:ml-2"
                                        >
                                            {item.title}
                                        </a>
                                    }.into_any()
                                }}
                            </li>
                        }
                    }).collect::<Vec<_>>()
                }}
            </ol>
        </nav>
    }
}

#[component]
pub fn MobileMenu(
    #[prop(into)] is_open: Signal<bool>,
    #[prop(into)] set_is_open: WriteSignal<bool>,
    #[prop(into)] items: Signal<Vec<NavItem>>,
) -> impl IntoView {
    view! {
        // Mobile menu overlay
        <Show when=move || is_open.get()>
            <div class="fixed inset-0 z-50 lg:hidden">
                // Background overlay
                <div
                    class="fixed inset-0 bg-black bg-opacity-25"
                    on:click=move |_| set_is_open.set(false)
                ></div>

                // Slide-out menu
                <div class="relative flex w-full max-w-xs flex-col overflow-y-auto bg-white pb-12 shadow-xl">
                    <div class="flex px-4 pb-2 pt-5">
                        <button
                            type="button"
                            class="-m-2 inline-flex items-center justify-center rounded-md p-2 text-gray-400"
                            on:click=move |_| set_is_open.set(false)
                        >
                            <span class="sr-only">"Close menu"</span>
                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>

                    <div class="mt-2 px-4">
                        <Navigation items=items mobile=true />
                    </div>
                </div>
            </div>
        </Show>
    }
}
