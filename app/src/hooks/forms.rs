use leptos::prelude::*;

// Basic form state management
pub fn use_form_state<T>(initial_data: T) -> (Signal<T>, WriteSignal<T>)
where
    T: Clone + Send + Sync + 'static,
{
    let (data, set_data) = signal(initial_data);
    (data.into(), set_data)
}

// Simple validation hook
pub fn use_field_error(_field_name: &str) -> (Signal<Option<String>>, WriteSignal<Option<String>>) {
    let (error, set_error) = signal(None::<String>);
    (error.into(), set_error)
}

// Form submission state
pub struct FormSubmissionState {
    pub is_submitting: Signal<bool>,
    pub error: Signal<Option<String>>,
    pub success: Signal<bool>,
}

pub fn use_form_submission() -> (
    FormSubmissionState,
    WriteSignal<bool>,
    WriteSignal<Option<String>>,
    WriteSignal<bool>,
) {
    let (is_submitting, set_is_submitting) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(false);

    let state = FormSubmissionState {
        is_submitting: is_submitting.into(),
        error: error.into(),
        success: success.into(),
    };

    (state, set_is_submitting, set_error, set_success)
}

// Basic validation rules
pub fn validate_required(value: &str, field_name: &str) -> Option<String> {
    if value.trim().is_empty() {
        Some(format!("{} is required", field_name))
    } else {
        None
    }
}

pub fn validate_email(email: &str) -> Option<String> {
    if email.contains('@') && email.contains('.') {
        None
    } else {
        Some("Please enter a valid email address".to_string())
    }
}

pub fn validate_min_length(value: &str, min: usize, field_name: &str) -> Option<String> {
    if value.len() < min {
        Some(format!(
            "{} must be at least {} characters",
            field_name, min
        ))
    } else {
        None
    }
}
