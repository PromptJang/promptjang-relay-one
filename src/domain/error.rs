use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    BadRequest,
    NotFound,
    Conflict,
    PayloadTooLarge,
    TooManyRequests,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainError {
    pub kind: ErrorKind,
    pub message: String,
}

impl DomainError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::BadRequest, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Conflict, message)
    }

    pub fn payload_too_large(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::PayloadTooLarge, message)
    }

    pub fn too_many_requests(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TooManyRequests, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for DomainError {}

impl From<sqlx::Error> for DomainError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "database operation failed");
        Self::internal("internal server error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_carry_their_kind() {
        // Arrange
        let cases = [
            (DomainError::bad_request("bad"), ErrorKind::BadRequest),
            (DomainError::not_found("gone"), ErrorKind::NotFound),
            (DomainError::conflict("clash"), ErrorKind::Conflict),
            (
                DomainError::payload_too_large("big"),
                ErrorKind::PayloadTooLarge,
            ),
            (
                DomainError::too_many_requests("fast"),
                ErrorKind::TooManyRequests,
            ),
            (DomainError::internal("boom"), ErrorKind::Internal),
        ];

        for (error, kind) in cases {
            // Act
            let actual = error.kind;

            // Assert
            assert_eq!(actual, kind);
        }
    }

    #[test]
    fn sqlx_errors_are_logged_and_sanitized() {
        // Arrange
        let source = sqlx::Error::RowNotFound;

        // Act
        let error = DomainError::from(source);

        // Assert
        assert_eq!(error.kind, ErrorKind::Internal);
        assert_eq!(error.message, "internal server error");
    }
}
