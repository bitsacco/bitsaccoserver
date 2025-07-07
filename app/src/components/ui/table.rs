use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct TableColumn {
    pub key: String,
    pub title: String,
    pub sortable: bool,
    pub width: Option<String>,
    pub align: TextAlign,
}

#[derive(Clone, Debug)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl TextAlign {
    fn to_class(&self) -> &'static str {
        match self {
            TextAlign::Left => "text-left",
            TextAlign::Center => "text-center",
            TextAlign::Right => "text-right",
        }
    }
}

#[derive(Clone, Debug)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    fn to_icon(&self) -> &'static str {
        match self {
            SortDirection::Asc => "↑",
            SortDirection::Desc => "↓",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TableState {
    pub sort_column: Option<String>,
    pub sort_direction: Option<SortDirection>,
    pub current_page: u32,
    pub page_size: u32,
    pub search_query: String,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            sort_column: None,
            sort_direction: None,
            current_page: 1,
            page_size: 10,
            search_query: String::new(),
        }
    }
}

#[component]
pub fn DataTable<T>(
    #[prop(into)] columns: Signal<Vec<TableColumn>>,
    #[prop(into)] data: Signal<Vec<T>>,
    #[prop(optional)] loading: Option<Signal<bool>>,
    #[prop(optional)] searchable: bool,
    #[prop(optional)] sortable: bool,
    #[prop(optional)] paginated: bool,
    #[prop(optional)] page_size_options: Option<Vec<u32>>,
    #[prop(optional)] on_sort: Option<Callback<(String, SortDirection)>>,
    #[prop(optional)] on_page_change: Option<Callback<u32>>,
    #[prop(optional)] on_search: Option<Callback<String>>,
    #[prop(optional)] row_render: Option<Callback<(T, usize), Vec<AnyView>>>,
    #[prop(optional)] empty_message: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    let (table_state, set_table_state) = signal(TableState::default());
    let is_loading = loading.unwrap_or_else(|| signal(false).0.into());
    let page_sizes = page_size_options.unwrap_or_else(|| vec![10, 25, 50, 100]);

    let handle_sort = move |column: String| {
        if !sortable {
            return;
        }

        set_table_state.update(|state| {
            if state.sort_column.as_ref() == Some(&column) {
                state.sort_direction = match state.sort_direction {
                    Some(SortDirection::Asc) => Some(SortDirection::Desc),
                    Some(SortDirection::Desc) => None,
                    None => Some(SortDirection::Asc),
                };
                if state.sort_direction.is_none() {
                    state.sort_column = None;
                }
            } else {
                state.sort_column = Some(column.clone());
                state.sort_direction = Some(SortDirection::Asc);
            }
        });

        if let (Some(callback), Some(direction)) = (on_sort, table_state.get().sort_direction) {
            callback.run((column, direction));
        }
    };

    let handle_search = move |query: String| {
        set_table_state.update(|state| {
            state.search_query = query.clone();
            state.current_page = 1; // Reset to first page on search
        });

        if let Some(callback) = on_search {
            callback.run(query);
        }
    };

    let handle_page_change = move |page: u32| {
        set_table_state.update(|state| {
            state.current_page = page;
        });

        if let Some(callback) = on_page_change {
            callback.run(page);
        }
    };

    let total_pages = Signal::derive(move || {
        let data_len = data.get().len() as u32;
        let page_size = table_state.get().page_size;
        (data_len + page_size - 1) / page_size
    });

    view! {
        <div class=format!("bg-white shadow rounded-lg {}", class.unwrap_or(""))>
            // Search bar
            <Show when=move || searchable>
                <div class="p-4 border-b border-gray-200">
                    <div class="max-w-sm">
                        <input
                            type="text"
                            placeholder="Search..."
                            class="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
                            value=move || table_state.get().search_query
                            on:input=move |ev| {
                                handle_search(event_target_value(&ev));
                            }
                        />
                    </div>
                </div>
            </Show>

            // Table
            <div class="overflow-hidden">
                <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200">
                        // Header
                        <thead class="bg-gray-50">
                            <tr>
                                <For
                                    each=move || columns.get()
                                    key=|column| column.key.clone()
                                    children=move |column| {
                                        let column_key = column.key.clone();
                                        let is_sorted = Signal::derive({
                                            let column_key = column_key.clone();
                                            move || table_state.get().sort_column.as_ref() == Some(&column_key)
                                        });

                                        view! {
                                            <th
                                                class=format!(
                                                    "px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider {}",
                                                    column.align.to_class()
                                                )
                                                style=column.width.map(|w| format!("width: {}", w)).unwrap_or_default()
                                            >
                                                {if column.sortable && sortable {
                                                    view! {
                                                    <button
                                                        class="group inline-flex items-center hover:text-gray-900"
                                                        on:click={
                                                            let column_key = column_key.clone();
                                                            move |_| handle_sort(column_key.clone())
                                                        }
                                                    >
                                                        <span>{column.title.clone()}</span>
                                                        {if is_sorted.get() {
                                                            view! {
                                                                <span class="ml-1">
                                                                    {move || {
                                                                        table_state.get().sort_direction
                                                                            .map(|d| d.to_icon())
                                                                            .unwrap_or("")
                                                                    }}
                                                                </span>
                                                            }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                    </button>
                                                    }.into_any()
                                                } else {
                                                    view! { <span>{column.title.clone()}</span> }.into_any()
                                                }}
                                            </th>
                                        }
                                    }
                                />
                            </tr>
                        </thead>

                        // Body
                        <tbody class="bg-white divide-y divide-gray-200">
                            {if !is_loading.get() {
                                view! {
                                {if !data.get().is_empty() {
                                    view! {
                                    {move || {
                                        data.get().into_iter().enumerate().map(|(index, item)| {
                                            view! {
                                                <tr class="hover:bg-gray-50">
                                                    {if let Some(renderer) = row_render {
                                                        renderer.run((item, index))
                                                    } else {
                                                        // Default row rendering would go here
                                                        // For now, just show placeholders
                                                        columns.get().into_iter().map(|_| {
                                                            view! {
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                    "Data"
                                                                </td>
                                                            }.into_any()
                                                        }).collect()
                                                    }}
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()
                                    }}
                                    }.into_any()
                                } else {
                                    view! {
                                        <tr>
                                            <td colspan=move || columns.get().len() class="px-6 py-4 text-center text-gray-500">
                                                {empty_message.unwrap_or("No data available")}
                                            </td>
                                        </tr>
                                    }.into_any()
                                }}
                                }.into_any()
                            } else {
                                view! {
                                    <tr>
                                        <td colspan=move || columns.get().len() class="px-6 py-4 text-center text-gray-500">
                                            <div class="flex items-center justify-center">
                                                <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
                                                <span class="ml-2">"Loading..."</span>
                                            </div>
                                        </td>
                                    </tr>
                                }.into_any()
                            }}
                        </tbody>
                    </table>
                </div>
            </div>

            // Pagination
            {if paginated {
                view! {
                <div class="bg-white px-4 py-3 flex items-center justify-between border-t border-gray-200 sm:px-6">
                    <div class="flex-1 flex justify-between sm:hidden">
                        <button
                            class="relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
                            disabled=move || table_state.get().current_page <= 1
                            on:click=move |_| {
                                let current = table_state.get().current_page;
                                if current > 1 {
                                    handle_page_change(current - 1);
                                }
                            }
                        >
                            "Previous"
                        </button>
                        <button
                            class="ml-3 relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
                            disabled=move || table_state.get().current_page >= total_pages.get()
                            on:click=move |_| {
                                let current = table_state.get().current_page;
                                let total = total_pages.get();
                                if current < total {
                                    handle_page_change(current + 1);
                                }
                            }
                        >
                            "Next"
                        </button>
                    </div>

                    <div class="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between">
                        <div>
                            <p class="text-sm text-gray-700">
                                "Showing page "
                                <span class="font-medium">{move || table_state.get().current_page}</span>
                                " of "
                                <span class="font-medium">{total_pages}</span>
                            </p>
                        </div>

                        <div class="flex items-center space-x-2">
                            <select
                                class="block w-20 rounded-md border-gray-300 text-sm"
                                prop:value=move || table_state.get().page_size.to_string()
                                on:change=move |ev| {
                                    if let Ok(size) = event_target_value(&ev).parse::<u32>() {
                                        set_table_state.update(|state| {
                                            state.page_size = size;
                                            state.current_page = 1;
                                        });
                                    }
                                }
                            >
                                {page_sizes.into_iter().map(|size| {
                                    view! {
                                        <option value=size.to_string()>{size}</option>
                                    }
                                }).collect::<Vec<_>>()}
                            </select>

                            <nav class="relative z-0 inline-flex rounded-md shadow-sm -space-x-px">
                                <button
                                    class="relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50"
                                    disabled=move || table_state.get().current_page <= 1
                                    on:click=move |_| {
                                        let current = table_state.get().current_page;
                                        if current > 1 {
                                            handle_page_change(current - 1);
                                        }
                                    }
                                >
                                    "‹"
                                </button>

                                // Page numbers would go here in a full implementation

                                <button
                                    class="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50"
                                    disabled=move || table_state.get().current_page >= total_pages.get()
                                    on:click=move |_| {
                                        let current = table_state.get().current_page;
                                        let total = total_pages.get();
                                        if current < total {
                                            handle_page_change(current + 1);
                                        }
                                    }
                                >
                                    "›"
                                </button>
                            </nav>
                        </div>
                    </div>
                </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}
