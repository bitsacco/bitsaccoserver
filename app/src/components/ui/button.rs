use leptos::prelude::*;

#[derive(Clone, Copy)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Success,
}

impl ButtonVariant {
    fn to_class(self) -> &'static str {
        match self {
            ButtonVariant::Primary => "bg-blue-600 hover:bg-blue-700 text-white",
            ButtonVariant::Secondary => "bg-gray-200 hover:bg-gray-300 text-gray-900",
            ButtonVariant::Danger => "bg-red-600 hover:bg-red-700 text-white",
            ButtonVariant::Success => "bg-green-600 hover:bg-green-700 text-white",
        }
    }
}

#[derive(Clone, Copy)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl ButtonSize {
    fn to_class(self) -> &'static str {
        match self {
            ButtonSize::Small => "px-3 py-1.5 text-sm",
            ButtonSize::Medium => "px-4 py-2 text-sm",
            ButtonSize::Large => "px-6 py-3 text-base",
        }
    }
}

#[component]
pub fn Button(
    #[prop(optional)] variant: Option<ButtonVariant>,
    #[prop(optional)] size: Option<ButtonSize>,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] onclick: Option<Callback<()>>,
    #[prop(optional)] type_: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(ButtonVariant::Primary);
    let size = size.unwrap_or(ButtonSize::Medium);
    let type_ = type_.unwrap_or("button");

    let button_class = format!(
        "inline-flex items-center justify-center rounded-md border border-transparent font-medium focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed {} {} {}",
        variant.to_class(),
        size.to_class(),
        class.unwrap_or("")
    );

    view! {
        <button
            type=type_
            class=button_class
            disabled=disabled
            on:click=move |_| {
                if let Some(cb) = onclick {
                    cb.run(())
                }
            }
        >
            {children()}
        </button>
    }
}
