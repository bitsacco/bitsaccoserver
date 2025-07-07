use std::collections::HashMap;

// Enhanced validation rules
pub struct ValidationRules;

impl ValidationRules {
    pub fn required(field_name: &str) -> Box<dyn Fn(&str) -> Option<String>> {
        let field_name = field_name.to_string();
        Box::new(move |value: &str| {
            if value.trim().is_empty() {
                Some(format!("{} is required", field_name))
            } else {
                None
            }
        })
    }

    pub fn min_length(min: usize, field_name: &str) -> Box<dyn Fn(&str) -> Option<String>> {
        let field_name = field_name.to_string();
        Box::new(move |value: &str| {
            if value.len() < min {
                Some(format!(
                    "{} must be at least {} characters",
                    field_name, min
                ))
            } else {
                None
            }
        })
    }

    pub fn max_length(max: usize, field_name: &str) -> Box<dyn Fn(&str) -> Option<String>> {
        let field_name = field_name.to_string();
        Box::new(move |value: &str| {
            if value.len() > max {
                Some(format!(
                    "{} must be no more than {} characters",
                    field_name, max
                ))
            } else {
                None
            }
        })
    }

    pub fn email() -> Box<dyn Fn(&str) -> Option<String>> {
        Box::new(|value: &str| {
            if value.is_empty() {
                return None; // Let required validation handle empty values
            }

            // Simple email validation
            if value.contains('@') && value.contains('.') && value.len() > 5 {
                None
            } else {
                Some("Please enter a valid email address".to_string())
            }
        })
    }

    pub fn numeric() -> Box<dyn Fn(&str) -> Option<String>> {
        Box::new(|value: &str| {
            if value.is_empty() {
                return None;
            }

            if value.parse::<f64>().is_err() {
                Some("Please enter a valid number".to_string())
            } else {
                None
            }
        })
    }

    pub fn positive_number() -> Box<dyn Fn(&str) -> Option<String>> {
        Box::new(|value: &str| {
            if value.is_empty() {
                return None;
            }

            match value.parse::<f64>() {
                Ok(num) if num > 0.0 => None,
                Ok(_) => Some("Please enter a positive number".to_string()),
                Err(_) => Some("Please enter a valid number".to_string()),
            }
        })
    }

    pub fn matches_pattern(pattern: &str, message: &str) -> Box<dyn Fn(&str) -> Option<String>> {
        let pattern = pattern.to_string();
        let message = message.to_string();
        Box::new(move |value: &str| {
            if value.is_empty() {
                return None;
            }

            // Simple pattern matching (you could use regex crate for more complex patterns)
            match pattern.as_str() {
                "phone" => {
                    // Simple phone validation (customize as needed)
                    let cleaned = value
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>();
                    if cleaned.len() >= 10 {
                        None
                    } else {
                        Some(message.clone())
                    }
                }
                _ => None,
            }
        })
    }

    pub fn password_strength() -> Box<dyn Fn(&str) -> Option<String>> {
        Box::new(|value: &str| {
            if value.is_empty() {
                return None;
            }

            let mut errors = Vec::new();

            if value.len() < 8 {
                errors.push("at least 8 characters");
            }

            if !value.chars().any(|c| c.is_uppercase()) {
                errors.push("one uppercase letter");
            }

            if !value.chars().any(|c| c.is_lowercase()) {
                errors.push("one lowercase letter");
            }

            if !value.chars().any(|c| c.is_ascii_digit()) {
                errors.push("one number");
            }

            if !value
                .chars()
                .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
            {
                errors.push("one special character");
            }

            if errors.is_empty() {
                None
            } else {
                Some(format!("Password must contain {}", errors.join(", ")))
            }
        })
    }

    pub fn confirm_password(original_password: String) -> Box<dyn Fn(&str) -> Option<String>> {
        Box::new(move |value: &str| {
            if value != original_password {
                Some("Passwords do not match".to_string())
            } else {
                None
            }
        })
    }
}

// Form validation state
#[derive(Clone, Debug)]
pub struct FormValidation {
    pub errors: HashMap<String, String>,
    pub touched: HashMap<String, bool>,
}

impl FormValidation {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
            touched: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, field: &str, error: &str) {
        self.errors.insert(field.to_string(), error.to_string());
    }

    pub fn remove_error(&mut self, field: &str) {
        self.errors.remove(field);
    }

    pub fn set_touched(&mut self, field: &str) {
        self.touched.insert(field.to_string(), true);
    }

    pub fn is_field_valid(&self, field: &str) -> bool {
        !self.errors.contains_key(field)
    }

    pub fn is_form_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn get_error(&self, field: &str) -> Option<&String> {
        self.errors.get(field)
    }

    pub fn is_field_touched(&self, field: &str) -> bool {
        self.touched.get(field).copied().unwrap_or(false)
    }

    pub fn should_show_error(&self, field: &str) -> bool {
        self.is_field_touched(field) && !self.is_field_valid(field)
    }
}

impl Default for FormValidation {
    fn default() -> Self {
        Self::new()
    }
}
