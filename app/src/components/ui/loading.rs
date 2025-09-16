use leptos::prelude::*;

#[derive(Clone, Copy)]
pub enum LoadingSize {
    Small,
    Medium,
    Large,
}

impl LoadingSize {
    fn to_class(&self) -> &'static str {
        match self {
            LoadingSize::Small => "h-4 w-4",
            LoadingSize::Medium => "h-8 w-8",
            LoadingSize::Large => "h-12 w-12",
        }
    }
}

#[component]
pub fn Spinner(
    #[prop(optional)] size: Option<LoadingSize>,
    #[prop(optional)] color: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let size = size.unwrap_or(LoadingSize::Medium);
    let color = color.unwrap_or("text-blue-600");

    view! {
        <svg
            class=format!("animate-spin {} {} {}", size.to_class(), color, class.unwrap_or(""))
            fill="none"
            viewBox="0 0 24 24"
        >
            <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
            />
            <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
        </svg>
    }
}

#[component]
pub fn LoadingDots(
    #[prop(optional)] size: Option<&'static str>,
    #[prop(optional)] color: Option<&'static str>,
) -> impl IntoView {
    let size = size.unwrap_or("w-1.5 h-1.5");
    let color = color.unwrap_or("bg-blue-600");

    view! {
        <div class="flex items-center space-x-1">
            <div class=format!("{} {} rounded-full animate-pulse", size, color) style="animation-delay: 0s;"></div>
            <div class=format!("{} {} rounded-full animate-pulse", size, color) style="animation-delay: 0.1s;"></div>
            <div class=format!("{} {} rounded-full animate-pulse", size, color) style="animation-delay: 0.2s;"></div>
        </div>
    }
}

#[component]
pub fn LoadingButton(
    #[prop(into)] text: String,
    #[prop(optional)] loading_text: Option<String>,
    #[prop(optional)] loading: bool,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] onclick: Option<Callback<()>>,
    #[prop(optional)] type_: Option<&'static str>,
    #[prop(optional)] variant: Option<crate::components::ui::button::ButtonVariant>,
    #[prop(optional)] size: Option<crate::components::ui::button::ButtonSize>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let loading_text = loading_text.unwrap_or_else(|| "Loading...".to_string());
    let is_disabled = loading || disabled;

    view! {
        <button
            type=type_.unwrap_or("button")
            class={
                let variant = variant.unwrap_or(crate::components::ui::button::ButtonVariant::Primary);
                let size = size.unwrap_or(crate::components::ui::button::ButtonSize::Medium);
                let base_class = "inline-flex items-center justify-center rounded-md border border-transparent font-medium focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-all duration-200";
                let variant_class = match variant {
                    crate::components::ui::button::ButtonVariant::Primary => "bg-blue-600 hover:bg-blue-700 text-white",
                    crate::components::ui::button::ButtonVariant::Secondary => "bg-gray-200 hover:bg-gray-300 text-gray-900",
                    crate::components::ui::button::ButtonVariant::Danger => "bg-red-600 hover:bg-red-700 text-white",
                    crate::components::ui::button::ButtonVariant::Success => "bg-green-600 hover:bg-green-700 text-white",
                };
                let size_class = match size {
                    crate::components::ui::button::ButtonSize::Small => "px-3 py-1.5 text-sm",
                    crate::components::ui::button::ButtonSize::Medium => "px-4 py-2 text-sm",
                    crate::components::ui::button::ButtonSize::Large => "px-6 py-3 text-base",
                };
                let disabled_class = if is_disabled { "opacity-50 cursor-not-allowed" } else { "" };

                format!("{} {} {} {} {}", base_class, variant_class, size_class, disabled_class, class.unwrap_or(""))
            }
            disabled=is_disabled
            on:click=move |_| {
                if !is_disabled {
                    if let Some(cb) = onclick {
                        cb.run(());
                    }
                }
            }
        >
            <Show
                when=move || loading
                fallback=move || view! { {text.clone()} }
            >
                <div class="flex items-center">
                    <Spinner size=LoadingSize::Small class="-ml-1 mr-2" />
                    {loading_text.clone()}
                </div>
            </Show>
        </button>
    }
}

