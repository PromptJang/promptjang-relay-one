use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::domain::{DomainError, ErrorKind};

pub struct AppError(StatusCode, String);

impl AppError {
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self(StatusCode::UNAUTHORIZED, message.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error": self.1}))).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        tracing::error!(%error, "request failed");
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal server error".into(),
        )
    }
}

impl From<DomainError> for AppError {
    fn from(error: DomainError) -> Self {
        let status = match error.kind {
            ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
            ErrorKind::NotFound => StatusCode::NOT_FOUND,
            ErrorKind::Conflict => StatusCode::CONFLICT,
            ErrorKind::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            ErrorKind::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self(status, error.message)
    }
}

pub type ApiResult<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_kinds_map_to_http_status_codes() {
        // Arrange
        let cases = [
            (DomainError::bad_request("x"), StatusCode::BAD_REQUEST),
            (DomainError::not_found("x"), StatusCode::NOT_FOUND),
            (DomainError::conflict("x"), StatusCode::CONFLICT),
            (
                DomainError::payload_too_large("x"),
                StatusCode::PAYLOAD_TOO_LARGE,
            ),
            (
                DomainError::too_many_requests("x"),
                StatusCode::TOO_MANY_REQUESTS,
            ),
            (
                DomainError::internal("x"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ];

        for (error, expected) in cases {
            // Act
            let response: AppError = error.into();

            // Assert
            assert_eq!(response.0, expected);
        }
    }
}
