use leptos::prelude::*;

#[component]
pub fn Select(
    #[prop(optional)] label: Option<&'static str>,
    #[prop(optional)] value: Signal<String>,
    #[prop(optional)] on_change: Option<Callback<String>>,
    #[prop(optional)] error: Signal<Option<String>>,
    #[prop(optional)] required: bool,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let has_error = Signal::derive(move || error.get().is_some());

    let select_class = Signal::derive(move || {
        let base_class = "block w-full rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm";
        let error_class = if has_error.get() {
            "border-red-300 text-red-900 focus:ring-red-500 focus:border-red-500"
        } else {
            "border-gray-300"
        };
        format!("{} {} {}", base_class, error_class, class.unwrap_or(""))
    });

    view! {
        <div>
            {label.map(|label| view! {
                <label class="block text-sm font-medium text-gray-700 mb-1">
                    {label}
                    {if required { " *" } else { "" }}
                </label>
            })}

            <select
                class=move || select_class.get()
                prop:value=move || value.get()
                required=required
                disabled=disabled
                on:change=move |ev| {
                    if let Some(callback) = on_change {
                        callback.run(event_target_value(&ev));
                    }
                }
            >
                {children()}
            </select>

            <Show when=move || error.get().is_some()>
                <p class="mt-1 text-sm text-red-600">
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>
        </div>
    }
}
