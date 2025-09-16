use crate::components::forms::{CheckboxField, FormField, SelectField, TextareaField};
use crate::components::ui::Button;
use leptos::prelude::*;
use std::collections::HashMap;

pub struct FormFieldConfig {
    pub name: String,
    pub label: String,
    pub field_type: FormFieldType,
    pub required: bool,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    pub validation_rules: Option<Vec<String>>, // Store validation rule names instead
    pub options: Option<Vec<(String, String)>>, // For select fields
    pub rows: Option<u32>,                     // For textarea fields
}

impl Clone for FormFieldConfig {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            label: self.label.clone(),
            field_type: self.field_type.clone(),
            required: self.required,
            placeholder: self.placeholder.clone(),
            help_text: self.help_text.clone(),
            validation_rules: self.validation_rules.clone(),
            options: self.options.clone(),
            rows: self.rows,
        }
    }
}

#[derive(Clone)]
pub enum FormFieldType {
    Text,
    Email,
    Password,
    Number,
    Tel,
    Select,
    Textarea,
    Checkbox,
}

impl FormFieldType {
    fn to_input_type(&self) -> &'static str {
        match self {
            FormFieldType::Text => "text",
            FormFieldType::Email => "email",
            FormFieldType::Password => "password",
            FormFieldType::Number => "number",
            FormFieldType::Tel => "tel",
            _ => "text",
        }
    }
}

