use leptos::prelude::*;

#[component]
pub fn Settings() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold text-gray-900">Settings</h1>
                <p class="mt-1 text-sm text-gray-600">
                    "Manage your account settings and preferences."
                </p>
            </div>

            <div class="bg-white shadow rounded-lg p-6">
                <h3 class="text-lg font-medium text-gray-900 mb-4">Account Information</h3>
                <p class="text-sm text-gray-600 mb-4">
                    "Update your account details and profile information."
                </p>

                <div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
                    <SimpleFormField label="First Name" placeholder="Enter first name"/>
                    <SimpleFormField label="Last Name" placeholder="Enter last name"/>
                    <SimpleFormField label="Email Address" placeholder="Enter email" input_type="email"/>
                    <SimpleFormField label="Phone Number" placeholder="Enter phone" input_type="tel"/>
                </div>

                <div class="mt-6 flex justify-end">
                    <button class="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500">
                        "Save Changes"
                    </button>
                </div>
            </div>

            <div class="bg-white shadow rounded-lg p-6">
                <h3 class="text-lg font-medium text-gray-900 mb-4">Preferences</h3>
                <p class="text-sm text-gray-600 mb-4">
                    "Customize your experience and display settings."
                </p>

                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">Theme</label>
                        <div class="space-y-2">
                            <label class="flex items-center">
                                <input type="radio" name="theme" value="light" checked class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300"/>
                                <span class="ml-2 text-sm text-gray-700">Light</span>
                            </label>
                            <label class="flex items-center">
                                <input type="radio" name="theme" value="dark" class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300"/>
                                <span class="ml-2 text-sm text-gray-700">Dark</span>
                            </label>
                        </div>
                    </div>

                    <SimpleSelectField label="Language" options=vec![("en", "English"), ("sw", "Swahili")]/>
                    <SimpleSelectField label="Timezone" options=vec![("Africa/Nairobi", "Africa/Nairobi"), ("UTC", "UTC")]/>
                </div>

                <div class="mt-6 flex justify-end">
                    <button class="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500">
                        "Save Preferences"
                    </button>
                </div>
            </div>

            <div class="bg-white shadow rounded-lg p-6">
                <h3 class="text-lg font-medium text-gray-900 mb-4">Notifications</h3>
                <p class="text-sm text-gray-600 mb-4">
                    "Configure how you receive notifications."
                </p>

                <div class="space-y-4">
                    <SimpleCheckboxField label="Email Notifications" checked=true/>
                    <SimpleCheckboxField label="SMS Notifications" checked=false/>
                    <SimpleCheckboxField label="Push Notifications" checked=true/>
                </div>

                <div class="mt-6 flex justify-end">
                    <button class="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500">
                        "Save Notification Settings"
                    </button>
                </div>
            </div>

            <div class="bg-white shadow rounded-lg p-6">
                <h3 class="text-lg font-medium text-gray-900 mb-4">Security</h3>
                <p class="text-sm text-gray-600 mb-4">
                    "Manage your password and security preferences."
                </p>

                <div class="space-y-4">
                    <SimpleFormField label="Current Password" placeholder="Enter current password" input_type="password"/>
                    <SimpleFormField label="New Password" placeholder="Enter new password" input_type="password"/>
                    <SimpleFormField label="Confirm Password" placeholder="Confirm new password" input_type="password"/>
                </div>

                <div class="mt-6 flex justify-end">
                    <button class="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500">
                        "Update Password"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn SimpleFormField(
    label: &'static str,
    placeholder: &'static str,
    #[prop(optional)] input_type: &'static str,
) -> impl IntoView {
    let input_type = if input_type.is_empty() {
        "text"
    } else {
        input_type
    };

    view! {
        <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
                {label}
            </label>
            <input
                type=input_type
                placeholder=placeholder
                class="block w-full rounded-md border-gray-300 shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
            />
        </div>
    }
}

#[component]
fn SimpleSelectField(
    label: &'static str,
    options: Vec<(&'static str, &'static str)>,
) -> impl IntoView {
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
                {label}
            </label>
            <select class="block w-full rounded-md border-gray-300 shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm">
                {options.into_iter().map(|(value, label)| {
                    view! {
                        <option value=value>{label}</option>
                    }
                }).collect::<Vec<_>>()}
            </select>
        </div>
    }
}

#[component]
fn SimpleCheckboxField(label: &'static str, #[prop(optional)] checked: bool) -> impl IntoView {
    view! {
        <div class="flex items-center">
            <input
                type="checkbox"
                prop:checked=checked
                class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
            />
            <label class="ml-2 text-sm text-gray-700">
                {label}
            </label>
        </div>
    }
}
