use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use regex::Regex;
use std::collections::HashMap;

// Basic form state management
pub fn use_form_state<T>(initial_data: T) -> (Signal<T>, WriteSignal<T>)
where
    T: Clone + Send + Sync + 'static,
{
    let (data, set_data) = signal(initial_data);
    (data.into(), set_data)
}

// Simple validation hook (deprecated - use use_form_field instead)
#[deprecated(note = "Use use_form_field instead for better validation")]
pub fn use_field_error(_field_name: &str) -> (Signal<Option<String>>, WriteSignal<Option<String>>) {
    let (error, set_error) = signal(None::<String>);
    (error.into(), set_error)
}

// Password confirmation validation
pub fn validate_password_confirmation(password: &str, confirmation: &str) -> Option<String> {
    if confirmation.is_empty() {
        return Some("Password confirmation is required".to_string());
    }

    if password != confirmation {
        Some("Passwords do not match".to_string())
    } else {
        None
    }
}

// Form field component with built-in validation
#[component]
pub fn ValidatedInput(
    field_config: FieldConfig,
    #[prop(optional)] placeholder: Option<String>,
    #[prop(optional)] input_type: Option<String>,
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] debounce_ms: Option<u32>,
    on_change: Callback<String, ()>,
) -> impl IntoView {
    let (field_state, set_value, set_is_touched) =
        use_form_field(String::new(), field_config.clone(), debounce_ms);

    let input_id = format!("input-{}", field_config.name);
    let error_id = format!("{}-error", input_id);

    // Notify parent of value changes
    Effect::new(move |_| {
        let value = field_state.value.get();
        on_change.run(value);
    });

    view! {
        <div class="space-y-1">
            <label
                for=input_id.clone()
                class="block text-sm font-medium text-gray-700"
            >
                {field_config.name.clone()}
                <Show when=move || field_config.required>
                    <span class="text-red-500 ml-1">"*"</span>
                </Show>
            </label>

            <input
                id=input_id.clone()
                type=input_type.unwrap_or_else(|| "text".to_string())
                placeholder=placeholder.unwrap_or_default()
                class=move || {
                    let base_class = class.clone().unwrap_or_else(||
                        "block w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 transition-colors".to_string()
                    );

                    if field_state.error.get().is_some() && field_state.is_touched.get() {
                        format!("{} border-red-300 focus:ring-red-500 focus:border-red-500", base_class)
                    } else {
                        format!("{} border-gray-300 focus:ring-blue-500 focus:border-blue-500", base_class)
                    }
                }
                prop:value=field_state.value
                on:input=move |ev| {
                    let value = event_target_value(&ev);
                    set_value.set(value);
                }
                on:blur=move |_| {
                    set_is_touched.set(true);
                }
                aria-describedby=error_id.clone()
                aria-invalid=move || field_state.error.get().is_some() && field_state.is_touched.get()
            />

            // Error message
            <Show when=move || field_state.error.get().is_some() && field_state.is_touched.get()>
                <p id=error_id.clone() class="text-sm text-red-600">
                    {move || field_state.error.get().unwrap_or_default()}
                </p>
            </Show>
        </div>
    }
}

// Enhanced form submission state
#[derive(Clone, Debug)]
pub struct FormSubmissionState {
    pub is_submitting: Signal<bool>,
    pub error: Signal<Option<String>>,
    pub success: Signal<bool>,
    pub validation_errors: Signal<HashMap<String, String>>,
    pub is_valid: Signal<bool>,
    pub submission_count: Signal<u32>,
}

// Field validation state
#[derive(Clone, Debug)]
pub struct FieldState {
    pub value: Signal<String>,
    pub error: Signal<Option<String>>,
    pub is_touched: Signal<bool>,
    pub is_valid: Signal<bool>,
}

// Form field configuration
#[derive(Clone, Debug)]
pub struct FieldConfig {
    pub name: String,
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub custom_validator: Option<fn(&str) -> Option<String>>,
}

pub fn use_form_submission() -> (
    FormSubmissionState,
    WriteSignal<bool>,
    WriteSignal<Option<String>>,
    WriteSignal<bool>,
    WriteSignal<HashMap<String, String>>,
    WriteSignal<bool>,
    WriteSignal<u32>,
) {
    let (is_submitting, set_is_submitting) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(false);
    let (validation_errors, set_validation_errors) = signal(HashMap::<String, String>::new());
    let (is_valid, set_is_valid) = signal(false);
    let (submission_count, set_submission_count) = signal(0u32);

    let state = FormSubmissionState {
        is_submitting: is_submitting.into(),
        error: error.into(),
        success: success.into(),
        validation_errors: validation_errors.into(),
        is_valid: is_valid.into(),
        submission_count: submission_count.into(),
    };

    (
        state,
        set_is_submitting,
        set_error,
        set_success,
        set_validation_errors,
        set_is_valid,
        set_submission_count,
    )
}

// Enhanced field hook with debounced validation
pub fn use_form_field(
    initial_value: String,
    config: FieldConfig,
    debounce_ms: Option<u32>,
) -> (FieldState, WriteSignal<String>, WriteSignal<bool>) {
    let (value, set_value) = signal(initial_value);
    let (error, set_error) = signal(None::<String>);
    let (is_touched, set_is_touched) = signal(false);
    let (is_valid, set_is_valid) = signal(false);

    let debounce_delay = debounce_ms.unwrap_or(300);

    // Debounced validation effect
    Effect::new(move |_| {
        let current_value = value.get();
        let config_clone = config.clone();

        if is_touched.get() {
            spawn_local(async move {
                TimeoutFuture::new(debounce_delay).await;

                // Re-check if value has changed during debounce
                if current_value == value.get() {
                    let validation_error = validate_field(&current_value, &config_clone);
                    set_error.set(validation_error.clone());
                    set_is_valid.set(validation_error.is_none());
                }
            });
        }
    });

    let state = FieldState {
        value: value.into(),
        error: error.into(),
        is_touched: is_touched.into(),
        is_valid: is_valid.into(),
    };

    (state, set_value, set_is_touched)
}

