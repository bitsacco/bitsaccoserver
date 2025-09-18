use leptos::prelude::*;
use wasm_bindgen::prelude::*;

// Client-side only mobile sidebar component
#[component]
pub fn MobileSidebar() -> impl IntoView {
    let (is_open, set_is_open) = signal(false);

    // Use effect to add click handlers via JavaScript directly
    Effect::new(move |_| {
        if let Ok(window) = web_sys::window() {
            if let Ok(document) = window.document() {
                // Add hamburger button click handler
                if let Ok(Some(hamburger)) = document.query_selector("button[aria-label='Open sidebar']") {
                    let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                        set_is_open.set(true);
                    }) as Box<dyn Fn(_)>);

                    let _ = hamburger.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
                    closure.forget(); // Keep the closure alive
                }
            }
        }
    });

    view! {
        // Mobile sidebar overlay - only render if open
        {move || {
            if is_open.get() {
                view! {
                    <div class="fixed inset-0 z-40 lg:hidden" id="mobile-sidebar-overlay">
                        // Backdrop
                        <div
                            class="fixed inset-0 bg-gray-600 opacity-75 transition-opacity duration-300 ease-in-out"
                            on:click=move |_| set_is_open.set(false)
                        ></div>

                        // Sliding sidebar panel
                        <div class="relative flex-1 flex flex-col max-w-xs w-full bg-white shadow-xl border-r border-gray-200 transform transition-transform duration-300 ease-in-out">
                            // Close button
                            <div class="absolute top-0 right-0 -mr-12 pt-2">
                                <button
                                    type="button"
                                    class="ml-1 flex items-center justify-center h-10 w-10 rounded-full bg-black/20 backdrop-blur-sm focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white transition-all duration-200 hover:bg-black/30 hover:scale-105"
                                    on:click=move |_| set_is_open.set(false)
                                >
                                    <span class="sr-only">"Close sidebar"</span>
                                    <svg class="h-6 w-6 text-white transform transition-transform duration-200 hover:rotate-90" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                    </svg>
                                </button>
                            </div>

                            // Sidebar content
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

                                // Navigation
                                <nav class="flex-1 px-4 space-y-2 pt-6 pb-4 overflow-y-auto">
                                    <a href="/dashboard" class="group flex items-center px-3 py-2.5 text-sm font-medium rounded-lg text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 hover:text-gray-900 hover:shadow-sm transition-all duration-200"
                                       on:click=move |_| set_is_open.set(false)>
                                        <svg class="w-5 h-5 mr-3 text-gray-400 group-hover:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2z" />
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 5a2 2 0 012-2h4a2 2 0 012 2v4H8V5z" />
                                        </svg>
                                        <span class="font-medium">"Dashboard"</span>
                                    </a>

                                    <a href="/members" class="group flex items-center px-3 py-2.5 text-sm font-medium rounded-lg text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 hover:text-gray-900 hover:shadow-sm transition-all duration-200"
                                       on:click=move |_| set_is_open.set(false)>
                                        <svg class="w-5 h-5 mr-3 text-gray-400 group-hover:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z" />
                                        </svg>
                                        <span class="font-medium">"Members"</span>
                                    </a>

                                    <a href="/groups" class="group flex items-center px-3 py-2.5 text-sm font-medium rounded-lg text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 hover:text-gray-900 hover:shadow-sm transition-all duration-200"
                                       on:click=move |_| set_is_open.set(false)>
                                        <svg class="w-5 h-5 mr-3 text-gray-400 group-hover:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                                        </svg>
                                        <span class="font-medium">"Groups"</span>
                                    </a>

                                    <a href="/shares" class="group flex items-center px-3 py-2.5 text-sm font-medium rounded-lg text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 hover:text-gray-900 hover:shadow-sm transition-all duration-200"
                                       on:click=move |_| set_is_open.set(false)>
                                        <svg class="w-5 h-5 mr-3 text-gray-400 group-hover:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                                        </svg>
                                        <span class="font-medium">"Shares"</span>
                                    </a>

                                    <div class="pt-4">
                                        <div class="text-xs font-semibold text-gray-400 uppercase tracking-wider px-2 mb-2">
                                            "Settings"
                                        </div>
                                        <a href="/settings" class="group flex items-center px-3 py-2.5 text-sm font-medium rounded-lg text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 hover:text-gray-900 hover:shadow-sm transition-all duration-200"
                                           on:click=move |_| set_is_open.set(false)>
                                            <svg class="w-5 h-5 mr-3 text-gray-400 group-hover:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                            </svg>
                                            <span class="font-medium">"System Settings"</span>
                                        </a>
                                    </div>
                                </nav>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }
        }}
    }
}