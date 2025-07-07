use leptos::prelude::*;

// Simple API data hook (placeholder - will be implemented when we need it)
pub fn use_api_data<T>(_url: String) -> (Signal<Option<T>>, Signal<bool>, Signal<Option<String>>)
where
    T: Clone + Send + Sync + 'static,
{
    let (data, _set_data) = signal(None::<T>);
    let (loading, _set_loading) = signal(false);
    let (error, _set_error) = signal(None::<String>);

    (data.into(), loading.into(), error.into())
}

// Simple pagination state
pub struct PaginationState {
    pub page: Signal<u32>,
    pub set_page: WriteSignal<u32>,
    pub per_page: Signal<u32>,
    pub set_per_page: WriteSignal<u32>,
}

pub fn use_pagination(initial_page: u32, initial_per_page: u32) -> PaginationState {
    let (page, set_page) = signal(initial_page);
    let (per_page, set_per_page) = signal(initial_per_page);

    PaginationState {
        page: page.into(),
        set_page,
        per_page: per_page.into(),
        set_per_page,
    }
}

// Search state
pub fn use_search_state() -> (Signal<String>, WriteSignal<String>) {
    let (search, set_search) = signal(String::new());
    (search.into(), set_search)
}
