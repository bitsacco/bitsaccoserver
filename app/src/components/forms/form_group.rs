use leptos::prelude::*;

#[component]
pub fn FormGroup(
    #[prop(optional)] title: Option<String>,
    #[prop(optional)] description: Option<String>,
    #[prop(optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("space-y-4 {}", class.unwrap_or_default())>
            {if let Some(title) = title {
                view! {
                    <div class="border-b border-gray-200 pb-2">
                        <h3 class="text-lg leading-6 font-medium text-gray-900">
                            {title}
                        </h3>
                        {if let Some(desc) = description {
                            view! {
                                <p class="mt-1 text-sm text-gray-500">
                                    {desc}
                                </p>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            <div class="space-y-4">
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn FormActions(#[prop(optional)] class: Option<String>, children: Children) -> impl IntoView {
    view! {
        <div class=format!(
            "flex items-center justify-end space-x-3 pt-6 border-t border-gray-200 {}",
            class.unwrap_or_default()
        )>
            {children()}
        </div>
    }
}

#[component]
pub fn FormContainer(
    #[prop(optional)] title: Option<String>,
    #[prop(optional)] description: Option<String>,
    #[prop(optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("bg-white shadow rounded-lg {}", class.unwrap_or_default())>
            {if let Some(title) = title {
                view! {
                    <div class="px-6 py-4 border-b border-gray-200">
                        <h2 class="text-xl font-semibold text-gray-900">
                            {title}
                        </h2>
                        {if let Some(desc) = description {
                            view! {
                                <p class="mt-1 text-sm text-gray-500">
                                    {desc}
                                </p>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            <div class="px-6 py-4">
                {children()}
            </div>
        </div>
    }
}
