use leptos::prelude::*;

#[component]
pub fn Checkbox(
    #[prop(optional)] label: Option<&'static str>,
    #[prop(optional)] checked: Signal<bool>,
    #[prop(optional)] on_change: Option<Callback<bool>>,
    #[prop(optional)] error: Signal<Option<String>>,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] description: Option<&'static str>,
) -> impl IntoView {
    let has_error = Signal::derive(move || error.get().is_some());

    let checkbox_class = Signal::derive(move || {
        let base_class = "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded";
        let error_class = if has_error.get() {
            "border-red-300 focus:ring-red-500"
        } else {
            ""
        };
        format!("{} {} {}", base_class, error_class, class.unwrap_or(""))
    });

    view! {
        <div class="relative flex items-start">
            <div class="flex items-center h-5">
                <input
                    type="checkbox"
                    class=move || checkbox_class.get()
                    prop:checked=move || checked.get()
                    disabled=disabled
                    on:change=move |ev| {
                        if let Some(callback) = on_change {
                            callback.run(event_target_checked(&ev));
                        }
                    }
                />
            </div>

            {if label.is_some() || description.is_some() {
                view! {
                    <div class="ml-3 text-sm">
                        {label.map(|label| view! {
                            <label class=format!(
                                "font-medium {}",
                                if has_error.get() { "text-red-900" } else { "text-gray-700" }
                            )>
                                {label}
                            </label>
                        })}

                        {description.map(|desc| view! {
                            <p class=format!(
                                "{}",
                                if has_error.get() { "text-red-600" } else { "text-gray-500" }
                            )>
                                {desc}
                            </p>
                        })}
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            <Show when=move || error.get().is_some()>
                <p class="mt-1 text-sm text-red-600">
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>
        </div>
    }
}
