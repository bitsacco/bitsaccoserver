use leptos::prelude::*;

#[component]
pub fn FormField(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    #[prop(optional)] type_: Option<&'static str>,
    #[prop(optional)] placeholder: Option<String>,
    #[prop(into)] value: Signal<String>,
    #[prop(into)] set_value: WriteSignal<String>,
    #[prop(optional)] required: bool,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] _validation_rules: Option<Vec<Box<dyn Fn(&str) -> Option<String>>>>,
    #[prop(optional)] help_text: Option<String>,
) -> impl IntoView {
    let (error, set_error) = signal(None::<String>);
    let (touched, set_touched) = signal(false);

    // Validation will be done inline to avoid closure capture issues

    // Watch for value changes to validate
    {
        let label_clone = label.clone();
        Effect::new(move |_| {
            let _ = value.get();
            if touched.get() {
                let current_value = value.get();
                let mut validation_error = None;

                // Check required
                if required && current_value.trim().is_empty() {
                    validation_error = Some(format!("{} is required", label_clone));
                }

                set_error.set(validation_error);
            }
        });
    }

    view! {
        <div class="space-y-1">
            <label class="block text-sm font-medium text-gray-700">
                {label.clone()}
                {if required { " *" } else { "" }}
            </label>

            <input
                type=type_.unwrap_or("text")
                name=name
                class=format!(
                    "block w-full rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm {}",
                    if error.get().is_some() {
                        "border-red-300 text-red-900 placeholder-red-300 focus:ring-red-500 focus:border-red-500"
                    } else {
                        "border-gray-300"
                    }
                )
                placeholder=placeholder.unwrap_or_default()
                prop:value=move || value.get()
                required=required
                disabled=disabled
                on:input=move |ev| {
                    set_value.set(event_target_value(&ev));
                }
                on:blur={
                    let label_clone = label.clone();
                    move |_| {
                        set_touched.set(true);
                        if touched.get() {
                            let current_value = value.get();
                            let mut validation_error = None;

                            // Check required
                            if required && current_value.trim().is_empty() {
                                validation_error = Some(format!("{} is required", label_clone));
                }

                set_error.set(validation_error);
            }
                    }
                }
            />

            {if let Some(help) = help_text {
                view! {
                    <p class="text-sm text-gray-500">{help}</p>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            <Show when=move || error.get().is_some()>
                <p class="text-sm text-red-600">
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>
        </div>
    }
}

#[component]
pub fn SelectField(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    #[prop(into)] value: Signal<String>,
    #[prop(into)] set_value: WriteSignal<String>,
    #[prop(optional)] required: bool,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] help_text: Option<String>,
    children: Children,
) -> impl IntoView {
    let (error, set_error) = signal(None::<String>);
    let (touched, set_touched) = signal(false);

    // Validation will be done inline to avoid closure capture issues

    // Watch for value changes to validate
    {
        let label_clone = label.clone();
        Effect::new(move |_| {
            let _ = value.get();
            if touched.get() {
                let current_value = value.get();
                let mut validation_error = None;

                // Check required
                if required && current_value.trim().is_empty() {
                    validation_error = Some(format!("{} is required", label_clone));
                }

                set_error.set(validation_error);
            }
        });
    }

    view! {
        <div class="space-y-1">
            <label class="block text-sm font-medium text-gray-700">
                {label.clone()}
                {if required { " *" } else { "" }}
            </label>

            <select
                name=name
                class=format!(
                    "block w-full rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm {}",
                    if error.get().is_some() {
                        "border-red-300 text-red-900 focus:ring-red-500 focus:border-red-500"
                    } else {
                        "border-gray-300"
                    }
                )
                prop:value=move || value.get()
                required=required
                disabled=disabled
                on:change={
                    let label_clone = label.clone();
                    move |ev| {
                        set_value.set(event_target_value(&ev));
                        set_touched.set(true);
                        if touched.get() {
                            let current_value = value.get();
                            let mut validation_error = None;

                            // Check required
                            if required && current_value.trim().is_empty() {
                                validation_error = Some(format!("{} is required", label_clone));
                }

                set_error.set(validation_error);
            }
                    }
                }
            >
                {children()}
            </select>

            {if let Some(help) = help_text {
                view! {
                    <p class="text-sm text-gray-500">{help}</p>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            <Show when=move || error.get().is_some()>
                <p class="text-sm text-red-600">
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>
        </div>
    }
}

#[component]
pub fn CheckboxField(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] set_checked: WriteSignal<bool>,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] help_text: Option<String>,
) -> impl IntoView {
    view! {
        <div class="space-y-1">
            <div class="flex items-center">
                <input
                    type="checkbox"
                    name=name
                    class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                    prop:checked=move || checked.get()
                    disabled=disabled
                    on:change=move |ev| {
                        set_checked.set(event_target_checked(&ev));
                    }
                />
                <label class="ml-2 block text-sm text-gray-900">
                    {label}
                </label>
            </div>

            {if let Some(help) = help_text {
                view! {
                    <p class="text-sm text-gray-500 ml-6">{help}</p>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}
