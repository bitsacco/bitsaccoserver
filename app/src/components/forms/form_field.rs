use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Copy)]
pub enum ValidationState {
    None,
    Validating,
    Valid,
    Invalid,
}

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
    #[prop(optional)] debounce_ms: Option<u64>,
    #[prop(optional)] show_success: bool,
) -> impl IntoView {
    let (error, set_error) = signal(None::<String>);
    let (touched, set_touched) = signal(false);
    let (validation_state, set_validation_state) = signal(ValidationState::None);
    let debounce_timeout = debounce_ms.unwrap_or(300);

    // Debounced validation effect
    let label_clone = label.clone();
    Effect::new(move |_| {
        let current_value = value.get();
        if touched.get() {
            set_validation_state.set(ValidationState::Validating);

            let current_value_clone = current_value.clone();
            let label_for_validation = label_clone.clone();

            spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(debounce_timeout as u32).await;

                // Re-check if value has changed during debounce
                if current_value_clone == value.get() {
                    // Inline validation logic
                    let mut validation_error = None;

                    // Check required
                    if required && current_value_clone.trim().is_empty() {
                        validation_error = Some(format!("{} is required", label_for_validation));
                    }

                    // Apply custom validation rules (temporarily disabled due to FnMut trait bounds)
                    // TODO: Fix custom validation rules implementation
                    // if validation_error.is_none() {
                    //     if let Some(rules) = &validation_rules {
                    //         for rule in rules {
                    //             if let Some(error) = rule(&current_value_clone) {
                    //                 validation_error = Some(error);
                    //                 break;
                    //             }
                    //         }
                    //     }
                    // }

                    set_error.set(validation_error.clone());

                    if validation_error.is_some() {
                        set_validation_state.set(ValidationState::Invalid);
                    } else {
                        set_validation_state.set(ValidationState::Valid);
                    }
                }
            });
        }
    });

    view! {
        <div class="space-y-1">
            <label class="block text-sm font-medium text-gray-700">
                {label.clone()}
                {if required { " *" } else { "" }}
            </label>

            <div class="relative">
                <input
                    type=type_.unwrap_or("text")
                    name=name
                    class={
                        let base_classes = "block w-full rounded-md shadow-sm focus:outline-none focus:ring-2 sm:text-sm pr-10 transition-all duration-200";
                        move || {
                            match (error.get().is_some(), validation_state.get()) {
                                (true, _) => format!(
                                    "{} border-red-300 text-red-900 placeholder-red-300 focus:ring-red-500 focus:border-red-500",
                                    base_classes
                                ),
                                (false, ValidationState::Valid) if show_success => format!(
                                    "{} border-green-300 focus:ring-green-500 focus:border-green-500",
                                    base_classes
                                ),
                                (false, ValidationState::Validating) => format!(
                                    "{} border-blue-300 focus:ring-blue-500 focus:border-blue-500",
                                    base_classes
                                ),
                                _ => format!(
                                    "{} border-gray-300 focus:ring-blue-500 focus:border-blue-500",
                                    base_classes
                                ),
                            }
                        }
                    }
                    placeholder=placeholder.unwrap_or_default()
                    prop:value=move || value.get()
                    required=required
                    disabled=disabled
                    on:input=move |ev| {
                        set_value.set(event_target_value(&ev));
                    }
                    on:blur=move |_| {
                        set_touched.set(true);
                    }
                />

                // Validation state indicator
                <div class="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none">
                    {
                        move || match validation_state.get() {
                            ValidationState::Validating => view! {
                                <svg class="h-4 w-4 text-blue-500 animate-spin" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
                                </svg>
                            }.into_any(),
                            ValidationState::Valid if show_success => view! {
                                <svg class="h-4 w-4 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                                </svg>
                            }.into_any(),
                            ValidationState::Invalid => view! {
                                <svg class="h-4 w-4 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                                </svg>
                            }.into_any(),
                            _ => view! { <div></div> }.into_any(),
                        }
                    }
                </div>
            </div>

            {if let Some(help) = help_text {
                view! {
                    <p class="text-sm text-gray-500">{help}</p>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            <Show when=move || error.get().is_some()>
                <p class="text-sm text-red-600 mt-1 flex items-center">
                    <svg class="h-4 w-4 mr-1 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                    </svg>
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>

            <Show when=move || show_success && validation_state.get() as u8 == ValidationState::Valid as u8 && error.get().is_none()>
                <p class="text-sm text-green-600 mt-1 flex items-center">
                    <svg class="h-4 w-4 mr-1 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                    </svg>
                    "Looks good!"
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

// Textarea component
#[component]
pub fn TextareaField(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    #[prop(optional)] placeholder: Option<String>,
    #[prop(into)] value: Signal<String>,
    #[prop(into)] set_value: WriteSignal<String>,
    #[prop(optional)] required: bool,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] rows: Option<u32>,
    #[prop(optional)] help_text: Option<String>,
) -> impl IntoView {
    let (error, set_error) = signal(None::<String>);
    let (touched, set_touched) = signal(false);
    let rows = rows.unwrap_or(3);

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

            <textarea
                name=name
                rows=rows
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
                on:blur=move |_| {
                    set_touched.set(true);
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
                <p class="text-sm text-red-600 mt-1 flex items-center">
                    <svg class="h-4 w-4 mr-1 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                    </svg>
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>
        </div>
    }
}