// Validate a single field based on configuration
pub fn validate_field(value: &str, config: &FieldConfig) -> Option<String> {
    // Required validation
    if config.required {
        if let Some(error) = validate_required(value, &config.name) {
            return Some(error);
        }
    }

    // Skip other validations if field is empty and not required
    if value.trim().is_empty() && !config.required {
        return None;
    }

    // Min length validation
    if let Some(min_len) = config.min_length {
        if let Some(error) = validate_min_length(value, min_len, &config.name) {
            return Some(error);
        }
    }

    // Max length validation
    if let Some(max_len) = config.max_length {
        if let Some(error) = validate_max_length(value, max_len, &config.name) {
            return Some(error);
        }
    }

    // Pattern validation
    if let Some(pattern) = &config.pattern {
        if let Ok(regex) = Regex::new(pattern) {
            if !regex.is_match(value) {
                return Some(format!("{} format is invalid", config.name));
            }
        }
    }

    // Custom validation
    if let Some(validator) = config.custom_validator {
        if let Some(error) = validator(value) {
            return Some(error);
        }
    }

    None
}

// Bulk form validation
pub fn use_form_validation(
    fields: Vec<(Signal<String>, FieldConfig)>,
) -> (Signal<HashMap<String, String>>, Signal<bool>) {
    let (validation_errors, set_validation_errors) = signal(HashMap::<String, String>::new());
    let (is_form_valid, set_is_form_valid) = signal(false);

    // Validate all fields whenever any field changes
    Effect::new(move |_| {
        let mut errors = HashMap::new();

        for (field_value, field_config) in &fields {
            let value = field_value.get();
            if let Some(error) = validate_field(&value, field_config) {
                errors.insert(field_config.name.clone(), error);
            }
        }

        let is_valid = errors.is_empty();
        set_validation_errors.set(errors);
        set_is_form_valid.set(is_valid);
    });

    (validation_errors.into(), is_form_valid.into())
}

// Enhanced validation rules with better error messages
pub fn validate_required(value: &str, field_name: &str) -> Option<String> {
    if value.trim().is_empty() {
        Some(format!("{} is required", field_name))
    } else {
        None
    }
}

pub fn validate_email(email: &str) -> Option<String> {
    let email = email.trim();

    if email.is_empty() {
        return Some("Email is required".to_string());
    }

    // Enhanced email validation
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

    if !email_regex.is_match(email) {
        Some("Please enter a valid email address".to_string())
    } else if email.len() > 254 {
        Some("Email address is too long".to_string())
    } else {
        None
    }
}

pub fn validate_min_length(value: &str, min: usize, field_name: &str) -> Option<String> {
    if value.len() < min {
        Some(format!(
            "{} must be at least {} character{}",
            field_name,
            min,
            if min == 1 { "" } else { "s" }
        ))
    } else {
        None
    }
}

pub fn validate_max_length(value: &str, max: usize, field_name: &str) -> Option<String> {
    if value.len() > max {
        Some(format!(
            "{} must not exceed {} character{}",
            field_name,
            max,
            if max == 1 { "" } else { "s" }
        ))
    } else {
        None
    }
}

pub fn validate_phone_number(phone: &str) -> Option<String> {
    let phone = phone.trim().replace(&[' ', '-', '(', ')', '+'][..], "");

    if phone.is_empty() {
        return Some("Phone number is required".to_string());
    }

    if phone.len() < 10 {
        return Some("Phone number must be at least 10 digits".to_string());
    }

    if phone.len() > 15 {
        return Some("Phone number is too long".to_string());
    }

    if !phone.chars().all(|c| c.is_ascii_digit()) {
        return Some("Phone number must contain only digits".to_string());
    }

    None
}

pub fn validate_password_strength(password: &str) -> Option<String> {
    if password.is_empty() {
        return Some("Password is required".to_string());
    }

    if password.len() < 8 {
        return Some("Password must be at least 8 characters long".to_string());
    }

    if password.len() > 128 {
        return Some("Password is too long".to_string());
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

    let mut missing = Vec::new();

    if !has_uppercase {
        missing.push("uppercase letter");
    }
    if !has_lowercase {
        missing.push("lowercase letter");
    }
    if !has_digit {
        missing.push("number");
    }
    if !has_special {
        missing.push("special character");
    }

    if !missing.is_empty() {
        Some(format!(
            "Password must contain at least one {}",
            missing.join(", ")
        ))
    } else {
        None
    }
}

pub fn validate_pin(pin: &str, min_length: usize) -> Option<String> {
    if pin.is_empty() {
        return Some("PIN is required".to_string());
    }

    if pin.len() < min_length {
        return Some(format!("PIN must be at least {} digits", min_length));
    }

    if !pin.chars().all(|c| c.is_ascii_digit()) {
        return Some("PIN must contain only digits".to_string());
    }

    None
}

pub fn validate_nostr_pubkey(pubkey: &str) -> Option<String> {
    let pubkey = pubkey.trim();

    if pubkey.is_empty() {
        return Some("Nostr public key is required".to_string());
    }

    if !pubkey.starts_with("npub") {
        return Some("Nostr public key must start with 'npub'".to_string());
    }

    if pubkey.len() != 63 {
        // npub + 59 chars
        return Some("Invalid Nostr public key format".to_string());
    }

    None
}