#[component]
pub fn LoadingCard(
    #[prop(optional)] title: Option<String>,
    #[prop(optional)] message: Option<String>,
    #[prop(optional)] show_spinner: bool,
) -> impl IntoView {
    let show_spinner = show_spinner;

    view! {
        <div class="bg-white rounded-lg shadow-sm border p-6 text-center">
            {if show_spinner {
                view! {
                    <div class="mb-4">
                        <Spinner size=LoadingSize::Large class="mx-auto" />
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            {if let Some(title_text) = title {
                view! {
                    <h3 class="text-lg font-medium text-gray-900 mb-2">
                        {title_text}
                    </h3>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            {if let Some(message_text) = message {
                view! {
                    <p class="text-gray-600">
                        {message_text}
                    </p>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}

#[component]
pub fn ProgressBar(
    #[prop(into)] progress: Signal<f64>, // 0.0 to 1.0
    #[prop(optional)] color: Option<&'static str>,
    #[prop(optional)] background_color: Option<&'static str>,
    #[prop(optional)] height: Option<&'static str>,
    #[prop(optional)] show_percentage: bool,
    #[prop(optional)] animated: bool,
) -> impl IntoView {
    let color = color.unwrap_or("bg-blue-600");
    let background_color = background_color.unwrap_or("bg-gray-200");
    let height = height.unwrap_or("h-2");

    view! {
        <div class="w-full">
            <div class=format!("relative {} rounded-full {}", height, background_color)>
                <div
                    class=format!("h-full rounded-full transition-all duration-300 ease-out {} {}",
                        color,
                        if animated { "bg-gradient-to-r from-blue-400 to-blue-600" } else { "" }
                    )
                    style=move || format!("width: {}%", (progress.get() * 100.0).clamp(0.0, 100.0))
                >
                    {if animated {
                        view! {
                            <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white to-transparent opacity-20 animate-pulse"></div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }}
                </div>
            </div>

            {if show_percentage {
                view! {
                    <div class="text-sm text-gray-600 text-center mt-2">
                        {move || format!("{}%", (progress.get() * 100.0).round() as i32)}
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}

#[component]
pub fn LoadingOverlay(
    #[prop(optional)] message: Option<String>,
    #[prop(optional)] show_backdrop: bool,
) -> impl IntoView {
    let show_backdrop = show_backdrop;
    let message = message.unwrap_or_else(|| "Loading...".to_string());

    view! {
        <div class=format!(
            "fixed inset-0 z-50 flex items-center justify-center {}",
            if show_backdrop { "bg-black bg-opacity-50" } else { "pointer-events-none" }
        )>
            <div class="bg-white rounded-lg p-6 shadow-xl max-w-sm w-full mx-4 pointer-events-auto">
                <div class="text-center">
                    <Spinner size=LoadingSize::Large class="mx-auto mb-4" />
                    <p class="text-gray-700 font-medium">{message}</p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn SkeletonLoader(
    #[prop(optional)] lines: Option<u32>,
    #[prop(optional)] width: Option<&'static str>,
    #[prop(optional)] height: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let lines = lines.unwrap_or(3);
    let width = width.unwrap_or("w-full");
    let height = height.unwrap_or("h-4");

    view! {
        <div class=format!("space-y-3 {}", class.unwrap_or(""))>
            {(0..lines).map(|i| {
                let line_width = if i == lines - 1 && lines > 1 {
                    "w-3/4" // Last line shorter
                } else {
                    width
                };

                view! {
                    <div class=format!("{} {} bg-gray-200 rounded animate-pulse", line_width, height)></div>
                }
            }).collect_view()}
        </div>
    }
}

// Utility function to create loading state
pub fn use_loading() -> (ReadSignal<bool>, WriteSignal<bool>) {
    let (loading, set_loading) = signal(false);
    (loading, set_loading)
}

// Async operation wrapper with loading state
pub async fn with_loading<F, T>(set_loading: WriteSignal<bool>, operation: F) -> T
where
    F: std::future::Future<Output = T>,
{
    set_loading.set(true);
    let result = operation.await;
    set_loading.set(false);
    result
}