#[component]
pub fn FormBuilder(
    #[prop(into)] title: Option<String>,
    #[prop(into)] description: Option<String>,
    #[prop(into)] fields: Vec<FormFieldConfig>,
    #[prop(into)] submit_text: String,
    #[prop(optional)] cancel_text: Option<String>,
    #[prop(optional)] loading: bool,
    #[prop(optional)] on_submit: Option<Callback<HashMap<String, String>>>,
    #[prop(optional)] on_cancel: Option<Callback<()>>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    // Create signals for all form fields
    let field_signals: Vec<(String, RwSignal<String>, RwSignal<bool>)> = fields
        .iter()
        .map(|field| {
            let name = field.name.clone();
            let value_signal = RwSignal::new(String::new());
            let checkbox_signal = RwSignal::new(false);
            (name, value_signal, checkbox_signal)
        })
        .collect();

    // Form validation state
    let (form_errors, _set_form_errors) = signal(HashMap::<String, String>::new());
    let (_form_touched, _set_form_touched) = signal(HashMap::<String, bool>::new());

    // Check if form is valid
    let fields_clone_for_validation = fields.clone();
    let field_signals_clone_for_validation = field_signals.clone();
    let is_form_valid = Signal::derive(move || {
        form_errors.get().is_empty()
            && fields_clone_for_validation.iter().all(|field| {
                if field.required {
                    if let Some((_, value_signal, checkbox_signal)) =
                        field_signals_clone_for_validation
                            .iter()
                            .find(|(name, _, _)| name == &field.name)
                    {
                        match field.field_type {
                            FormFieldType::Checkbox => checkbox_signal.get(),
                            _ => !value_signal.get().trim().is_empty(),
                        }
                    } else {
                        false
                    }
                } else {
                    true
                }
            })
    });

    // Handle form submission
    let fields_clone_for_submit = fields.clone();
    let field_signals_clone_for_submit = field_signals.clone();
    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        if let Some(callback) = on_submit {
            let mut form_data = HashMap::new();
            for (name, value_signal, checkbox_signal) in &field_signals_clone_for_submit {
                let field = fields_clone_for_submit
                    .iter()
                    .find(|f| &f.name == name)
                    .unwrap();
                let value = match field.field_type {
                    FormFieldType::Checkbox => checkbox_signal.get().to_string(),
                    _ => value_signal.get(),
                };
                form_data.insert(name.clone(), value);
            }
            callback.run(form_data);
        }
    };

    view! {
        <div class=format!("bg-white shadow-sm rounded-lg {}", class.unwrap_or_default())>
            // Header
            {if title.is_some() || description.is_some() {
                view! {
                    <div class="px-6 py-4 border-b border-gray-200">
                        {if let Some(title_text) = title {
                            view! {
                                <h2 class="text-xl font-semibold text-gray-900">
                                    {title_text}
                                </h2>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}
                        {if let Some(desc_text) = description {
                            view! {
                                <p class="mt-1 text-sm text-gray-500">
                                    {desc_text}
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

            // Form content
            <form class="px-6 py-4 space-y-6" on:submit=handle_submit>
                {fields.into_iter().map(|field| {
                    let field_name = field.name.clone();
                    let (_, value_signal, checkbox_signal) = field_signals
                        .iter()
                        .find(|(name, _, _)| name == &field_name)
                        .unwrap();

                    match field.field_type {
                        FormFieldType::Select => {
                            if let Some(options) = field.options {
                                view! {
                                    <SelectField
                                        label=field.label
                                        name=field.name
                                        value=value_signal.read_only()
                                        set_value=value_signal.write_only()
                                        required=field.required
                                        help_text=field.help_text.unwrap_or_default()
                                    >
                                        <option value="">"Please select..."</option>
                                        {options.into_iter().map(|(value, label)| {
                                            view! {
                                                <option value=value>
                                                    {label}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </SelectField>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }
                        FormFieldType::Textarea => {
                            view! {
                                <TextareaField
                                    label=field.label
                                    name=field.name
                                    placeholder=field.placeholder.unwrap_or_default()
                                    value=value_signal.read_only()
                                    set_value=value_signal.write_only()
                                    required=field.required
                                    rows=field.rows.unwrap_or(4)
                                    help_text=field.help_text.unwrap_or_default()
                                />
                            }.into_any()
                        }
                        FormFieldType::Checkbox => {
                            view! {
                                <CheckboxField
                                    label=field.label
                                    name=field.name
                                    checked=checkbox_signal.read_only()
                                    set_checked=checkbox_signal.write_only()
                                    help_text=field.help_text.unwrap_or_default()
                                />
                            }.into_any()
                        }
                        _ => {
                            view! {
                                <FormField
                                    label=field.label
                                    name=field.name
                                    type_=field.field_type.to_input_type()
                                    placeholder=field.placeholder.unwrap_or_default()
                                    value=value_signal.read_only()
                                    set_value=value_signal.write_only()
                                    required=field.required
                                    _validation_rules=vec![]
                                    help_text=field.help_text.unwrap_or_default()
                                    show_success=true
                                />
                            }.into_any()
                        }
                    }
                }).collect_view()}

                // Form actions
                <div class="flex items-center justify-end space-x-3 pt-6 border-t border-gray-200">
                    {if let Some(cancel_label) = cancel_text {
                        view! {
                            <Button
                                variant=crate::components::ui::button::ButtonVariant::Secondary
                                disabled=loading
                                onclick=Callback::new(move |_| {
                                    if let Some(callback) = on_cancel {
                                        callback.run(());
                                    }
                                })
                            >
                                {cancel_label}
                            </Button>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }}

                    <Button
                        type_="submit"
                        disabled=loading || !is_form_valid.get()
                        class="min-w-[120px]"
                    >
                        <Show
                            when=move || loading
                            fallback=move || view! { {submit_text.clone()} }
                        >
                            <div class="flex items-center">
                                <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
                                </svg>
                                "Processing..."
                            </div>
                        </Show>
                    </Button>
                </div>
            </form>
        </div>
    }
}

// Quick form creation utilities
pub struct QuickFormBuilder {
    fields: Vec<FormFieldConfig>,
}

impl QuickFormBuilder {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn add_text_field(mut self, name: &str, label: &str, required: bool) -> Self {
        self.fields.push(FormFieldConfig {
            name: name.to_string(),
            label: label.to_string(),
            field_type: FormFieldType::Text,
            required,
            placeholder: None,
            help_text: None,
            validation_rules: None,
            options: None,
            rows: None,
        });
        self
    }

    pub fn add_email_field(mut self, name: &str, label: &str, required: bool) -> Self {
        self.fields.push(FormFieldConfig {
            name: name.to_string(),
            label: label.to_string(),
            field_type: FormFieldType::Email,
            required,
            placeholder: Some("you@example.com".to_string()),
            help_text: None,
            validation_rules: Some(vec!["email".to_string()]),
            options: None,
            rows: None,
        });
        self
    }

    pub fn add_password_field(
        mut self,
        name: &str,
        label: &str,
        required: bool,
        strong: bool,
    ) -> Self {
        let validation_rules = if strong {
            Some(vec!["password_strength".to_string()])
        } else {
            Some(vec!["min_length_6".to_string()])
        };

        self.fields.push(FormFieldConfig {
            name: name.to_string(),
            label: label.to_string(),
            field_type: FormFieldType::Password,
            required,
            placeholder: None,
            help_text: if strong {
                Some("Must contain at least 8 characters, including uppercase, lowercase, number, and special character".to_string())
            } else {
                Some("At least 6 characters".to_string())
            },
            validation_rules,
            options: None,
            rows: None,
        });
        self
    }

    pub fn add_select_field(
        mut self,
        name: &str,
        label: &str,
        options: Vec<(String, String)>,
        required: bool,
    ) -> Self {
        self.fields.push(FormFieldConfig {
            name: name.to_string(),
            label: label.to_string(),
            field_type: FormFieldType::Select,
            required,
            placeholder: None,
            help_text: None,
            validation_rules: None,
            options: Some(options),
            rows: None,
        });
        self
    }

    pub fn add_textarea_field(
        mut self,
        name: &str,
        label: &str,
        rows: u32,
        required: bool,
    ) -> Self {
        self.fields.push(FormFieldConfig {
            name: name.to_string(),
            label: label.to_string(),
            field_type: FormFieldType::Textarea,
            required,
            placeholder: None,
            help_text: None,
            validation_rules: None,
            options: None,
            rows: Some(rows),
        });
        self
    }

    pub fn add_checkbox_field(mut self, name: &str, label: &str) -> Self {
        self.fields.push(FormFieldConfig {
            name: name.to_string(),
            label: label.to_string(),
            field_type: FormFieldType::Checkbox,
            required: false,
            placeholder: None,
            help_text: None,
            validation_rules: None,
            options: None,
            rows: None,
        });
        self
    }

    pub fn build(self) -> Vec<FormFieldConfig> {
        self.fields
    }
}

impl Default for QuickFormBuilder {
    fn default() -> Self {
        Self::new()
    }
}
