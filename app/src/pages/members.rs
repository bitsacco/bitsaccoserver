use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use crate::api::{client::*, *};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberResponse {
    pub id: uuid::Uuid,
    pub member_number: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: String,
    pub groups: Option<Vec<GroupInfo>>,
    pub shares_count: Option<u64>,
    pub total_shares_value: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub id: uuid::Uuid,
    pub name: String,
}

#[component]
pub fn MembersPage() -> impl IntoView {
    // Create SSR-compatible resource for members data
    let members_resource = Resource::new(
        || (),
        |_| crate::api::get_members(None, None, None),
    );

    let show_create_form = RwSignal::new(false);
    let selected_member = RwSignal::new(None::<MemberResponse>);

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-semibold text-gray-900">"Members"</h1>
                    <p class="mt-1 text-sm text-gray-500">"Member management and registration"</p>
                </div>
                <button
                    class="bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-2 px-4 rounded-lg"
                    on:click=move |_| show_create_form.set(true)
                >
                    "Add New Member"
                </button>
            </div>

            <Suspense fallback=|| view! {
                <div class="bg-white shadow rounded-lg p-8">
                    <div class="text-center">
                        <div class="animate-pulse">
                            <div class="h-12 w-12 bg-gray-200 rounded-full mx-auto mb-4"></div>
                            <div class="h-4 bg-gray-200 rounded w-1/3 mx-auto mb-2"></div>
                            <div class="h-3 bg-gray-200 rounded w-1/2 mx-auto"></div>
                        </div>
                    </div>
                </div>
            }>
                {move || {
                    match members_resource.get() {
                        Some(result) => {
                            match result {
                                Ok(response) => {
                                    if let Some(paginated_data) = response.data {
                                        view! {
                                            <div class="bg-white shadow overflow-hidden sm:rounded-md">
                                                <ul role="list" class="divide-y divide-gray-200">
                                                    {paginated_data.data.into_iter().map(|member| {
                                                        view! {
                                                            <li class="px-6 py-4">
                                                                <div class="flex items-center justify-between">
                                                                    <div class="flex-1 min-w-0">
                                                                        <div class="flex items-center justify-between">
                                                                            <div>
                                                                                <p class="text-sm font-medium text-indigo-600 truncate">
                                                                                    {member.name.clone()}
                                                                                </p>
                                                                                <p class="text-sm text-gray-500">
                                                                                    "Member #" {member.member_number.clone()}
                                                                                </p>
                                                                            </div>
                                                                            <div class="ml-2 flex-shrink-0 flex">
                                                                                <p class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">
                                                                                    {member.status.clone()}
                                                                                </p>
                                                                            </div>
                                                                        </div>
                                                                        <div class="mt-2 flex flex-wrap">
                                                                            {member.email.as_ref().map(|email| {
                                                                                view! {
                                                                                    <p class="text-sm text-gray-500 mr-4">
                                                                                        "📧 " {email.clone()}
                                                                                    </p>
                                                                                }
                                                                            })}
                                                                            {member.phone.as_ref().map(|phone| {
                                                                                view! {
                                                                                    <p class="text-sm text-gray-500 mr-4">
                                                                                        "📞 " {phone.clone()}
                                                                                    </p>
                                                                                }
                                                                            })}
                                                                        </div>
                                                                        <div class="mt-2 flex flex-wrap">
                                                                            {member.shares_count.map(|count| {
                                                                                view! {
                                                                                    <p class="text-sm text-gray-500 mr-4">
                                                                                        "Shares: " {count.to_string()}
                                                                                    </p>
                                                                                }
                                                                            })}
                                                                            {member.total_shares_value.as_ref().map(|value| {
                                                                                use rust_decimal::prelude::ToPrimitive;
                                                                                view! {
                                                                                    <p class="text-sm text-gray-500 mr-4">
                                                                                        "Value: $" {format!("{:.2}", value.to_f64().unwrap_or(0.0))}
                                                                                    </p>
                                                                                }
                                                                            })}
                                                                        </div>
                                                                        {member.groups.as_ref().map(|groups| {
                                                                            if !groups.is_empty() {
                                                                                view! {
                                                                                    <div class="mt-2">
                                                                                        <p class="text-sm text-gray-500">
                                                                                            "Groups: " {groups.iter().map(|g| g.name.clone()).collect::<Vec<_>>().join(", ")}
                                                                                        </p>
                                                                                    </div>
                                                                                }.into_any()
                                                                            } else {
                                                                                view! { <div></div> }.into_any()
                                                                            }
                                                                        })}
                                                                    </div>
                                                                    <div class="flex space-x-2">
                                                                        <button
                                                                            class="text-indigo-600 hover:text-indigo-900 text-sm font-medium"
                                                                            on:click={
                                                                                let member_clone = member.clone();
                                                                                move |_| {
                                                                                    selected_member.set(Some(member_clone.clone()));
                                                                                    show_create_form.set(true);
                                                                                }
                                                                            }
                                                                        >
                                                                            "Edit"
                                                                        </button>
                                                                        <button
                                                                            class="text-red-600 hover:text-red-900 text-sm font-medium"
                                                                            on:click={
                                                                                let member_id = member.id;
                                                                                let member_name = member.name.clone();
                                                                                move |_| {
                                                                                    if web_sys::window()
                                                                                        .unwrap()
                                                                                        .confirm_with_message(&format!("Are you sure you want to delete member '{}'? This action cannot be undone.", member_name))
                                                                                        .unwrap_or(false)
                                                                                    {
                                                                                        spawn_local(async move {
                                                                                            if let Ok(_) = delete_member(member_id).await {
                                                                                                members_resource.refetch();
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                }
                                                                            }
                                                                        >
                                                                            "Delete"
                                                                        </button>
                                                                    </div>
                                                                </div>
                                                            </li>
                                                        }
                                                    }).collect_view()}
                                                </ul>
                                            </div>
                                            <MembersApiStatusCard />
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="bg-white shadow rounded-lg p-8">
                                                <div class="text-center">
                                                    <p class="text-gray-500">"No members found"</p>
                                                </div>
                                            </div>
                                        }.into_any()
                                    }
                                },
                                Err(e) => view! {
                                    <div class="bg-red-50 border border-red-200 rounded-lg p-4">
                                        <div class="text-center">
                                            <p class="text-red-800">"Error loading members: " {format!("{:?}", e)}</p>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        },
                        None => view! {
                            <div class="bg-white shadow rounded-lg p-8">
                                <div class="text-center">
                                    <p class="text-gray-500">"Loading members..."</p>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </Suspense>

            // Member form modal
            {move || {
                if show_create_form.get() {
                    view! {
                        <MemberFormModal
                            _show=show_create_form
                            member=selected_member
                            on_close=move || {
                                show_create_form.set(false);
                                selected_member.set(None);
                            }
                            on_save=move |_| {
                                members_resource.refetch();
                                show_create_form.set(false);
                                selected_member.set(None);
                            }
                        />
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn MembersApiStatusCard() -> impl IntoView {
    view! {
        <div class="bg-green-50 border border-green-200 rounded-lg p-4 mt-6">
            <div class="flex">
                <div class="flex-shrink-0">
                    <svg class="h-5 w-5 text-green-400" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
                    </svg>
                </div>
                <div class="ml-3">
                    <h3 class="text-sm font-medium text-green-800">"✅ Members API Integration Active"</h3>
                    <div class="mt-2 text-sm text-green-700">
                        <ul class="list-disc list-inside space-y-1">
                            <li>"SSR-compatible server functions ✓"</li>
                            <li>"Backend API endpoints available ✓"</li>
                            <li>"Real-time data loading with error handling ✓"</li>
                            <li>"Database integration ready ✓"</li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn MemberFormModal(
    _show: RwSignal<bool>,
    member: RwSignal<Option<MemberResponse>>,
    on_close: impl Fn() + 'static + Copy,
    on_save: impl Fn(MemberResponse) + 'static + Copy,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (phone, set_phone) = signal(String::new());
    let (member_number, set_member_number) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (error_message, set_error_message) = signal(None::<String>);
    let (success_message, set_success_message) = signal(None::<String>);
    
    // Validation states
    let (name_error, set_name_error) = signal(None::<String>);
    let (email_error, set_email_error) = signal(None::<String>);
    let (phone_error, set_phone_error) = signal(None::<String>);
    
    // Validation functions
    let validate_name = move |value: &str| -> Option<String> {
        if value.trim().is_empty() {
            Some("Name is required".to_string())
        } else if value.trim().len() < 2 {
            Some("Name must be at least 2 characters".to_string())
        } else {
            None
        }
    };
    
    let validate_email = move |value: &str| -> Option<String> {
        if !value.is_empty() && !value.contains('@') {
            Some("Please enter a valid email address".to_string())
        } else {
            None
        }
    };
    
    let validate_phone = move |value: &str| -> Option<String> {
        if !value.is_empty() && value.len() < 10 {
            Some("Phone number must be at least 10 digits".to_string())
        } else {
            None
        }
    };

    // Update form fields when member changes
    Effect::new(move || {
        if let Some(member_data) = member.get() {
            set_name.set(member_data.name);
            set_email.set(member_data.email.unwrap_or_default());
            set_phone.set(member_data.phone.unwrap_or_default());
            set_member_number.set(member_data.member_number);
        } else {
            set_name.set(String::new());
            set_email.set(String::new());
            set_phone.set(String::new());
            set_member_number.set(String::new());
        }
    });

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        // Clear previous messages
        set_error_message.set(None);
        set_success_message.set(None);
        
        // Validate all fields
        let name_val = name.get();
        let email_val = email.get();
        let phone_val = phone.get();
        
        let name_validation = validate_name(&name_val);
        let email_validation = validate_email(&email_val);
        let phone_validation = validate_phone(&phone_val);
        
        set_name_error.set(name_validation.clone());
        set_email_error.set(email_validation.clone());
        set_phone_error.set(phone_validation.clone());
        
        // Check if there are any validation errors
        if name_validation.is_some() || email_validation.is_some() || phone_validation.is_some() {
            set_error_message.set(Some("Please fix the validation errors before submitting".to_string()));
            return;
        }

        set_is_loading.set(true);

        let current_member = member.get();
        let is_updating = current_member.is_some();
        let email_val = if email_val.is_empty() { None } else { Some(email_val) };
        let phone_val = if phone_val.is_empty() { None } else { Some(phone_val) };
        let member_number_val = if member_number.get().is_empty() { None } else { Some(member_number.get()) };

        spawn_local(async move {
            let result = if let Some(existing_member) = current_member {
                // Update existing member
                update_member(UpdateMemberRequest {
                    id: existing_member.id,
                    name: Some(name_val),
                    email: email_val,
                    phone: phone_val,
                    member_number: member_number_val,
                }).await
            } else {
                // Create new member
                create_member(CreateMemberRequest {
                    name: name_val,
                    email: email_val,
                    phone: phone_val,
                    member_number: member_number_val,
                }).await
            };

            set_is_loading.set(false);

            match result {
                Ok(response) => {
                    if let Some(member_data) = response.data {
                        set_success_message.set(Some(response.message.unwrap_or_else(|| 
                            if is_updating { "Member updated successfully".to_string() } 
                            else { "Member created successfully".to_string() }
                        )));
                        
                        on_save(member_data);
                    }
                }
                Err(e) => {
                    let error_msg = if e.to_string().contains("Database error") {
                        "A database error occurred. Please try again.".to_string()
                    } else if e.to_string().contains("Config error") {
                        "A configuration error occurred. Please contact support.".to_string()
                    } else if e.to_string().contains("email") {
                        "Invalid email address. Please check and try again.".to_string()
                    } else {
                        format!("An error occurred: {}", e)
                    };
                    set_error_message.set(Some(error_msg));
                }
            }
        });
    };

    view! {
        <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
            <div class="relative top-20 mx-auto p-5 border w-11/12 md:w-1/2 lg:w-1/3 shadow-lg rounded-md bg-white">
                <div class="mt-3">
                    <div class="flex justify-between items-center mb-4">
                        <h3 class="text-lg font-medium text-gray-900">
                            {move || if member.get().is_some() { "Edit Member" } else { "Add New Member" }}
                        </h3>
                        <button
                            class="text-gray-400 hover:text-gray-600"
                            on:click=move |_| on_close()
                        >
                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                            </svg>
                        </button>
                    </div>

                    {move || {
                        if let Some(error) = error_message.get() {
                            view! {
                                <div class="mb-4 bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded">
                                    <div class="flex">
                                        <svg class="h-5 w-5 text-red-400 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"></path>
                                        </svg>
                                        {error}
                                    </div>
                                </div>
                            }.into_any()
                        } else if let Some(success) = success_message.get() {
                            view! {
                                <div class="mb-4 bg-green-50 border border-green-200 text-green-600 px-4 py-3 rounded">
                                    <div class="flex">
                                        <svg class="h-5 w-5 text-green-400 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
                                        </svg>
                                        {success}
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}

                    <form on:submit=handle_submit>
                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Name *"
                                </label>
                                <input
                                    type="text"
                                    required
                                    class=move || if name_error.get().is_some() {
                                        "w-full px-3 py-2 border border-red-300 rounded-md focus:outline-none focus:ring-2 focus:ring-red-500"
                                    } else {
                                        "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    }
                                    prop:value=move || name.get()
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        set_name.set(value.clone());
                                        set_name_error.set(validate_name(&value));
                                    }
                                />
                                {move || {
                                    if let Some(error) = name_error.get() {
                                        view! {
                                            <p class="mt-1 text-sm text-red-600">{error}</p>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }}
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Email"
                                </label>
                                <input
                                    type="email"
                                    class=move || if email_error.get().is_some() {
                                        "w-full px-3 py-2 border border-red-300 rounded-md focus:outline-none focus:ring-2 focus:ring-red-500"
                                    } else {
                                        "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    }
                                    prop:value=move || email.get()
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        set_email.set(value.clone());
                                        set_email_error.set(validate_email(&value));
                                    }
                                />
                                {move || {
                                    if let Some(error) = email_error.get() {
                                        view! {
                                            <p class="mt-1 text-sm text-red-600">{error}</p>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }}
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Phone"
                                </label>
                                <input
                                    type="tel"
                                    class=move || if phone_error.get().is_some() {
                                        "w-full px-3 py-2 border border-red-300 rounded-md focus:outline-none focus:ring-2 focus:ring-red-500"
                                    } else {
                                        "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    }
                                    prop:value=move || phone.get()
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        set_phone.set(value.clone());
                                        set_phone_error.set(validate_phone(&value));
                                    }
                                />
                                {move || {
                                    if let Some(error) = phone_error.get() {
                                        view! {
                                            <p class="mt-1 text-sm text-red-600">{error}</p>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }}
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Member Number"
                                    <span class="text-gray-500 text-xs">" (auto-generated if empty)"</span>
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    prop:value=move || member_number.get()
                                    on:input=move |ev| set_member_number.set(event_target_value(&ev))
                                />
                            </div>
                        </div>

                        <div class="flex justify-end space-x-3 mt-6">
                            <button
                                type="button"
                                class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-200 hover:bg-gray-300 rounded-md"
                                on:click=move |_| on_close()
                            >
                                "Cancel"
                            </button>
                            <button
                                type="submit"
                                disabled=move || is_loading.get() || name_error.get().is_some() || email_error.get().is_some() || phone_error.get().is_some() || name.get().trim().is_empty()
                                class="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed rounded-md"
                            >
                                {move || if is_loading.get() {
                                    "Saving..."
                                } else if member.get().is_some() {
                                    "Update Member"
                                } else {
                                    "Create Member"
                                }}
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    }
}
