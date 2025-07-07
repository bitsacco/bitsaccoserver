use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn to_class(&self) -> &'static str {
        match self {
            Theme::Light => "",
            Theme::Dark => "dark",
        }
    }

    pub fn toggle(&self) -> Theme {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ThemeContext {
    pub theme: Signal<Theme>,
    pub set_theme: WriteSignal<Theme>,
}

#[component]
pub fn ThemeProvider(children: Children) -> impl IntoView {
    // Initialize theme from localStorage or default to Light
    let (theme, set_theme) = signal(Theme::Light);

    let theme_context = ThemeContext {
        theme: theme.into(),
        set_theme,
    };
    provide_context(theme_context);

    view! {
        <div class=move || format!("min-h-screen {}", theme.get().to_class())>
            {children()}
        </div>
    }
}

pub fn use_theme() -> ThemeContext {
    use_context::<ThemeContext>().expect("ThemeContext not provided")
}

#[component]
pub fn ThemeToggle() -> impl IntoView {
    let theme_ctx = use_theme();

    let toggle_theme = move |_| {
        let current = theme_ctx.theme.get();
        let new_theme = current.toggle();
        theme_ctx.set_theme.set(new_theme);
    };

    view! {
        <button
            type="button"
            class="bg-white p-1 rounded-full text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 dark:bg-gray-800 dark:text-gray-300 dark:hover:text-gray-200"
            title=move || {
                match theme_ctx.theme.get() {
                    Theme::Light => "Switch to dark mode",
                    Theme::Dark => "Switch to light mode",
                }
            }
            on:click=toggle_theme
        >
            <span class="sr-only">"Toggle theme"</span>
            {move || {
                match theme_ctx.theme.get() {
                    Theme::Light => view! {
                        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
                        </svg>
                    }.into_any(),
                    Theme::Dark => view! {
                        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
                        </svg>
                    }.into_any(),
                }
            }}
        </button>
    }
}
