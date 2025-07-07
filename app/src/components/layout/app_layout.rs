use super::{Header, Sidebar};
use leptos::prelude::*;

#[component]
pub fn AppLayout(children: Children) -> impl IntoView {
    let (mobile_open, set_mobile_open) = signal(false);

    view! {
        <div class="min-h-screen bg-gray-50">
            <Sidebar mobile_open=mobile_open.into() set_mobile_open=set_mobile_open/>
            <div class="lg:pl-64">
                <Header set_mobile_open=set_mobile_open/>
                <main class="py-8">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                        {children()}
                    </div>
                </main>
            </div>
        </div>
    }
}
