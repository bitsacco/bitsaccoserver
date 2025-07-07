use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="bg-white border-t border-gray-200">
            <div class="max-w-7xl mx-auto py-4 px-4 sm:px-6 lg:px-8">
                <div class="flex justify-between items-center">
                    <div class="text-sm text-gray-500">
                        "© 2024 Bitsaccoserver. Built with Rust, Leptos, and ❤️"
                    </div>
                    <div class="flex space-x-6">
                        <a href="#" class="text-sm text-gray-500 hover:text-gray-900">
                            "Documentation"
                        </a>
                        <a href="#" class="text-sm text-gray-500 hover:text-gray-900">
                            "Support"
                        </a>
                        <a href="#" class="text-sm text-gray-500 hover:text-gray-900">
                            "Privacy"
                        </a>
                    </div>
                </div>
            </div>
        </footer>
    }
}
