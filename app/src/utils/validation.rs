use axum::http::StatusCode;
use uuid::Uuid;

/// Validate UUID parameter
pub fn validate_uuid(value: &str, field_name: &str) -> Result<Uuid, (StatusCode, String)> {
    Uuid::parse_str(value).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid {} format: must be a valid UUID", field_name),
        )
    })
}

/// Validate non-empty string
pub fn validate_non_empty(value: &str, field_name: &str) -> Result<(), (StatusCode, String)> {
    if value.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("{} cannot be empty", field_name),
        ));
    }
    Ok(())
}

/// Validate positive number
pub fn validate_positive_decimal(
    value: rust_decimal::Decimal,
    field_name: &str,
) -> Result<(), (StatusCode, String)> {
    if value <= rust_decimal::Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("{} must be positive", field_name),
        ));
    }
    Ok(())
}

/// Validate email format (basic validation)
pub fn validate_email(email: &str) -> Result<(), (StatusCode, String)> {
    if !email.contains('@') || !email.contains('.') {
        return Err((StatusCode::BAD_REQUEST, "Invalid email format".to_string()));
    }
    Ok(())
}

/// Validate pagination parameters
pub fn validate_pagination(
    page: Option<u32>,
    per_page: Option<u32>,
) -> Result<(u32, u32), (StatusCode, String)> {
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(20);

    if page == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Page must be greater than 0".to_string(),
        ));
    }

    if per_page == 0 || per_page > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Per page must be between 1 and 100".to_string(),
        ));
    }

    Ok((page, per_page))
}
