use crate::domain::DomainError;
use axum::http::HeaderMap;

pub fn validate_name(value: &str) -> Result<(), DomainError> {
    if value.trim().is_empty() || value.len() > 100 {
        Err(DomainError::bad_request(
            "name must contain 1 to 100 characters",
        ))
    } else {
        Ok(())
    }
}

pub fn extract_header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty() && value.len() <= 128)
        .map(String::from)
}

pub fn idempotency_key(headers: &HeaderMap) -> Result<Option<String>, DomainError> {
    match headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
    {
        None => Ok(None),
        Some(value) if value.is_empty() || value.len() > 255 => Err(DomainError::bad_request(
            "Idempotency-Key must contain 1 to 255 characters",
        )),
        Some(value) => Ok(Some(value.to_string())),
    }
}

pub fn ensure_payload_size(size: usize, maximum: usize) -> Result<(), DomainError> {
    if size > maximum {
        Err(DomainError::payload_too_large(format!(
            "payload exceeds configured maximum of {maximum} bytes"
        )))
    } else {
        Ok(())
    }
}
