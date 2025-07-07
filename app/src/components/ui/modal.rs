use leptos::prelude::*;

#[component]
pub fn Modal(
    #[prop(optional)] show: Signal<bool>,
    #[prop(optional)] on_close: Option<Callback<()>>,
    #[prop(optional)] title: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    if show.get() {
        view! {
            <div class="fixed inset-0 z-50 overflow-y-auto">
                // Backdrop
                <div
                    class="fixed inset-0 bg-black bg-opacity-50 transition-opacity"
                    on:click=move |_| {
                        if let Some(callback) = on_close {
                            callback.run(());
                        }
                    }
                ></div>

                // Modal content
                <div class="flex min-h-full items-center justify-center p-4">
                    <div class="relative bg-white rounded-lg shadow-xl max-w-lg w-full">
                        // Header
                        {title.map(|title| view! {
                            <div class="flex items-center justify-between p-6 border-b border-gray-200">
                                <h3 class="text-lg font-medium text-gray-900">
                                    {title}
                                </h3>
                                <button
                                    type="button"
                                    class="text-gray-400 hover:text-gray-500"
                                    on:click=move |_| {
                                        if let Some(callback) = on_close {
                                            callback.run(());
                                        }
                                    }
                                >
                                    <span class="sr-only">"Close"</span>
                                    <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                    </svg>
                                </button>
                            </div>
                        })}

                        // Body
                        <div class="p-6">
                            {children()}
                        </div>
                    </div>
                </div>
            </div>
        }.into_any()
    } else {
        view! { <div></div> }.into_any()
    }
}
