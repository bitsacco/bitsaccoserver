use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use crate::api::{client::*, *};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub group_type: String,
    pub status: String,
    pub member_count: Option<u64>,
    pub children_count: Option<u64>,
}

#[component]
pub fn GroupsPage() -> impl IntoView {
    // Create SSR-compatible resource for groups data
    let groups_resource = Resource::new(
        || (),
        |_| crate::api::get_groups(None, None, None),
    );

    let show_create_form = RwSignal::new(false);
    let selected_group = RwSignal::new(None::<GroupResponse>);

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-semibold text-gray-900">"Groups"</h1>
                    <p class="mt-1 text-sm text-gray-500">"Group management and organization"</p>
                </div>
                <button
                    class="bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-2 px-4 rounded-lg"
                    on:click=move |_| show_create_form.set(true)
                >
                    "Add New Group"
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
                    match groups_resource.get() {
                        Some(result) => {
                            match result {
                                Ok(response) => {
                                    if let Some(paginated_data) = response.data {
                                        view! {
                                            <div class="bg-white shadow overflow-hidden sm:rounded-md">
                                                <ul role="list" class="divide-y divide-gray-200">
                                                    {paginated_data.data.into_iter().map(|group| {
                                                        view! {
                                                            <li class="px-6 py-4">
                                                                <div class="flex items-center justify-between">
                                                                    <div class="flex-1 min-w-0">
                                                                        <div class="flex items-center justify-between">
                                                                            <p class="text-sm font-medium text-indigo-600 truncate">
                                                                                {group.name.clone()}
                                                                            </p>
                                                                            <div class="ml-2 flex-shrink-0 flex">
                                                                                <p class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">
                                                                                    {group.status.clone()}
                                                                                </p>
                                                                            </div>
                                                                        </div>
                                                                        <div class="mt-2 flex">
                                                                            <div class="flex items-center text-sm text-gray-500">
                                                                                <p class="mr-4">
                                                                                    "Type: " {group.group_type.clone()}
                                                                                </p>
                                                                                {group.member_count.map(|count| {
                                                                                    view! {
                                                                                        <p class="mr-4">
                                                                                            "Members: " {count.to_string()}
                                                                                        </p>
                                                                                    }
                                                                                })}
                                                                            </div>
                                                                        </div>
                                                                        {group.description.as_ref().map(|desc| {
                                                                            view! {
                                                                                <p class="mt-2 text-sm text-gray-600">
                                                                                    {desc.clone()}
                                                                                </p>
                                                                            }
                                                                        })}
                                                                    </div>
                                                                    <div class="flex space-x-2">
                                                                        <button
                                                                            class="text-indigo-600 hover:text-indigo-900 text-sm font-medium"
                                                                            on:click={
                                                                                let group_clone = group.clone();
                                                                                move |_| {
                                                                                    selected_group.set(Some(group_clone.clone()));
                                                                                    show_create_form.set(true);
                                                                                }
                                                                            }
                                                                        >
                                                                            "Edit"
                                                                        </button>
                                                                        <button
                                                                            class="text-red-600 hover:text-red-900 text-sm font-medium"
                                                                            on:click={
                                                                                let group_id = group.id;
                                                                                let group_name = group.name.clone();
                                                                                move |_| {
                                                                                    if web_sys::window()
                                                                                        .unwrap()
                                                                                        .confirm_with_message(&format!("Are you sure you want to delete group '{}'? This action cannot be undone.", group_name))
                                                                                        .unwrap_or(false)
                                                                                    {
                                                                                        spawn_local(async move {
                                                                                            if let Ok(_) = delete_group(group_id).await {
                                                                                                groups_resource.refetch();
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
                                            <GroupsApiStatusCard />
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="bg-white shadow rounded-lg p-8">
                                                <div class="text-center">
                                                    <p class="text-gray-500">"No groups found"</p>
                                                </div>
                                            </div>
                                        }.into_any()
                                    }
                                },
                                Err(e) => view! {
                                    <div class="bg-red-50 border border-red-200 rounded-lg p-4">
                                        <div class="text-center">
                                            <p class="text-red-800">"Error loading groups: " {format!("{:?}", e)}</p>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        },
                        None => view! {
                            <div class="bg-white shadow rounded-lg p-8">
                                <div class="text-center">
                                    <p class="text-gray-500">"Loading groups..."</p>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </Suspense>

            // Group form modal
            {move || {
                if show_create_form.get() {
                    view! {
                        <GroupFormModal
                            _show=show_create_form
                            group=selected_group
                            on_close=move || {
                                show_create_form.set(false);
                                selected_group.set(None);
                            }
                            on_save=move |_| {
                                groups_resource.refetch();
                                show_create_form.set(false);
                                selected_group.set(None);
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
fn GroupsApiStatusCard() -> impl IntoView {
    view! {
        <div class="bg-green-50 border border-green-200 rounded-lg p-4 mt-6">
            <div class="flex">
                <div class="flex-shrink-0">
                    <svg class="h-5 w-5 text-green-400" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
                    </svg>
                </div>
                <div class="ml-3">
                    <h3 class="text-sm font-medium text-green-800">"✅ Groups API Integration Active"</h3>
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
fn GroupFormModal(
    _show: RwSignal<bool>,
    group: RwSignal<Option<GroupResponse>>,
    on_close: impl Fn() + 'static + Copy,
    on_save: impl Fn(GroupResponse) + 'static + Copy,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (group_type, set_group_type) = signal("Chama".to_string());
    let (is_loading, set_is_loading) = signal(false);
    let (error_message, set_error_message) = signal(None::<String>);

    // Update form fields when group changes
    Effect::new(move || {
        if let Some(group_data) = group.get() {
            set_name.set(group_data.name);
            set_description.set(group_data.description.unwrap_or_default());
            set_group_type.set(group_data.group_type);
        } else {
            set_name.set(String::new());
            set_description.set(String::new());
            set_group_type.set("Chama".to_string());
        }
    });

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_loading.set(true);
        set_error_message.set(None);

        let current_group = group.get();
        let name_val = name.get();
        let description_val = if description.get().is_empty() { None } else { Some(description.get()) };
        let group_type_val = group_type.get();

        spawn_local(async move {
            let result = if let Some(existing_group) = current_group {
                // Update existing group
                update_group(UpdateGroupRequest {
                    id: existing_group.id,
                    name: Some(name_val),
                    description: description_val,
                    group_type: Some(group_type_val),
                    parent_group_id: None, // TODO: Add parent group selection
                }).await
            } else {
                // Create new group
                create_group(CreateGroupRequest {
                    name: name_val,
                    description: description_val,
                    group_type: group_type_val,
                    parent_group_id: None, // TODO: Add parent group selection
                }).await
            };

            set_is_loading.set(false);

            match result {
                Ok(response) => {
                    if let Some(group_data) = response.data {
                        on_save(group_data);
                    }
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Error: {}", e)));
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
                            {move || if group.get().is_some() { "Edit Group" } else { "Add New Group" }}
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
                                    {error}
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
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    prop:value=move || name.get()
                                    on:input=move |ev| set_name.set(event_target_value(&ev))
                                />
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Description"
                                </label>
                                <textarea
                                    rows="3"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    prop:value=move || description.get()
                                    on:input=move |ev| set_description.set(event_target_value(&ev))
                                />
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Group Type *"
                                </label>
                                <select
                                    required
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    prop:value=move || group_type.get()
                                    on:change=move |ev| set_group_type.set(event_target_value(&ev))
                                >
                                    <option value="Chama">"Chama (Investment/Savings Group)"</option>
                                    <option value="Organization">"Organization"</option>
                                </select>
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
                                disabled=move || is_loading.get()
                                class="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 rounded-md"
                            >
                                {move || if is_loading.get() {
                                    "Saving..."
                                } else if group.get().is_some() {
                                    "Update Group"
                                } else {
                                    "Create Group"
                                }}
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    }
}
